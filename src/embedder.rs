use std::fs;
use anyhow::{Result};
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel, Embedding};

pub fn chunk_string_0(input: String, chunk_size: usize) -> Vec<String> {
    input.chars()
         .collect::<Vec<char>>()  // Convert to a vector of chars
         .chunks(chunk_size)      // Create chunks of specified size
         .map(|chunk| chunk.iter().collect())  // Convert each chunk back to a String
         .collect()               // Collect all chunks into a Vector
}

pub fn chunk_string_1(input: String, chunk_size: usize) -> Vec<String> {
    let mut chunks: Vec<String> = input.chars()
        .collect::<Vec<char>>()  // Convert to a vector of chars
        .chunks(chunk_size)      // Create chunks of specified size
        .map(|chunk| chunk.iter().collect())  // Convert each chunk back to a String
        .collect();              // Collect all chunks into a Vector
    
    // Check if the last chunk is less than 2 characters and there's more than one chunk to merge with
    if chunks.last().map_or(false, |chunk| chunk.len() < 2 && chunks.len() > 1) {
        let last_chunk = chunks.pop().unwrap();  // Remove the last chunk
        let last_but_one_chunk = chunks.pop().unwrap();  // Remove the new last chunk
        chunks.push(last_but_one_chunk + &last_chunk);  // Concatenate and push back
    }
    
    chunks
}

pub fn chunk_string(input: &str, chunk_size: usize) -> Vec<String> {
    let words = input.split_whitespace().collect::<Vec<&str>>();
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for word in words {
        // Check if adding this word plus a space would exceed the chunk size.
        if current_chunk.len() + word.len() + 1 > chunk_size {
            if !current_chunk.is_empty() {
                // Push the current chunk to the chunks vector.
                chunks.push(current_chunk);
                current_chunk = String::new(); // Reset current chunk.
            }
        }
        // Add word to the current chunk, with space if needed.
        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(word);
    }

    // Possible left-over chunk.
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

pub fn embed_file_txt(path: &str, chunk_size: usize) -> anyhow::Result<Vec<String>> {
    let contents = fs::read_to_string(path)?;
    Ok(chunk_string(&contents, chunk_size))
}


pub fn embeddings<S: AsRef<str> + Send + Sync>(texts: Vec<S>) -> anyhow::Result<Vec<Embedding>>  {
    // Instantiate the model.
    let model = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::AllMiniLML6V2,
        show_download_progress: true,
        ..Default::default()
    }).expect("Cannot Initialise model.");
    
    // Generate embeddings.
    let embeddings = model.embed(texts, None).expect("Cannot create embeddings.");
    Ok(embeddings)
}
