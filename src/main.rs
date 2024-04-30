use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use oasysdb::prelude::*;

fn main() -> anyhow::Result<()> {
    // With default InitOptions
    //let model = TextEmbedding::try_new(Default::default()).expect("Cannot initialise model.");

    //dbg!(TextEmbedding::list_supported_models());
    
    // With custom InitOptions
    let model = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::AllMiniLML6V2,
        show_download_progress: true,
        ..Default::default()
    }).expect("Cannot Initialise model.");

    let documents = vec![
        "passage: Hello, World!",
        "query: Hello, World!",
        "passage: This is an example passage.",
        // You can leave out the prefix but it's recommended
        "fastembed-rs is licensed under Apache  2.0"
    ];
    
    // Generate embeddings with the default batch size, 256
    let embeddings = model.embed(documents, None).expect("Cannot create embeddings.");
    //println!("{:?}", embeddings);
    
    println!("Embeddings length: {}", embeddings.len()); // -> Embeddings length: 4
    println!("Embedding dimension: {}", embeddings[0].len()); // -> Embedding dimension: 384

    // ----

    // Vector dimension must be uniform.
    let dimension = 384;

    // Replace with your own data.
    //let records = Record::many_random(dimension, 100);

    let mut config = Config::default();

    // Optionally set the distance function. Default to Euclidean.
    //config.distance = Distance::Cosine;

    // Create a vector collection.
    //let collection = Collection::build(&config, &records).unwrap();
    
    let mut db = Database::open("data/test").unwrap();

    let data = vec!["This is an example.", "Hello world!", "Another example"];
    let vectors = model.embed(data.clone(), None).expect("Cannot create embeddings.");
    let mut records = vec![];
    for (chunk, vector) in data.iter().zip(vectors.iter()) {
        let v = Vector((&vector).to_vec());
        let m = Metadata::Text((&chunk).to_string());
        let record = Record::new(&v, &m);
        println!("Record {:?}", m);
        records.push(record);
    }

    //let collection = Collection::build(&config, &records).unwrap();
    //db.save_collection("vectors", &collection).unwrap();
    let collection = db.get_collection("vectors").unwrap();
    
    // Search for the nearest neighbors.
    let data = vec!["This is another example"];
    let vectors = model.embed(data, None).expect("Cannot create embeddings.");
    let v = vectors.get(0).expect("uh");
    let query = Vector((&v).to_vec());
    let result = collection.search(&query, 2).unwrap();

    for res in result {
        //println!("{:?}", res);
        let md = match res.data {
            Metadata::Text(value) => value,
            _ => panic!("Data is not a text."),
        };
        let (id, distance) = (res.id, res.distance);
        println!("{distance:.5} | ID: {id} {md}");
    }
    
    Ok(())
}

// FEST 7e maj!
