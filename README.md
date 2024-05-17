# Minerva

## Notes

 - Needs a HF token!
 - Code needs cleaning/improving.

## Ubuntu

To compile on Ubuntu, we need to set the path to CUDA libs and nvcc (this works for on Ubuntu 22.04):
```
export PATH=$PATH:/usr/lib/gcc/x86_64-linux-gnu/11/
export PATH=$PATH:/usr/local/cuda-12.2/bin/
```
 
## Ask a question.

### Example 1

```shell
% cat texts/facts.txt
We have a cat called Sirius. We have another cat called Maja. We refers to Peter and Elisabet. They live in Rörums Holma. Rörumns Holma is in Skåne. Skåne is in Sweden.

% cargo run --release -- --chunksize 32 -f texts/facts.txt

% cargo run --release -- -q "How many cats does Peter have?"
Args { filename: None, chunksize: 512, collection: "vectors", dirname: None, knearest: 3, query: Some("How many cats does Peter have?"), verbose: false, command: None }
DB contains 1 collections.
Size of collection 5.
Asking How many cats does Peter have?
Model TheBloke/Mistral-7B-Instruct-v0.2-GGUF | mistral-7b-instruct-v0.2.Q5_K_M.gguf
Device Cpu
loaded 291 tensors (5.13GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096
[INST] You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Print the name of document used from the context. Do not repeat the question or references. Today is Wednesday, May 15, 2024. Context:
(document:"texts/facts.txt/1", with contents:We have another cat called Maja.),
(document:"texts/facts.txt/2", with contents:We refers to Peter and Elisabet.),
(document:"texts/facts.txt/0", with contents:We have a cat called Sirius.).
Question: How many cats does Peter have?. [/INST]

Based on the context provided, Peter and Elisabet have a cat named Sirius (from facts.txt/0),
and they also have another cat named Maja (from facts.txt/1). Therefore,
Peter has a total of two cats.
```

### Example 2 -- Larger Chunks

```shell
cargo run --release -- -f facts.txt
Args { filename: Some("facts.txt"), chunksize: 512, collection: "vectors", dirname: None, knearest: 3, query: None, verbose: false, showprompt: false, showcontext: false, command: None }
Embedding dim 384
DB contains 1 collections.
Added 1 items
Size of collection 1.

cargo run --release -- -q "How many cats does Peter have?"
Embedding dim 384
DB contains 1 collections.
Size of collection 1.
Asking How many cats does Peter have?
0.6091 | facts.txt/0
Model TheBloke/Mistral-7B-Instruct-v0.2-GGUF | mistral-7b-instruct-v0.2.Q5_K_M.gguf
Device Cuda(CudaDevice(DeviceId(1)))
loaded 291 tensors (5.13GB) in 0.30s
model built
model::MAX_SEQ_LEN 4096
Prompt length 156, pre-processing...

> Peter has two cats, Sirius and Maja. (Referenced document: facts.txt/0)

cargo run --release -- -q "Where is Peter's cat?"
> Peter has two cats named Sirius and Maja. Based on the context,
> they live in Rörums Holma, which is located in Skåne, Sweden.
> So, Peter's cats are in Sweden at Rörums Holma. (facts.txt/0)

cargo run --release -- -q "Is Peter's cat called Nisse?"
> No, according to the given document "facts.txt/0",
> Peter's cats are named Sirius and Maja.

cargo run --release -- -q "Peter's cats are called Sirius and Maja."
> Sirius and Maja are the names of Peter and Elisabet's cats. (facts.txt/0)

cargo run --release -- -q "Peter's cats are called Sirius and Nisse."
> I'm sorry for the misunderstanding, but according to the
> provided context from the document "facts.txt/0",
> Peter's cats are named Sirius and Maja, not Sirius and Nisse.
```

### Example 3

```shell
pberck@Peters-MacBook-Pro-2 minerva-rs % cargo run --release -- -q "Where is Sirius?"
Args { filename: None, chunksize: 512, collection: "vectors", knearest: 2, query: Some("Where is Sirius?"), verbose: false, command: None }
DB contains 1 collections.
Size of collection 1.
Asking Where is Sirius?
Model file mistral-7b-instruct-v0.2.Q5_K_M.gguf
Device Cpu
loaded 291 tensors (5.13GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096

[INST] You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not repeat the question or references. Today is Thursday, May  9, 2024. Context: We have a cat called Sirius. We have another cat called Maja. We refers to Peter and Elisabet. They live in Rörums Holma. Rörumns Holma is in Skåne. Skåne is in Sweden.. Question: Where is Sirius?. [/INST]

 115 prompt tokens processed: 13.42 token/s
  17 tokens generated: 9.52 token/s
TimeDelta { secs: 11, nanos: 935309000 }

Sirius is at Rörums Holma in Skåne, Sweden.
```

