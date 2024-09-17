//use genai::chat::{ChatMessage, ChatRequest};
//use genai::client::Client;
//use genai::utils::{print_chat_stream, PrintChatStreamOptions};

use genai::chat::printer::print_chat_stream;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest};
use genai::{Client, ClientConfig};

//const MODEL_OLLAMA: &str = "mistral"; //"gpt-3.5-turbo"; 

#[tokio::main]
/*
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
    
    let chat_res = client.exec_chat_stream(model, chat_req.clone(), None).await.expect("Ollama:");
    print_chat_stream(chat_res, Some(&print_options)).await?;
    
    println!();

    Ok(())
}
 */

pub async fn genai_generate(sys_msg: &str, question: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    // -- Global ChatOptions
    // Note: The properties of ChatOptions set at the client config level will be
    //       the fallback values if not provided at the chat execution level.
    let client_config =
	ClientConfig::default().with_chat_options(ChatOptions::default().with_temperature(0.9).with_top_p(0.9));
    
    // -- Build the new client with this client_config
    let client = Client::builder().with_config(client_config).build();
    
    // -- Build the chat request
    let chat_req = ChatRequest::new(vec![
	// -- Messages (de/activate to see the differences)
	ChatMessage::system(sys_msg),
	ChatMessage::user(question),
    ]);
    
    // -- Build the chat request options (used per execution chat)
    let options = ChatOptions::default().with_max_tokens(20000);
    
    // -- Execute and print
    println!("\n--- Question:\n{question}");
    let chat_res = client.exec_chat_stream(model, chat_req.clone(), Some(&options)).await?;
    
    let adapter_kind = client.resolve_model_iden(model)?.adapter_kind;
    println!("\n--- Answer: ({model} - {adapter_kind})");
    print_chat_stream(chat_res, None).await?;
    
    Ok(())
}
