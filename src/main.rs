use base64::{Engine as _, engine::general_purpose::STANDARD};
use inquire::{Select, Text};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompt_source = Select::new(
        "Comment voulez-vous entrer le prompt ?",
        vec!["Terminal", "Fichier texte/csv"],
    )
    .prompt()?;

    let prompt = if prompt_source == "Terminal" {
        Text::new("Entrez votre prompt :").prompt()?
    } else {
        let path = Text::new("Entrez le chemin vers votre fichier texte/csv :").prompt()?;
        fs::read_to_string(&path).expect("Impossible de lire le fichier prompt")
    };

    let file_path = Text::new("Entrez le chemin vers votre image (ex: image.png) :").prompt()?;

    let file_bytes = fs::read(&file_path).expect("Impossible de lire le fichier image");
    let base64_file = STANDARD.encode(&file_bytes);

    let mime_type = if file_path.ends_with(".png") {
        "image/png"
    } else if file_path.ends_with(".pdf") {
        "application/pdf"
    } else {
        "image/jpeg"
    };

    let data_url = format!("data:{};base64,{}", mime_type, base64_file);

    let request_body = ChatRequest {
        model: "local-model".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: vec![
                ContentPart::Text { text: prompt },
                ContentPart::ImageUrl {
                    image_url: ImageUrl { url: data_url },
                },
            ],
        }],
        temperature: 0.7,
        max_tokens: 200,
    };

    println!("⏳ Envoi de la requête à LM Studio (http://localhost:1234/v1/chat/completions)...");
    let client = Client::new();
    let res = client
        .post("http://localhost:1234/v1/chat/completions")
        .json(&request_body)
        .send()
        .await?;

    if !res.status().is_success() {
        eprintln!("❌ Erreur API : {:?}", res.text().await?);
        return Ok(());
    }

    let response_data: ChatResponse = res.json().await?;
    let llm_reply = &response_data.choices[0].message.content;

    let output_choice = Select::new(
        "Où voulez-vous afficher la réponse ?",
        vec!["Terminal", "Fichier de sortie (txt)"],
    )
    .prompt()?;

    if output_choice == "Terminal" {
        println!("\n🤖 Réponse du modèle :\n{}\n", llm_reply);
    } else {
        let out_path =
            Text::new("Entrez le nom du fichier de sortie (ex: reponse.txt) :").prompt()?;
        let mut file = fs::File::create(&out_path)?;
        file.write_all(llm_reply.as_bytes())?;
        println!("✅ Réponse sauvegardée dans {}", out_path);
    }

    Ok(())
}
