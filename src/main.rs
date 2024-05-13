use oasysdb::prelude::*;
use clap::{Parser, Subcommand};
mod database;
use database::{get_db, data_to_record};
mod embedder;
use embedder::{chunk_string, embed_file_txt, embed_file_pdf, embeddings};
mod textgen;
use std::path::Path;
mod qmistral;
use qmistral::run_qmistral;
use std::collections::HashMap;

// =====================================================================
// Command line arguments.
// =====================================================================

#[derive(Parser, Debug, Clone)]
#[command(name = "Minerva")]
#[command(about = "Minerva is a RAG", long_about = None)]
struct Args {
    // Filename
    #[arg(short, long, help = "The file... but what is it?")]
    pub filename: Option<String>, // Path thingy?

    // Chunk size
    #[clap(long, action, default_value_t = 512, help = "Chunk size in characters.")]
    pub chunksize: usize,

    // Name of the database (collection)
    #[arg(long, default_value = "vectors", help = "Name of the database collection.")]
    pub collection: String,

    // The k-nearest neighbours.
    #[clap(short, long, action, default_value_t = 2, help = "The k-nearest neighbours.")]
    pub knearest: usize,

    // Query
    #[arg(short, long, help = "Question?")]
    pub query: Option<String>,

    // Extra output
    #[arg(long, short, action, help = "Produce superfluous output.")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    /// List collection.
    List {
    },

    Del {
    },
}

// =====================================================================
// Main.
// =====================================================================
// Function to convert Metadata of type Object back into a HashMap

fn md_to_hashmap(metadata: &Metadata) -> Option<HashMap<String, Metadata>> {
    match metadata {
        Metadata::Object(hm) => Some(hm.clone()),
        _ => None
    }
}

fn md_to_str(metadata: &Metadata) -> Option<String> {
    match metadata {
        Metadata::Text(txt) => Some(txt.to_string()),
        _ => None,
    }
}

fn main() -> anyhow::Result<()> {

    let args = Args::parse();
    println!("{:?}", &args);
        
    // This is the saved DB, containing different collections.
    let mut db = get_db();
    let mut collection = db.get_collection(&args.collection).unwrap_or_else(|_| {
        println!("Creating a new empty collection.");
        let config = Config::default();
        //Collection::build(&config, &records).unwrap()
        let c = Collection::new(&config);
        db.save_collection(&args.collection, &c).unwrap(); // Save it so it exists on disk.
        /*
        match db.save_collection(&args.collection, &c) {
            Ok(_) => c,
            Err(e) => {
                eprintln!("Failed to save the new collection: {}", e);
                panic!("Critical error: could not save collection");
            }
        }
        */
        c
    });
    
    if let Some(filename) = &args.filename {
        let path = Path::new(filename);
        let mut chunked_data: Option<Vec<String>> = None;
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "txt" {
                    chunked_data = Some(embed_file_txt(filename, args.chunksize).expect("File does not exist?"));
                } else if ext == "pdf" {
                    chunked_data = Some(embed_file_pdf(filename, args.chunksize).expect("File does not exist?"));
                }
            }
        }
        if let Some(data) = chunked_data {
            let vectors = embeddings(data.clone()).expect("Cannot create embeddings.");
            let mut records = vec![];
            for (chunk, vector) in data.iter().zip(vectors.iter()) {
                // With custom InitOptions
                let record = data_to_record(vector, filename, chunk);
                //println!("Record {:?}", record);
                records.push(record);
            }

            // Add it to the current collection.
            let ids = collection.insert_many(&records).unwrap();
            println!("Added {:?} items", ids.len());

            // And make it persistent.
            db.save_collection(&args.collection, &collection).unwrap();
        }
    }
    println!("Size of collection {}.", collection.len());
    
    // Shouldn't really mix --parameters and commands...
    match args.command {
        Some(Commands::List { }) => {
            let list = collection.list().unwrap();
            for (id, item) in list {
                println!("{:5} | {:?}", id.0, item.data); // data = Metadata
                let hm = md_to_hashmap(&item.data).unwrap();
                println!("{:?}/{:?}/{:?}", md_to_str(
                    hm.get("ulid").unwrap()
                ).unwrap(),
                md_to_str(
                    hm.get("date").unwrap()
                ).unwrap(),
                md_to_str(
                    hm.get("filename").unwrap()
                ).unwrap());
                println!("{:?}", md_to_str(
                    hm.get("text").unwrap()
                ).unwrap());
            }
        },
        Some(Commands::Del { }) => {
            let _ = db.delete_collection(&args.collection);
            println!("Deleted collection \"{}\"", &args.collection);
        },
        None => {}
    }
        
    // Search for the nearest neighbours.
    if let Some(query) = &args.query {
        println!("Asking {}", &query);
        
        let data = chunk_string(query, args.chunksize);
        //println!("{:?}", data); // Only if verbose!
        let vectors = embeddings(data).expect("Cannot create embeddings.");
        let v = vectors.get(0).expect("uh");
        let embedded_query = Vector((&v).to_vec());
        let result = collection.search(&embedded_query, args.knearest).unwrap();

        //let mut string_context = vec![];
        let mut context_str = String::new();
        if result.len() == 0 {
            context_str = "Use any knowledge you have.".to_string();
        }
        let mut sep = "";
        for res in result {
            let hm = md_to_hashmap(&res.data).unwrap();
            let filename = md_to_str(hm.get("filename").unwrap()).unwrap();
            let text = md_to_str(hm.get("text").unwrap()).unwrap();
            context_str += &(sep.to_owned() + "(document:" + &filename + ", with contents:" + &text + ")");
            sep = ", ";
        }
/*
        for res in result {
            //println!("{:?}", res);
            let md = match res.data {
                Metadata::Text(value) => value,
                _ => "Data is not text.".to_string()
            };
            //let (id, distance) = (res.id, res.distance);
            //println!("{distance:.5} | ID: {id} {md}"); // Use verbosity
            string_context.push(md.clone());
            context_str += &md;
        }*/

        /*
        let ts_start = chrono::Local::now();
        let ans = generate_answer(&query, &string_context);
        let ts_end = chrono::Local::now();
        println!("{:?}", ts_end - ts_start);
        println!("{:?}", ans.unwrap().trim().to_string());
         */
        // ---
        
        let ts_start = chrono::Local::now();
        let q = format!("You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not repeat the question or references. Today is {date}. Context: {context}. Question: {question}.", context=context_str, question=query, date=chrono::Local::now().format("%A, %B %e, %Y"));
        let ans = run_qmistral(&q);
        let ts_end = chrono::Local::now();
        println!("{:?}", ts_end - ts_start);
        println!("\n{}", ans.unwrap().trim().to_string());
    }

    Ok(())
}
