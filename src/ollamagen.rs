use ollama_rs::{
    generation::completion::{
        request::GenerationRequest, GenerationContext, GenerationResponseStream,
    },
    Ollama,
};
use tokio::io::{stdout, AsyncWriteExt};
use tokio_stream::StreamExt;

use ollama_rs::{
    generation::chat::{ChatMessage},
};


#[tokio::main]
pub async fn _ollama_generate(sys_msg: &str, question: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let mut stdout = stdout();
    let context: Option<GenerationContext> = None;

    ChatMessage::assistant(sys_msg.to_string()); // ??
    //ollama.set_system_response("0".to_string(), sys_msg.to_string());

    /*
    let res = ollama.generate(GenerationRequest::new(model.to_string(), question.to_string())).await;
    if let Ok(res) = res {
        println!("{}", res.response);
    }
    */
    let mut request = GenerationRequest::new(model.into(), question.to_string());
    if let Some(context) = context.clone() {
        request = request.context(context);
    }
    let mut stream: GenerationResponseStream = ollama.generate_stream(request).await?;
    
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

