use oasysdb::prelude::*;
use crate::Args;
use clap::Parser;

// With custom InitOptions


/*
In short, use Collection to store your vector records or search
similar vectors and use Database to persist a vector collection to the
disk.
*/

pub fn get_db() -> Database {
    let args = Args::parse(); // Should not be here, have function args instead.
    let mut db = Database::open("data/test").unwrap();
    // let collection = db.get_collection("vectors").unwrap();
    println!("DB contains {} collections.", db.len());
    db
}

pub fn save_db(db: &mut Database) {
    let collection = db.get_collection("vectors").unwrap();
    db.save_collection("vectors", &collection).unwrap();
}

// We need a save, load, new, ...

/*
    // Replace with your own data.
    //let records = Record::many_random(dimension, 100);

    let config = Config::default();

    // Optionally set the distance function. Default to Euclidean.
    //config.distance = Distance::Cosine;

    // Create a vector collection.
    //let collection = Collection::build(&config, &records).unwrap();
    
    let mut db = Database::open("data/test").unwrap();

    //let collection = Collection::build(&config, &records).unwrap();
    //db.save_collection("vectors", &collection).unwrap();
    let collection = db.get_collection(&args.collection).unwrap();
*/
