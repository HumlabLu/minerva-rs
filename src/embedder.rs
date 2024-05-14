use std::fs;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel, Embedding};
use text_splitter::{TextSplitter};
use std::path::PathBuf;
use std::fs::read_dir;
use std::path::Path;

// Chunk blindly into parts.
pub fn _chunk_string_0(input: String, chunk_size: usize) -> Vec<String> {
    input.chars()
         .collect::<Vec<char>>()  // Convert to a vector of chars
         .chunks(chunk_size)      // Create chunks of specified size
         .map(|chunk| chunk.iter().collect())  // Convert each chunk back to a String
         .collect()               // Collect all chunks into a Vector
}

// Chunk blindly into parts but merge small chunks with the previous chunk.
pub fn _chunk_string_1(input: String, chunk_size: usize) -> Vec<String> {
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

// Chunk around whitespace, try to get the number of characters close
// to the suggested chunk_size.
pub fn _chunk_string_2(input: &str, chunk_size: usize) -> Vec<String> {
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

// Chunk on end-of-sentence markers.
pub fn _chunk_string_3(text: &str, max_len: usize) -> Vec<String> {
    let mut sentences = Vec::new();
    let punctuation_marks = ['.', '?', '!'];

    // Split text into sentences based on punctuation, preserving the delimiter
    let mut start = 0;
    for (index, _matched) in text.match_indices(|c| punctuation_marks.contains(&c)) {
        let end = index + 1;
        let sentence = text[start..end].trim().to_string();
        start = end;

        // Handle each sentence based on its length
        if sentence.chars().count() > max_len {
            // Further split the sentence by commas if it's too long
            let parts = sentence.split_terminator(',').collect::<Vec<_>>();
            let mut current_part = String::new();

            for part in parts {
                let trimmed_part = part.trim();
                if trimmed_part.is_empty() {
                    continue; // Skip empty parts resulting from multiple commas
                }

                // Append part to the current part with a comma only if current_part is not empty
                let new_part = if current_part.is_empty() {
                    trimmed_part.to_string()
                } else {
                    format!("{}, {}", current_part, trimmed_part)
                };

                // Check if adding this part exceeds max_len
                if new_part.len() > max_len {
                    // If current_part is not empty, push it to sentences
                    if !current_part.is_empty() {
                        sentences.push(current_part);
                    }
                    // Start new current_part with the current trimmed_part
                    current_part = trimmed_part.to_string();
                } else {
                    // Continue building the current_part
                    current_part = new_part;
                }
            }

            // Push the last part if not empty
            if !current_part.is_empty() {
                sentences.push(current_part);
            }
        } else {
            // Push shorter sentences directly
            sentences.push(sentence);
        }
    }

    // Check if there's any residual text after the last punctuation mark
    if start < text.len() {
        let residual = text[start..].trim();
        if !residual.is_empty() {
            sentences.push(residual.to_string());
        }
    }

    sentences
}

// Use textsplitter-rs.
/*
text_splitter = RecursiveCharacterTextSplitter(
        chunk_size=800,
        chunk_overlap=80,
        length_function=len,
        is_separator_regex=False,
)
*/
pub fn chunk_string(text: &str, max_len: usize) -> Vec<String> {
    // Maximum number of characters in a chunk
    let max_characters = max_len-25..max_len+25; //225..275;
    let splitter = TextSplitter::new(max_characters);
    let chunks = splitter.chunks(text).collect::<Vec<_>>();
    let chunks: Vec<String> = chunks.into_iter().map(|v| v.to_string()).collect(); // hmmpf.

    chunks
}

// Return a vector with filenames with correct extension.
pub fn read_dir_contents<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<PathBuf>> {
    // Read the directory
    let mut file_paths = Vec::new();

    for entry in read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "xml" || ext == "txt" || ext == "md" {
                    file_paths.push(path);
                }
            }
        } else if path.is_dir() { // Meander down into sub-directories.
            println!("Dir {:?}", path);
            for fp in read_dir_contents(path).unwrap() {
                file_paths.push(fp);
            }
        }
    }
    Ok(file_paths)   
}

pub fn embed_file_txt<P: AsRef<Path>>(path: P, chunk_size: usize) -> anyhow::Result<Vec<String>> {
    let contents = fs::read_to_string(path)?;
    Ok(chunk_string(&contents, chunk_size))
}

pub fn embed_file_pdf<P: AsRef<Path>>(path: P, chunk_size: usize) -> anyhow::Result<Vec<String>> {
    let bytes = std::fs::read(path).unwrap();
    let out = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    Ok(chunk_string(&out, chunk_size))
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
