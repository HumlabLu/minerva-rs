use ollama_rs::{
    generation::completion::{
        request::GenerationRequest, GenerationContext, GenerationResponseStream,
    },
    Ollama,
};
use ollama_rs::generation::options::GenerationOptions;
use tokio::io::{stdout, AsyncWriteExt};
use tokio_stream::StreamExt;

use ollama_rs::{
    generation::chat::{ChatMessage},
};


#[tokio::main]
pub async fn ollama_generate(sys_msg: &str, question: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut ollama = Ollama::default();
    let mut stdout = stdout();
    let context: Option<GenerationContext> = None;

    ChatMessage::system(sys_msg.to_string());
    //ChatMessage::assistant(sys_msg.to_string()); // ??

    ollama.set_system_response("0".to_string(), sys_msg.to_string()); // ?

    /*
    let res = ollama.generate(GenerationRequest::new(model.to_string(), question.to_string())).await;
    if let Ok(res) = res {
        println!("{}", res.response);
    }
     */
    let options = GenerationOptions::default()
        .num_ctx(42000)
        .temperature(0.9)
        .repeat_penalty(1.5)
        .repeat_last_n(-1)
        .top_k(100)
        .top_p(0.9);

    // This is needed to get a correct response from the context?
    let prompt = format!("System: {}\nUser: {}", sys_msg, question);

    // Only way to get the context for the sys_msg in ollama? Different from
    // Python interface?
    //let mut request = GenerationRequest::new(model.into(), question.to_string()).options(options);
    let mut request = GenerationRequest::new(model.into(), prompt.to_string()).options(options);
    
    if let Some(context) = context.clone() {
        request = request.context(context);
    }
    let mut stream: GenerationResponseStream = ollama.generate_stream(request).await?;

    println!(" -- ");
    while let Some(Ok(res)) = stream.next().await {
        for ele in res {
            stdout.write_all(ele.response.as_bytes()).await?;
            stdout.flush().await?;
            
            if ele.context.is_some() {
                //context = ele.context;
                _ = ele.context;
            }
        }
    }
    println!("");

    Ok(())
}

