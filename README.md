# Minerva

## Intro

Simple RAG system written in Rust. It uses `fastembed-rs` to create
embeddings for the vector database. The vector database is implemented
using `oasysdb`. Texts are split into chunks using `text-splitter`. The
Candle library is used to run the models (which can be downloaded from
huggingface). Alternatively, Ollama can be used to generate answers.

The system runs locally, no data is uploaded or stored online.

There is a second database, implemented using `tantivy`, which can store
plain text documents which can be retrieved with a keyword
search. This is still work in progress.

## Notes

 - Depending on the model, might need a HF token!
 - Code needs cleaning/improving.
 - Tantivy DB integration needs work (is slow).

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
pberck@Peters-MacBook-Pro-2 minerva-rs % cargo run --release -- -q "How many cats does Peter have?" -m 0.75
    Finished `release` profile [optimized] target(s) in 0.15s
     Running `target/release/minerva-rs -q 'How many cats does Peter have?' -m 0.75`
Args { filename: None, chunksize: 1024, collection: "vectors", dirname: None, tantdirname: None, maxdist: 0.75, nearest: 3, query: Some("How many cats does Peter have?"), keyword: None, verbose: false, showprompt: false, showcontext: false, command: None }
Embedding dim 384
Number of documents in the index: 0
DB contains 1 collections.
Size of collection 5.

