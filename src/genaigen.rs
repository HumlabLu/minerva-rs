use genai::chat::{ChatMessage, ChatRequest};
use genai::client::Client;
use genai::utils::{print_chat_stream, PrintChatStreamOptions};
use anyhow::{Result, anyhow};

#[tokio::main]
pub async fn genai_generate(sys_msg: &str, question: &str, model: &str) -> Result<()> {
    let chat_req = ChatRequest::new(vec![
        ChatMessage::system(sys_msg),
        ChatMessage::user(question),
    ]);
    
    let client = Client::default();
    let print_options = PrintChatStreamOptions::from_stream_events(false);
    
    let adapter_kind = client.resolve_adapter_kind(model)
        .map_err(|e| anyhow!("Failed to resolve adapter kind: {}", e))?;
    
    println!("\nmodel {model} | {adapter_kind}");

    let chat_res = client.exec_chat_stream(model, chat_req.clone(), None)
        .await
        .map_err(|e| anyhow!("Failed to execute chat stream: {}", e))?;
    
    print_chat_stream(chat_res, Some(&print_options))
        .await
        .map_err(|e| anyhow!("Failed to print chat stream: {}", e))?;
    
    println!();

    Ok(())
}
