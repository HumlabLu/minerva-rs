// Candle examples
// https://github.com/huggingface/candle/blob/main/candle-examples/examples/quantized/main.rs

// Some examples:
// https://rust.marcoinacio.com/data/candle/

// Mistral
// https://prest.blog/llm-mistral

// Tokens
// https://discuss.huggingface.co/t/how-to-pass-the-api-token-using-transformers-candle-rust/84400

// See https://github.com/huggingface/candle/blob/96f1a28e390fceeaa12b3272c8ac5dcccc8eb5fa/candle-examples/examples/phi/main.rs

use anyhow::{Error as E, Result};
use candle_core::{DType, Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_mixformer::Config;
use candle_transformers::models::quantized_mixformer::MixFormerSequentialForCausalLM as QMixFormer;
use hf_hub::{api::sync::Api, Repo};
use lazy_static::lazy_static;
use serde_json::json;
use tokenizers::Tokenizer;
use candle_core::utils::{cuda_is_available, metal_is_available};

// https://github.com/huggingface/candle/blob/main/candle-examples/src/lib.rs
pub fn device(cpu: bool) -> Result<Device> {
    if cpu {
        Ok(Device::Cpu)
    } else if cuda_is_available() {
        Ok(Device::new_cuda(0)?)
    } else if metal_is_available() {
        Ok(Device::new_metal(0)?)
    } else {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            /*println!(
                "Running on CPU, to run on GPU(metal), build this example with `--features metal`"
            );*/
        }
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            println!("Running on CPU, to run on GPU, build this example with `--features cuda`");
        }
        Ok(Device::Cpu)
    }
}

lazy_static! {
    pub static ref PHI: (QMixFormer, Tokenizer) = load_model().expect("Unable to load model");
}

/*
Prompt format: This model uses ChatML prompt format.

<|im_start|>system
You are Dolphin, a helpful AI assistant.<|im_end|>
<|im_start|>user
{prompt}<|im_end|>
<|im_start|>assistant

Example:

<|im_start|>system
You are an AI assistant expert at dolphin training<|im_end|>
<|im_start|>user
Please give ideas and a detailed plan about how to assemble and train an army of dolphin companions to swim me anywhere I want to go and protect me from my enemies and bring me fish to eat.<|im_end|>
<|im_start|>assistant
Assembling, training, and utilizing dolphins as your companions for transportation, protection, and fishing is no small task. However, with careful planning and execution, it can be accomplished. Here's a detailed guide on how to achieve this:

1. **Acquiring Dolphins**: Finding dolphins isn't easy, but you could potentially find them in the wild or at aquariums. For ethical reasons, we suggest acquiring adult dolphins that have been born into captivity. They may not have had as much exposure to humans as their wild counterparts, which makes them easier to handle.

2. ...
 */
// mistral-7b-instruct-v0.2.Q4_K_M.gguf
// zephyr-7b-beta.Q4_K_M.gguf
// See also https://prest.blog/llm-mistral
pub fn load_model() -> Result<(QMixFormer, Tokenizer)> {
    let api = Api::new()?.repo(Repo::model(
        "Demonthos/dolphin-2_6-phi-2-candle".to_string(),
        //"lmz/candle-mistral".to_string(),
    ));
    let tokenizer_filename = api.get("tokenizer.json")?;
    let weights_filename = api.get("model-q4k.gguf")?;

    let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;
    let config = Config::v2();
    let vb = candle_transformers::quantized_var_builder::VarBuilder::from_gguf(&weights_filename, &device(false)?)?;
    let model = QMixFormer::new_v2(&config, vb)?;

    Ok((model, tokenizer))
}

struct TextGeneration {
    model: QMixFormer,
    device: Device,
    tokenizer: Tokenizer,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    fn new(
        model: QMixFormer,
        tokenizer: Tokenizer,
        seed: u64,
        temp: Option<f64>,
        top_p: Option<f64>,
        repeat_penalty: f32,
        repeat_last_n: usize,
        device: &Device,
    ) -> Self {
        let logits_processor = LogitsProcessor::new(seed, temp, top_p);
        Self {
            model,
            tokenizer,
            logits_processor,
            repeat_penalty,
            repeat_last_n,
            device: device.clone(),
        }
    }

