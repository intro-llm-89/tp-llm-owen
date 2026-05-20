use base64::{Engine as _, engine::general_purpose::STANDARD};
use inquire::{Select, Text};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

// (Keep all your ChatRequest, Message, ContentPart, etc. structures identical here)
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
    // 1. Retrieving the base prompt
    let prompt_source = Select::new(
        "Comment voulez-vous entrer le prompt ?",
        vec!["Terminal", "Fichier texte/csv"],
    )
    .prompt()?;

    let mut prompt = if prompt_source == "Terminal" {
        Text::new("Entrez votre prompt :").prompt()?
    } else {
        let path = Text::new("Entrez le chemin vers votre fichier texte/csv :").prompt()?;
        fs::read_to_string(&path).expect("Impossible de lire le fichier prompt")
    };

    // --- NEW: Anti-Markdown via prompt ---
    // We force the LLM to answer in plain text to avoid Markdown
    prompt.push_str("\n\n(Consigne stricte: Réponds uniquement en texte brut. N'utilise absolument AUCUN formatage Markdown, pas d'astérisques, pas de hashtags, pas de code blocks.)");

    // --- NEW: Windows multiple file selector ---
    println!("📂 Ouverture du sélecteur de fichiers...");
    let selected_files = rfd::FileDialog::new()
        .add_filter("Fichiers pris en charge", &["png", "jpg", "jpeg", "pdf"])
        .set_title("Sélectionnez jusqu'à 5 fichiers")
        .pick_files()
        .unwrap_or_default(); // Returns an empty list if the user closes the window

    if selected_files.is_empty() {
        println!("❌ Aucun fichier sélectionné. Arrêt du programme.");
        return Ok(());
    }

    if selected_files.len() > 5 {
        println!(
            "❌ Erreur : Vous avez sélectionné {} fichiers. La limite est de 5.",
            selected_files.len()
        );
        return Ok(());
    }

    // 3. Preparing the message content
    // Start by adding the text
    let mut message_content = vec![ContentPart::Text { text: prompt }];

    // --- NEW: Loop over selected files ---
    for file_path in selected_files {
        let file_bytes = fs::read(&file_path).expect("Impossible de lire un des fichiers");
        let base64_file = STANDARD.encode(&file_bytes);

        // Retrieve the extension for MIME type
        let ext = file_path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_lowercase();
        let mime_type = match ext.as_str() {
            "png" => "image/png",
            "pdf" => "application/pdf",
            _ => "image/jpeg",
        };

        let data_url = format!("data:{};base64,{}", mime_type, base64_file);

        // Add the file to the content
        message_content.push(ContentPart::ImageUrl {
            image_url: ImageUrl { url: data_url },
        });
    }

    // 4. Building the request
    let request_body = ChatRequest {
        model: "local-model".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: message_content,
        }],
        temperature: 0.7,
        max_tokens: 2048,
    };

    // 5. Sending to the API
    println!(
        "⏳ Envoi de la requête à LM Studio avec {} fichier(s)...",
        request_body.messages[0].content.len() - 1
    );
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

    // --- NEW: Markdown safety cleanup ---
    // In case the model is stubborn, we roughly clean basic markdown tags
    let llm_reply = response_data.choices[0]
        .message
        .content
        .replace("**", "")
        .replace("### ", "")
        .replace("## ", "")
        .replace("# ", "")
        .replace("`", "");

    // 6. Display / Save
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
