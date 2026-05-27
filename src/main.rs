use base64::{Engine as _, engine::general_purpose::STANDARD};
use inquire::{Confirm, Select, Text};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Write;

/// The structure representing the chat request to the LLM
///
/// `model`: The name of the model to use for the chat request
/// `messages`: The list of messages to send to the model
/// `temperature`: The temperature to use for the chat request
/// `max_tokens`: The maximum number of tokens to generate
/// `tools`: The list of tools to use for the chat request
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Serialize, Debug)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionDef,
}

#[derive(Serialize, Debug)]
struct FunctionDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Serialize)]
struct Message {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<MessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize, Clone)]
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

#[derive(Deserialize, Debug)]
struct ResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FunctionCall {
    name: String,
    arguments: String,
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

// ATTENTION: Never do this in production !
const WEATHER_API_KEY: &str = "2e793b07cddc4adff79978460baaf41a";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompt_source = Select::new(
        "How do you want to input the prompt?",
        vec!["Terminal", "Text/CSV file"],
    )
    .prompt()?;

    let mut prompt = if prompt_source == "Terminal" {
        Text::new("Enter your prompt:").prompt()?
    } else {
        let path = Text::new("Enter the path to your text/csv file:").prompt()?;
        fs::read_to_string(&path).expect("Failed to read the prompt file")
    };

    prompt.push_str("\n\n(Strict instruction: Answer in plain text only. Do absolutely NOT use Markdown formatting, no asterisks, no hashtags, no code blocks.)");

    let attach_files = Confirm::new("Do you want to attach files (images/pdf) to your prompt?")
        .with_default(false)
        .prompt()?;

    let mut selected_files = Vec::new();

    if attach_files {
        println!("Opening file selector...");
        selected_files = rfd::FileDialog::new()
            .add_filter("Supported files", &["png", "jpg", "jpeg", "pdf"])
            .set_title("Select up to 5 files")
            .pick_files()
            .unwrap_or_default();

        if selected_files.len() > 5 {
            println!(
                "Error: You selected {} files. The limit is 5.",
                selected_files.len()
            );
            return Ok(());
        }

        if selected_files.is_empty() {
            println!("No file selected. Proceeding with text only.");
        }
    }

    let mut message_content = vec![ContentPart::Text { text: prompt }];

    for file_path in &selected_files {
        let file_bytes = fs::read(file_path).expect("Failed to read one of the files");
        let base64_file = STANDARD.encode(&file_bytes);

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

        message_content.push(ContentPart::ImageUrl {
            image_url: ImageUrl { url: data_url },
        });
    }

    let tools = Some(vec![Tool {
        tool_type: "function".to_string(),
        function: FunctionDef {
            name: "get_weather".to_string(),
            description: "Get the weather for a specific location using the given location."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The location to get the weather for (ex: Paris, France)"
                    }
                },
                "required": ["location"]
            }),
        },
    }]);

    let request_body = ChatRequest {
        model: "local-model".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: Some(MessageContent::Parts(message_content.clone())),
            tool_calls: None,
            tool_call_id: None,
        }],
        temperature: 0.7,
        max_tokens: 2048,
        tools,
    };

    println!(
        "Sending the request to LM Studio with {} file(s)...",
        selected_files.len()
    );

    let client = Client::new();
    let res = client
        .post("http://localhost:1234/v1/chat/completions")
        .json(&request_body)
        .send()
        .await?;

    if !res.status().is_success() {
        eprintln!("API Error: {:?}", res.text().await?);
        return Ok(());
    }

    let response_data: ChatResponse = res.json().await?;
    let message = &response_data.choices[0].message;

    let mut final_llm_reply = String::new();

    let has_tool_calls = match &message.tool_calls {
        Some(calls) => !calls.is_empty(),
        None => false,
    };

    if has_tool_calls {
        let tool_calls = message.tool_calls.as_ref().unwrap();
        let mut conversation_history = vec![
            Message {
                role: "user".to_string(),
                content: Some(MessageContent::Parts(message_content)),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "assistant".to_string(),
                content: None,
                tool_calls: Some(tool_calls.clone()),
                tool_call_id: None,
            },
        ];

        for tool_call in tool_calls {
            if tool_call.function.name == "get_weather" {
                let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;
                let location = args["location"].as_str().unwrap_or("Paris");

                println!("Location found: {}", location);

                let url = format!(
                    "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}",
                    location, WEATHER_API_KEY
                );

                let weather_data = reqwest::get(&url).await?.text().await?;
                println!("Raw weather result retrieved.");

                conversation_history.push(Message {
                    role: "tool".to_string(),
                    content: Some(MessageContent::Text(weather_data)),
                    tool_calls: None,
                    tool_call_id: Some(tool_call.id.clone()),
                });
            }
        }

        println!("Sending tool result to LLM to generate final response...");
        let second_request_body = ChatRequest {
            model: "local-model".to_string(),
            messages: conversation_history,
            temperature: 0.7,
            max_tokens: 2048,
            tools: None,
        };

        let res2 = client
            .post("http://localhost:1234/v1/chat/completions")
            .json(&second_request_body)
            .send()
            .await?;

        let second_response_data: ChatResponse = res2.json().await?;
        if let Some(content) = &second_response_data.choices[0].message.content {
            final_llm_reply = content.clone();
        }
    } else {
        if let Some(content) = &message.content {
            final_llm_reply = content.clone();
        }
    }

    let llm_reply = final_llm_reply
        .replace("**", "")
        .replace("### ", "")
        .replace("## ", "")
        .replace("# ", "")
        .replace("`", "");

    let output_choice = Select::new(
        "Where do you want to display the response?",
        vec!["Terminal", "Output file (txt)"],
    )
    .prompt()?;

    if output_choice == "Terminal" {
        println!("\nModel response:\n{}\n", llm_reply);
    } else {
        let out_path = Text::new("Enter the output file name (e.g., response.txt):").prompt()?;
        let mut file = fs::File::create(&out_path)?;
        file.write_all(llm_reply.as_bytes())?;
        println!("Response saved to {}", out_path);
    }

    Ok(())
}
