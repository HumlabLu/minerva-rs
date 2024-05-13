use std::io::Write;
use tokenizers::Tokenizer;

use candle_core::quantized::gguf_file;
use candle_core::{Tensor};
use candle_transformers::generation::LogitsProcessor;

use candle_transformers::models::quantized_llama as model;
use model::ModelWeights;

use anyhow::{Error as E, Result};
use tqdm::tqdm;
use tqdm::pbar;

use crate::textgen::device;

pub fn run_qmistral(prompt: &str) -> Result<String> {

    // The length of the sample to generate (in tokens).
    let sample_len: usize = 1200;

    // The temperature used to generate samples, use 0 for greedy sampling.
    let temperature: f64 = 0.8;

    // Nucleus sampling probability cutoff.
    let top_p: Option<f64> = None;

    // The seed to use when generating random samples.
    let seed: u64 = 28;//299792458;

    // Display the token for the specified prompt.
    let verbose_prompt: bool = false;

    // Penalty to be applied for repeating tokens, 1. means no penalty.
    let repeat_penalty: f32 = 1.1;

    // The context size to consider for the repeat penalty.
    let repeat_last_n: usize = 64;

    let temperature = if temperature == 0. {
        None
    } else {
        Some(temperature)
    };

    // /Users/pberck/.cache/huggingface/hub/models--TheBloke--Mistral-7B-Instruct-v0.2-GGUF
    // /Users/pberck/.cache/huggingface/hub/models--TheBloke--Mistral-7B-Instruct-v0.1-GGUF

    //let repo = "TheBloke/Mistral-7B-v0.1-GGUF";
    let repo = "TheBloke/Mistral-7B-Instruct-v0.2-GGUF"; // 0.1, 0.2

    // See list on https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.1-GGUF
   
    let filename = "mistral-7b-instruct-v0.2.Q5_K_M.gguf"; // Twice as slow as Q4_K_M
    //let filename = "mistral-7b-instruct-v0.2.Q6_K.gguf"; // Twice as slow as Q4_K_M
    println!("Model file {}", filename);
    //let filename = "mistral-7b-instruct-v0.1.Q4_K_S.gguf";
    //let filename = "mistral-7b-instruct-v0.1.Q2_K.gguf"; // 0.1, 0.2
    
    let api = hf_hub::api::sync::Api::new()?;
    let api = api.model(repo.to_string());
    let model_path = api.get(filename)?;
    
    let mut file = std::fs::File::open(model_path)?;
    let start = std::time::Instant::now();
    let device = device(false)?; //Device::Cpu;
    println!("Device {:?}", device);
    
    let mut model = {
        let model = gguf_file::Content::read(&mut file)?;
        let mut total_size_in_bytes = 0;
        for (_, tensor) in model.tensor_infos.iter() {
            let elem_count = tensor.shape.elem_count();
            total_size_in_bytes += elem_count
                * tensor.ggml_dtype.type_size()
                / tensor.ggml_dtype.block_size();
        }

        println!(
            "loaded {:?} tensors ({}) in {:.2}s",
            model.tensor_infos.len(),
            &format_size(total_size_in_bytes),
            start.elapsed().as_secs_f32(),
        );
        ModelWeights::from_gguf(model, &mut file, &device)?
    };

    println!("model built");
    println!("model::MAX_SEQ_LEN {}", model::MAX_SEQ_LEN);
    
    let api = hf_hub::api::sync::Api::new().expect("api?");
    let repo = "mistralai/Mistral-7B-v0.1";
    let api = api.model(repo.to_string());

    let tokenizer_path = api.get("tokenizer.json").expect("tokeniser?");
    println!("{:?}", tokenizer_path);
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(E::msg)?;
    
    let pre_prompt_tokens = vec![]; // PJB remove
    let mut response = String::new();
    
    let prompt = format!("[INST] {prompt} [/INST]");
    //let prompt = format!("{prompt}");
    println!("{}", &prompt);
    
    let tokens = tokenizer.encode(prompt, true).map_err(E::msg)?;
    println!("Prompt length {}", tokens.len());
        
    if verbose_prompt {
        for (token, id) in tokens.get_tokens().iter().zip(tokens.get_ids().iter()) {
            let token = token.replace('▁', " ").replace("<0x0A>", "\n");
            println!("{id:7} -> '{token}'");
        }
    }

    let prompt_tokens = [&pre_prompt_tokens, tokens.get_ids()].concat();
    let to_sample = sample_len.saturating_sub(1);

    let prompt_tokens = if prompt_tokens.len() + to_sample > model::MAX_SEQ_LEN - 10 {
        let to_remove = prompt_tokens.len() + to_sample + 10 - model::MAX_SEQ_LEN;
        prompt_tokens[prompt_tokens.len().saturating_sub(to_remove)..]
            .to_vec()
    } else {
        prompt_tokens
    };

    let mut all_tokens = vec![];
    let mut logits_processor = LogitsProcessor::new(seed, temperature, top_p);

    let start_prompt_processing = std::time::Instant::now();
    let mut next_token = {
        let input = Tensor::new(prompt_tokens.as_slice(), &device)?
            .unsqueeze(0)?;
        let logits = model.forward(&input, 0)?;
        let logits = logits.squeeze(0)?;
        logits_processor.sample(&logits)?
    };
    
    let prompt_dt = start_prompt_processing.elapsed();
    all_tokens.push(next_token);
    //print_token(next_token, &tokenizer); // PJB if verbose?
    if let Some(token) = get_token(next_token, &tokenizer)  {
        response += &token; // first character
    }

    let eos_token = *tokenizer.get_vocab(true).get("</s>").unwrap();

    let start_post_prompt = std::time::Instant::now();
    let mut pbar = pbar(Some(to_sample));
    for index in 0..to_sample {
        let input = Tensor::new(&[next_token], &device)?.unsqueeze(0)?;
        let logits = model.forward(&input, prompt_tokens.len() + index)?;
        let logits = logits.squeeze(0)?;
        let logits = if repeat_penalty == 1. {
            logits
        } else {
            let start_at = all_tokens.len().saturating_sub(repeat_last_n);
            candle_transformers::utils::apply_repeat_penalty(
                &logits,
                repeat_penalty,
                &all_tokens[start_at..],
            )?
        };
        next_token = logits_processor.sample(&logits)?;
        all_tokens.push(next_token);
        
        //print_token(next_token, &tokenizer); // PJB if verbose?
        if next_token == eos_token {
            break;
        };
        if let Some(token) = get_token(next_token, &tokenizer)  {
            response += &token;
            pbar.update(1).unwrap();
        }
    } 

    let dt = start_post_prompt.elapsed();
    println!(
        "\n\n{:4} prompt tokens processed: {:.2} token/s",
        prompt_tokens.len(),
        prompt_tokens.len() as f64 / prompt_dt.as_secs_f64(),
    );

    println!(
        "{:4} tokens generated: {:.2} token/s",
        all_tokens.len(),
        all_tokens.len()  as f64 / dt.as_secs_f64(),
    );


    Ok(response.trim().to_string())
}

