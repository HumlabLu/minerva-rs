use oasysdb::prelude::*;
use crate::Args;
use clap::Parser;


pub fn get_db() -> Database {
    let args = Args::parse();
    let mut db = Database::open("data/test").unwrap();
    db
}
/*
    // Replace with your own data.
    //let records = Record::many_random(dimension, 100);

    let config = Config::default();

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
    let collection = db.get_collection(&args.collection).unwrap();
*/