Asking How many cats does Peter have?
0.7070 | texts/facts.txt/1 *
0.7257 | texts/facts.txt/2 *
0.7490 | texts/facts.txt/0 *
Model TheBloke/Mistral-7B-Instruct-v0.2-GGUF | mistral-7b-instruct-v0.2.Q5_K_M.gguf
Device Cpu
loaded 291 tensors (5.13GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096
Prompt length 177, pre-processing...

According to the facts given in the "texts/facts.txt" document, Peter and Elisabet
have a cat called Sirius (document: "texts/facts.txt/0") and another cat named Maja
(document: "texts/facts.txt/1"). Therefore, Peter has a total of two cats.
```

### Example 2 -- Larger Chunks

```shell
pberck@Peters-MacBook-Pro-2 minerva-rs % cargo run --release -- -f texts/facts.txt
    Finished `release` profile [optimized] target(s) in 0.16s
     Running `target/release/minerva-rs -f texts/facts.txt`
Args { filename: Some("texts/facts.txt"), chunksize: 1024, collection: "vectors", dirname: None, tantdirname: None, maxdist: 0.65, nearest: 3, query: None, keyword: None, verbose: false, showprompt: false, showcontext: false, command: None }
Embedding dim 384
Number of documents in the index: 0
DB contains 1 collections.
Chunking text file.
Creating embeddings.
Storing embeddings.
Added 1 items
Size of collection 1.

pberck@Peters-MacBook-Pro-2 minerva-rs % cargo run --release -- -q "Where is Peter's cat?"
    Finished `release` profile [optimized] target(s) in 0.15s
     Running `target/release/minerva-rs -q 'Where is Peter'\''s cat?'`
Args { filename: None, chunksize: 1024, collection: "vectors", dirname: None, tantdirname: None, maxdist: 0.65, nearest: 3, query: Some("Where is Peter's cat?"), keyword: None, verbose: false, showprompt: false, showcontext: false, command: None }
Embedding dim 384
Number of documents in the index: 0
DB contains 1 collections.
Size of collection 1.

Asking Where is Peter's cat?
0.6011 | texts/facts.txt/0 *
Model TheBloke/Mistral-7B-Instruct-v0.2-GGUF | mistral-7b-instruct-v0.2.Q5_K_M.gguf
Device Cpu
loaded 291 tensors (5.13GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096
Prompt length 169, pre-processing...

Peter has two cats named Sirius and Maja. According to the context from the
document "facts.txt/0", they live in Rörums Holma, which is located in Skåne,
Sweden. So, Peter's cats are in Rörums Holma, Skåne, Sweden.

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
pberck@Peters-MacBook-Pro-2 minerva-rs % cargo run --release -- -q "Where is Sirius?" -n0
[INST] You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not repeat the question or references. Today is Thursday, May  9, 2024. Context: Use any knowledge you have.. Question: Where is Sirius?. [/INST]

  68 prompt tokens processed: 13.20 token/s
 129 tokens generated: 9.18 token/s
TimeDelta { secs: 20, nanos: 721854000 }

Sirius is the brightest star in the night sky, and it's part of the constellation Canis Major (The Greater Dog). You can locate it by finding the constellation Orion (identified by its distinctive "Square of Orion" asterism), then drawing a line from Betelgeuse, one of Orion's shoulders, to Bellatrix, another star in Orion. Sirius is located roughly where that line intersects with the Milky Way. Keep in mind that visibility of stars depends on various factors such as location, weather conditions, and time of year.
```

## Workflow

### Add a text.

```shell
cargo run --release -- -f texts/water.txt

Embedding dim 384
Number of documents in the tantivy database: 0
DB contains 1 collections.
Chunking text file.
Creating embeddings.
Storing embeddings.
Added 4 items
Size of vector database 5.
```

### Ask a question.

```shell
cargo run -q --release -- -q "Write one sentence about water"

Embedding dim 384
Number of documents in the tantivy database: 0
DB contains 1 collections.
Size of vector database 5.

Asking Write one sentence about water
0.5036 | texts/water.txt/2 *
0.5204 | texts/water.txt/0 *
0.5407 | texts/water.txt/3 *
Model TheBloke/Mistral-7B-Instruct-v0.2-GGUF | mistral-7b-instruct-v0.2.Q5_K_M.gguf
Device Cpu
loaded 291 tensors (5.13GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096
Prompt length 656, pre-processing...

Water, a universal solvent with a unique polar structure, is essential for all
known forms of life and plays a crucial role in the functioning of ecosystems,
climate systems, and human societies.

(No specific document used as the context provided multiple documents
discussing water and its importance.)
```

### Ask another question.

```shell
cargo run -q --release -- -q "Write two sentences about water"

Embedding dim 384
Number of documents in the tantivy database: 0
DB contains 1 collections.
Size of vector database 5.

Asking "Write two sentences about water"
0.5268 | texts/water.txt/2 *
0.5294 | texts/water.txt/0 *
0.5567 | texts/water.txt/3 *
Model TheBloke/Mistral-7B-Instruct-v0.2-GGUF | mistral-7b-instruct-v0.2.Q5_K_M.gguf
Device Cpu
loaded 291 tensors (5.13GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096
Prompt length 656, pre-processing...

Water is a vital substance for all known forms of life, acting as a solvent for biological
reactions and playing a crucial role in the regulation of temperature. Its unique chemical
structure, including its polarity, enables it to dissolve various substances and make up
approximately 71% of the Earth's surface.
(References: documents "keywords" - sections 8 and 1)

Water also significantly impacts climate systems as a greenhouse gas through water vapor
and influences weather patterns by distributing heat and shaping global air currents.
(References: document "keywords" - section 3)
```

### Ollama

By specifying the `-o` parameter, Ollama (mistral) will be used to generate answers. This expects Ollama to be installed and the mistral model to have been downloaded.

## List database contents.

```shell
cargo run --release -- list

Embedding dim 384
Number of documents in the tantivy database: 0
DB contains 1 collections.
Size of vector database 5.
    4/"01J2B1FRMNARAGT84H6NRAQNSK"/"20240709T0804"/"texts/water.txt"/"3"
"10. **Impact on Climate**: Water vapour is a significant greenhouse gas, ..."

    2/"01J2B1FRMNFXEGHRT6WZH5PXBS"/"20240709T0804"/"texts/water.txt"/"1"
"3. **Density and Ice Formation**: Uniquely, water expands and becomes less ..."

    0/"01J29B9S4B8Z4GFBYYSG1GDZ10"/"20240708T1617"/"texts/facts.txt"/"0"
"We have a cat called Sirius. We have another cat called Maja. We refers to ..."

    3/"01J2B1FRMN9AXZVTPFSHQ89NBH"/"20240709T0804"/"texts/water.txt"/"2"
"6. **The Water Cycle**: The water cycle describes the continuous movement of ..."

    1/"01J2B1FRMNKVZKE3K9HTP3YJ89"/"20240709T0804"/"texts/water.txt"/"0"
"Water is a fundamental substance with unique properties that have profound ..."
```

## Minerva

```shell
cargo run -q --release -- -q "Who was Minerva? Reference Roman mythology." -n0
Args { filename: None, chunksize: 1024, collection: "vectors", dirname: None, tantdirname: None, maxdist: 0.65, nearest: 0, query: Some("Who was Minerva? Reference Roman mythology."), keyword: None, verbose: false, showprompt: false, showcontext: false, command: None }
Embedding dim 384
Number of documents in the index: 0
DB contains 1 collections.
Size of collection 1.

Asking Who was Minerva? Reference Roman mythology.
All results have been filtered :-(
Model TheBloke/Mistral-7B-Instruct-v0.2-GGUF | mistral-7b-instruct-v0.2.Q5_K_M.gguf
Device Cpu
loaded 291 tensors (5.13GB) in 0.06s
model built
model::MAX_SEQ_LEN 4096
Prompt length 99, pre-processing...

Minerva was the Roman goddess of wisdom, strategic warfare, and crafts. She is
equivalent to the Greek goddess Athena. According to Roman mythology, she was
born fully grown from the forehead of Jupiter (the Roman equivalent of Zeus)
when he had a thunderbolt headache caused by the god Neptune striking the ground
with his trident in anger. Minerva is often depicted as wearing a helmet adorned
with olive leaves and carrying a spear, a shield, or an owl.
(Reference: Encyclopedia Britannica)
```