#[allow(dead_code)]
fn print_token(next_token: u32, tokenizer: &Tokenizer) {
    // https://github.com/huggingface/tokenizers/issues/1141#issuecomment-1562644141
    if let Some(text) = tokenizer.id_to_token(next_token) {
        let text = text.replace('▁', " ");
        // Convert to ascii
        let ascii = text // ascii: Option<u8>, text:String
            .strip_prefix("<0x")
            .and_then(|t| t.strip_suffix('>'))
            .and_then(|t| u8::from_str_radix(t, 16).ok());
        match ascii {
            None => print!("{text}"), // ok, use string anyway
            Some(ascii) => {
                if let Some(chr) = char::from_u32(ascii as u32) {
                    if chr.is_ascii() {
                        print!("{chr}")
                    }
                }
            }
        }
        let _ = std::io::stdout().flush();
    }
}

fn get_token(next_token: u32, tokenizer: &Tokenizer) -> Option<String> {
    if let Some(text) = tokenizer.id_to_token(next_token) {
        let text = text.replace('▁', " ");
        let ascii = text
            .strip_prefix("<0x")
            .and_then(|t| t.strip_suffix('>'))
            .and_then(|t| u8::from_str_radix(t, 16).ok());
        match ascii {
            None => Some(text), // return string directly
            Some(ascii) => {
                if let Some(chr) = char::from_u32(ascii as u32) {
                    if chr.is_ascii() {
                        return Some(chr.to_string());
                    }
                }
                None
            }
        }
    } else {
        None
    }
}

fn format_size(size_in_bytes: usize) -> String {
    if size_in_bytes < 1_000 {
        format!("{}B", size_in_bytes)
    } else if size_in_bytes < 1_000_000 {
        format!("{:.2}KB", size_in_bytes as f64 / 1e3)
    } else if size_in_bytes < 1_000_000_000 {
        format!("{:.2}MB", size_in_bytes as f64 / 1e6)
    } else {
        format!("{:.2}GB", size_in_bytes as f64 / 1e9)
    }

}
