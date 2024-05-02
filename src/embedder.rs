
pub fn chunk_string(input: String, chunk_size: usize) -> Vec<String> {
    input.chars()
         .collect::<Vec<char>>()  // Convert to a vector of chars
         .chunks(chunk_size)      // Create chunks of specified size
         .map(|chunk| chunk.iter().collect())  // Convert each chunk back to a String
         .collect()               // Collect all chunks into a Vector
}
