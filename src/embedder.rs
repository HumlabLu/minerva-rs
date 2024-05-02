
pub fn chunk_string_old(input: String, chunk_size: usize) -> Vec<String> {
    input.chars()
         .collect::<Vec<char>>()  // Convert to a vector of chars
         .chunks(chunk_size)      // Create chunks of specified size
         .map(|chunk| chunk.iter().collect())  // Convert each chunk back to a String
         .collect()               // Collect all chunks into a Vector
}

pub fn chunk_string(input: String, chunk_size: usize) -> Vec<String> {
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
