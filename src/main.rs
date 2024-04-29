use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    // With default InitOptions
    //let model = TextEmbedding::try_new(Default::default()).expect("Cannot initialise model.");

    dbg!(TextEmbedding::list_supported_models());
    
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

    /*
    let dimension = 384;

    let data: &str = "This is an example.";
    let vector = Vector::random(128);
    let vectors = model.embed(vec![data], None).expect("Cannot create embeddings.");

    for doc in documents {
        let v = Vector(vector);
        let record = Record::new(&v, &data.into());
    }
    for vector in vectors {
        let v = Vector(vector);
        let record = Record::new(&v, &data.into());
    }
    
    // Replace with your own data.
    let records = Record::many_random(dimension, 100);

    let mut config = Config::default();

    // Optionally set the distance function. Default to Euclidean.
    config.distance = Distance::Cosine;

    // Create a vector collection.
    let collection = Collection::build(&config, &records).unwrap();

    // Optionally save the collection to persist it.
    let mut db = Database::new("data/test").unwrap();
    db.save_collection("vectors", &collection).unwrap();

    // Search for the nearest neighbors.
    let query = Vector::random(dimension);
    let result = collection.search(&query, 5).unwrap();

    for res in result {
        let (id, distance) = (res.id, res.distance);
        println!("{distance:.5} | ID: {id}");
    }
*/
    
    Ok(())
}
