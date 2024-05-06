use oasysdb::prelude::*;
use clap::{Parser, Subcommand};
mod database;
use database::{get_db, data_to_record};
mod embedder;
use embedder::{chunk_string, embed_file_txt, embed_file_pdf, embeddings};
mod textgen;
use textgen::{generate_answer};
use std::path::Path;
mod mistral;
use mistral::test_mistral;

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

fn main() -> anyhow::Result<()> {

    let args = Args::parse();
    dbg!("{:?}", &args);

    test_mistral();
    
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
                let record = data_to_record(vector, chunk);
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
    
    // Shouldn't really mix --paraeters and commands...
    match args.command {
        Some(Commands::List { }) => {
            let list = collection.list().unwrap();
            for (id, item) in list {
                println!("{:5} | {:?}", id.0, item.data); // data = Metadata
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

        let mut string_context = vec![];
        for res in result {
            //println!("{:?}", res);
            let md = match res.data {
                Metadata::Text(value) => value,
                _ => "Data is not text.".to_string()
            };
            let (id, distance) = (res.id, res.distance);
            //println!("{distance:.5} | ID: {id} {md}"); // Use verbosity
            string_context.push(md.clone());
        }
        let ts_start = chrono::Local::now();
        let ans = generate_answer(&query, &string_context);
        let ts_end = chrono::Local::now();
        println!("{:?}", ts_end - ts_start);
        println!("{:?}", ans.unwrap().trim().to_string());
    }

    Ok(())
}
