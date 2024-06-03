use std::fs;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel, Embedding};
use text_splitter::{TextSplitter};
use std::path::PathBuf;
use std::fs::read_dir;
use std::path::Path;

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

    splitter.chunks(text).map(|v| v.to_string()).collect()
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

pub fn embeddings<S: AsRef<str> + Send + Sync>(texts: Vec<S>) -> anyhow::Result<Vec<Embedding>> {
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

pub fn get_embedding_dim() -> anyhow::Result<usize> {
    let test_model_info = TextEmbedding::get_model_info(&EmbeddingModel::AllMiniLML6V2);
    Ok(test_model_info.dim)
}
    
// =====================================================================
// Tests.
// Use
//   cargo test --release -- --nocapture
// to see dnbg/println output in tests.
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_ml100() {
        let max_len = 100;
        let max_characters = max_len-25..max_len+25;
        let splitter = TextSplitter::new(max_characters);
        let text = "the quick brown fox jumps over the lazy dog. And another sentence. Seven!".to_string();
        let result: Vec<_> = splitter.chunks(&text).collect();
        assert!(result[0] == "the quick brown fox jumps over the lazy dog. And another sentence. Seven!");
    }
    
    #[test]
    fn chunk_ml48() {
        let max_len = 48;
        let max_characters = max_len-12..max_len+12;
        let splitter = TextSplitter::new(max_characters);
        let text = "the quick brown fox jumps over the lazy dog. And another sentence. Seven!".to_string();
        let result: Vec<_> = splitter.chunks(&text).collect();
        assert!(result[0] == "the quick brown fox jumps over the lazy dog.");
        assert!(result[1] == "And another sentence. Seven!");
    }
    
    #[test]
    fn chunk_ml28() {
        let max_len = 28;
        let max_characters = max_len-12..max_len+12;
        let splitter = TextSplitter::new(max_characters);
        let text = "the quick brown fox jumps over the lazy dog. And another sentence. Seven!".to_string();
        let result: Vec<_> = splitter.chunks(&text).collect();
        assert!(result[0] == "the quick brown fox");
        assert!(result[1] == "jumps over the lazy dog.");
        assert!(result[2] == "And another sentence.");
        assert!(result[3] == "Seven!");
    }

    #[test]
    fn chunk_ml12() {
        let max_len = 12;
        let max_characters = max_len-4..max_len+4;
        let splitter = TextSplitter::new(max_characters);
        let text = "the quick brown fox jumps over the lazy dog. And another sentence. Seven!".to_string();
        let result: Vec<_> = splitter.chunks(&text).collect();
        //dbg!("{:?}", &result);
        assert!(result[0] ==  "the quick");
        assert!(result[1] == "brown fox");
        assert!(result[2] == "jumps over");
        assert!(result[3] == "the lazy dog.");
        assert!(result[4] == "And another");
        assert!(result[5] == "sentence.");
        assert!(result[6] == "Seven!");
    }
}