    fn run(&mut self, prompt: &str, sample_len: usize) -> Result<String> {
        //debug!(prompt = prompt, "starting the inference loop");
        let tokens = self.tokenizer.encode(prompt, true).map_err(E::msg)?;
        if tokens.is_empty() {
            anyhow::bail!("Empty prompts are not supported in the phi model.")
        }
        let mut tokens = tokens.get_ids().to_vec();
        let mut generated_tokens = 0usize;
        let eos_token = match self.tokenizer.get_vocab(true).get("<|endoftext|>") {
            Some(token) => *token,
            None => anyhow::bail!("cannot find the endoftext token"),
        };
        println!("eos token {}", eos_token);
        
        //let token = self.tokenizer.decode(&[198], true).map_err(E::msg)?;
        //println!("token '{}'", token); // \n (or "\n\n"?)
        /*let eol_token = match self.tokenizer.get_vocab(true).get("apple") {
            Some(token) => *token,
            None => anyhow::bail!("cannot find the token"),
        };
        println!("eol token {}", eol_token);*/
        
        let start_gen = std::time::Instant::now();

        let mut response = String::new();

        //println!("sample len {sample_len}"); // PJB
        for index in 0..sample_len {
            let context_size = if index > 0 {
                1
            } else {
                tokens.len()
            };
            //println!("context size {context_size}"); // PJB
            let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input)?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            //println!("{}", next_token); // PJB
            generated_tokens += 1;
            if next_token == eos_token || next_token == 198 { // 198 is \n\n
            //if next_token == eos_token { 
            //if next_token == 198 {
                println!("BREAK {}", next_token);
                break;
            }
            let token = self.tokenizer.decode(&[next_token], true).map_err(E::msg)?;
            //println!("{}", token); // PJB
            response += &token;
        }
        let dt = start_gen.elapsed();
        println!("{} tokens, {:.2} token/s", generated_tokens, generated_tokens as f64 / dt.as_secs_f64());
        Ok(response.trim().to_string())
    }
}


// Use the retrieved text as context.
pub fn generate_answer(query: &str, references: &Vec<String>) -> Result<String> {

    let mut context = Vec::new(); // :Vec<String>
    for reference in references {
        //println!("{:?}", reference.content_chunk);
        context.push(json!(
            {
                "context": reference,
                //"metadata": reference.metadata,
            }
        ))
    }

    let context = json!(context).to_string();

    let prompt = format!("<|im_start|>system\nYou are a friendly and helpful AI assistant. Your answer should be concise and to the point and use the references. Do not repeat the question or references. Today is {date}<|im_end|>\n<|im_start|>user\nquestion: \"{question}\"\nreferences: \"{context}\"\n<|im_end|>\n<|im_start|>assistant\n", context=context, question=query, date=chrono::Local::now().format("%A, %B %e, %Y"));

    let (model, tokenizer) = &*PHI;

    /*if args.verbose == true {
        println!("{}", prompt); // PJB
    }*/
    println!("{}", prompt); // PJB
    
    /*
        model: QMixFormer,
        tokenizer: Tokenizer,
        seed: u64,
        temp: Option<f64>,
        top_p: Option<f64>,
        repeat_penalty: f32,
        repeat_last_n: usize,
        device: &Device,
     */
    // See also https://www.shuttle.rs/blog/2024/05/01/using-huggingface-rust
    let mut pipeline = TextGeneration::new(
        model.clone(),
        tokenizer.clone(),
        28,
        Some(0.4), // temp, higher = more random
        Some(0.5), // top_p, cumulative probs, a higher value for top-p (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more focused and conservative text. The default value is 0.9.
        1.3, // repeat_penalty, control the repetition of token sequences in the generated text (default: 1.1).
        64,
        &device(false)?,
    );
    let response = pipeline.run(&prompt, 400)?; // 400...

    Ok(response)
}