Without the context.
```shell
pberck@Peters-MacBook-Pro-2 minerva-rs % cargo run --release -- -q "Where is Sirius?" -k0
[INST] You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not repeat the question or references. Today is Thursday, May  9, 2024. Context: Use any knowledge you have.. Question: Where is Sirius?. [/INST]

  68 prompt tokens processed: 13.20 token/s
 129 tokens generated: 9.18 token/s
TimeDelta { secs: 20, nanos: 721854000 }

Sirius is the brightest star in the night sky, and it's part of the constellation Canis Major (The Greater Dog). You can locate it by finding the constellation Orion (identified by its distinctive "Square of Orion" asterism), then drawing a line from Betelgeuse, one of Orion's shoulders, to Bellatrix, another star in Orion. Sirius is located roughly where that line intersects with the Milky Way. Keep in mind that visibility of stars depends on various factors such as location, weather conditions, and time of year.
```

## Add a text.

```shell
cargo run --release -- -f texts/water.txt
```


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

## List database contents.

```shell
cargo run --release -- list

[src/main.rs:68] &args = Args {
    filename: None,
    chunksize: 512,
    collection: "vectors",
    knearest: 2,
    query: None,
    verbose: false,
    command: Some(
        List,
    ),
}
DB contains 1 collections.
Size of collection 8.
    0 | Text("Water is a fundamental substance with unique properties that have profound implications for life on Earth, climate systems, and human society. Here are ten facts about water that highlight its importance and uniqueness:")
    4 | Text("6. **The Water Cycle**: The water cycle describes the continuous movement of water on, above, and below the surface of the Earth, involving processes such as evaporation, condensation, precipitation, and runoff. This cycle is essential for distributing heat and sustaining ecosystems.")
    3 | Text("4. **High Specific Heat Capacity**: Water has a high specific heat capacity, meaning it requires a lot of energy to change its temperature. This property helps regulate Earth's climate and the body temperatures of living organisms.\n\n5. **Cohesion and Surface Tension**: Water molecules are attracted to each other (cohesion), leading to a high surface tension. This allows insects to walk on water and plants to transport water from their roots to their leaves.")
    5 | Text("7. **Distribution on Earth**: About 71% of the Earth's surface is covered with water, but 97.5% of that is saltwater, leaving only 2.5% as freshwater. Of this freshwater, the majority is locked in ice caps and glaciers, making less than 1% readily accessible for human use.\n\n8. **Critical for Life**: Water is essential for all known forms of life. It serves as a solvent for biological reactions, a medium for transporting nutrients and waste, and plays a key role in regulating temperature.")
    7 | Text("These facts underscore water's critical role in sustaining life, shaping climates, and influencing human societies. The study of water and its management is central to environmental science, ecology, and global sustainability efforts.")
    6 | Text("9. **Human Use and Access**: While water is abundant on Earth, access to clean, fresh water is not evenly distributed. Many regions face water scarcity, affecting billions of people and leading to health, agricultural, and economic issues.\n\n10. **Impact on Climate**: Water vapour is a significant greenhouse gas, contributing to the greenhouse effect. The distribution and temperature of water bodies also influence weather patterns and climate systems globally.")
    2 | Text("2. **Universal Solvent**: Due to its polarity, water is known as the \"universal solvent\" because it can dissolve more substances than any other liquid. This property is crucial for the biochemical processes of living organisms.\n\n3. **Density and Ice Formation**: Uniquely, water expands and becomes less dense as it freezes. Ice has a lower density than liquid water, which is why ice floats. This anomaly is vital for aquatic life in cold climates, as the ice layer insulates the water below and maintains a habitable environment.")
    1 | Text("1. **Chemical Structure and Polarity**: Water (H2O) has a simple molecular structure consisting of two hydrogen atoms bonded to one oxygen atom. This arrangement gives water a polar nature, with a partial positive charge near the hydrogen atoms and a partial negative charge near the oxygen, enabling it to dissolve many substances.")
```

## Minerva

```shell
cargo run -q --release -- -q "Who was Minerva? Reference Roman mythology." -k0

  74 prompt tokens processed: 16.50 token/s
  72 tokens generated: 10.87 token/s
TimeDelta { secs: 12, nanos: 473691000 }
"Minerva was the goddess of wisdom, warfare, strategy, and crafts in Roman mythology. She is often portrayed as a helmeted woman holding a spear and shield in one hand and a spindle in the other. She was also known for her ability to grant wisdom and strategic planning to those who needed it most."
```
