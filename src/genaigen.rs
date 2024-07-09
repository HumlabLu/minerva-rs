use genai::chat::{ChatMessage, ChatRequest};
use genai::client::Client;
use genai::utils::{print_chat_stream, PrintChatStreamOptions};

//const MODEL_OLLAMA: &str = "mistral"; //"gpt-3.5-turbo"; 

#[tokio::main]
pub async fn genai_generate(sys_msg: &str, question: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    let chat_req = ChatRequest::new(vec![
	// -- Messages (de/activate to see the differences)
	ChatMessage::system(sys_msg),
	ChatMessage::user(question),
    ]);
    
    let client = Client::default();
    let print_options = PrintChatStreamOptions::from_stream_events(false);
    
    //let model = MODEL_OLLAMA;
    let adapter_kind = client.resolve_adapter_kind(model)?;
    
    println!("\nmodel {model} | {adapter_kind}");

    /*
    println!("\n--- Answer: (oneshot response)");
    let chat_res = client.exec_chat(model, chat_req.clone(), None).await?;
    println!("{}", chat_res.content.as_deref().unwrap_or("NO ANSWER"));
     */
    
    let chat_res = client.exec_chat_stream(model, chat_req.clone(), None).await?;
    print_chat_stream(chat_res, Some(&print_options)).await?;
    
    println!();

    Ok(())
}
