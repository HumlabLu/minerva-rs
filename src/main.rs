use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};


fn main() -> anyhow::Result<()> {
    // With default InitOptions
    //let model = TextEmbedding::try_new(Default::default()).expect("Cannot initialise model.");

    //dbg!(TextEmbedding::list_supported_models());
    
    // With custom InitOptions
    let model = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::AllMiniLML6V2,
        show_download_progress: true,
        ..Default::default()
    }).expect("Cannot Initialise model.");

    let documents = vec![
        "passage: Hello, World!",
        "query: Hello, World!",
        "passage: This is an example passage.",
        // You can leave out the prefix but it's recommended
        "fastembed-rs is licensed under Apache  2.0"
    ];
    
    // Generate embeddings with the default batch size, 256
    let embeddings = model.embed(documents, None).expect("Cannot create embeddings.");
    //println!("{:?}", embeddings);
    
    println!("Embeddings length: {}", embeddings.len()); // -> Embeddings length: 4
    println!("Embedding dimension: {}", embeddings[0].len()); // -> Embedding dimension: 384

    // ----

    
    Ok(())
}

// FEST 7e maj!
