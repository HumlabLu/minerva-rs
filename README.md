# Minerva

## Notes

 - Needs a HF token!

## Ubuntu

To compile on Ubuntu, we need to set the path to CUDA libs and nvcc (this works for on Ubuntu 22.04):
```
export PATH=$PATH:/usr/lib/gcc/x86_64-linux-gnu/11/
export PATH=$PATH:/usr/local/cuda-12.2/bin/
```
 
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
