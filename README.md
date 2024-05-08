# Minerva

## Notes

 - Needs a HF token!
 
## A text about water (from wikipedia).

```shell
cargo run --release -- -f texts/water.txt
```

## Ask a question.

```shell
cargo run -q --release -- -q "Write one sentence about water"

[src/main.rs:68] &args = Args {
    filename: None,
    chunksize: 512,
    collection: "vectors",
    knearest: 2,
    query: Some(
        "Write  one sentence about water",
    ),
    verbose: false,
    command: None,
}
DB contains 1 collections.
Size of collection 8.
Asking Write  one sentence about water
Device Cpu
loaded 291 tensors (4.37GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096
"/Users/pberck/.cache/huggingface/hub/models--mistralai--Mistral-7B-v0.1/snapshots/26bca36bde8333b5d7f72e9ed20ccda6a618af24/tokenizer.json"
[INST] You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not repeat the question or references. Today is Wednesday, May  8, 2024. Context: Water is a fundamental substance with unique properties that have profound implications for life on Earth, climate systems, and human society. Here are ten facts about water that highlight its importance and uniqueness:These facts underscore water's critical role in sustaining life, shaping climates, and influencing human societies. The study of water and its management is central to environmental science, ecology, and global sustainability efforts.. Question: Write  one sentence about water. [/INST]


 149 prompt tokens processed: 16.83 token/s
  22 tokens generated: 10.47 token/s
TimeDelta { secs: 12, nanos: 332276000 }
"Water is a fundamental and unique substance that sustains life, shapes climates, and influences human societies."
```

## Ask another question.

```shell
cargo run -q --release -- -q "Write two sentences about water"

[src/main.rs:68] &args = Args {
    filename: None,
    chunksize: 512,
    collection: "vectors",
    knearest: 2,
    query: Some(
        "Write two sentences about water",
    ),
    verbose: false,
    command: None,
}
DB contains 1 collections.
Size of collection 8.
Asking Write two sentences about water
Device Cpu
loaded 291 tensors (4.37GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096
"/Users/pberck/.cache/huggingface/hub/models--mistralai--Mistral-7B-v0.1/snapshots/26bca36bde8333b5d7f72e9ed20ccda6a618af24/tokenizer.json"
[INST] You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not repeat the question or references. Today is Wednesday, May  8, 2024. Context: Water is a fundamental substance with unique properties that have profound implications for life on Earth, climate systems, and human society. Here are ten facts about water that highlight its importance and uniqueness:These facts underscore water's critical role in sustaining life, shaping climates, and influencing human societies. The study of water and its management is central to environmental science, ecology, and global sustainability efforts.. Question: Write two sentences about water. [/INST]


 148 prompt tokens processed: 16.71 token/s
  60 tokens generated: 9.77 token/s
TimeDelta { secs: 16, nanos: 382399000 }
"Water is a fundamental substance on Earth that has unique properties, and its critical role in sustaining life, shaping climates, and influencing human societies cannot be overstated. The study of water and its management is central to environmental science, ecology, and global sustainability efforts."
```
