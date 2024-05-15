use oasysdb::prelude::*;
use clap::{Parser, Subcommand};
mod database;
use database::{get_db, data_to_record};
mod embedder;
use embedder::{chunk_string, embed_file_txt, embed_file_pdf, embeddings, read_dir_contents};
mod textgen;
//use textgen::{load_model, generate_answer};
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

    #[arg(short, long, help = "Directory with text files.")]
    pub dirname: Option<String>,

    // The k-nearest neighbours.
    #[clap(short, long, action, default_value_t = 3, help = "The k-nearest neighbours.")]
    pub knearest: usize,

    // Query
    #[arg(short, long, help = "Question?")]
    pub query: Option<String>,

    // Extra output
    #[arg(long, short, action, help = "Produce superfluous output.")]
    pub verbose: bool,

    #[arg(long, action, help = "Show prompt.")]
    pub showprompt: bool,
    
    #[arg(long, action, help = "Show context.")]
    pub showcontext: bool,

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
        Metadata::Integer(i) => Some(i.to_string()),
        _ => None,
    }
}

fn main() -> anyhow::Result<()> {

    let args = Args::parse();
    println!("{:?}", &args);

    // _ = load_model();
    
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

    if let Some(dirname) = &args.dirname {
        let mut records = vec![];
        let filenames = read_dir_contents(dirname).unwrap();
        for filename in filenames {
            let filename_str = filename.clone().into_os_string().into_string().unwrap();
            print!("Reading {}", filename_str);

            // Should check extension...
            let chunked_data = Some(embed_file_txt(filename, args.chunksize).expect("File does not exist?"));
            
            if let Some(data) = chunked_data {
                let vectors = embeddings(data.clone()).expect("Cannot create embeddings.");
                let mut chunk_counter = 0usize;
                for (chunk, vector) in data.iter().zip(vectors.iter()) {
                let record = data_to_record(vector, &filename_str, chunk, chunk_counter);
                    records.push(record);
                    chunk_counter += 1;
                }
                println!(", Items {}", data.len());
            }
        }
        if records.len() > 0 {
            let ids = collection.insert_many(&records).unwrap();
            println!("Added {:?} items", ids.len());
        } else {
            println!("No items to add");
        }
        
        // And make it persistent.
        db.save_collection(&args.collection, &collection).unwrap();
    }
    
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
            let mut chunk_counter = 0usize;
            for (chunk, vector) in data.iter().zip(vectors.iter()) {
                // With custom InitOptions
                let record = data_to_record(vector, filename, chunk, chunk_counter);
                //println!("Record {:?}", record);
                records.push(record);
                chunk_counter += 1;
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
            for (id, item) in list.iter() {
                //println!("{:5} | {:?}", id.0, item.data); // data = Metadata
                let hm = md_to_hashmap(&item.data).unwrap();
                println!("{:5}/{:?}/{:?}/{:?}/{:?}",
                    id.0,
                    md_to_str(hm.get("ulid").unwrap()).unwrap(),
                    md_to_str(hm.get("date").unwrap()).unwrap(),
                    md_to_str(hm.get("filename").unwrap()).unwrap(),
                    md_to_str(hm.get("ccnt").unwrap()).unwrap()
                );
                println!("{:?}\n", md_to_str(
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

        let mut context_str = String::new();
        if result.len() == 0 {
            context_str = "Use any knowledge you have.".to_string();
        }
        let mut sep = "";
        for res in result {
            let hm = md_to_hashmap(&res.data).unwrap();
            let filename = md_to_str(hm.get("filename").unwrap()).unwrap();
            let chunk_nr = md_to_str(hm.get("ccnt").unwrap()).unwrap();
            let text = md_to_str(hm.get("text").unwrap()).unwrap();

            let dist = res.distance;
            println!("{dist:.4} | {filename}/{chunk_nr}");
            if args.showcontext == true {
                println!("  {}\n", text);
            }

            context_str += &(sep.to_owned() + "\n(document:\"" + &filename + "/" + &chunk_nr + "\", with contents:" + &text + ")");
            sep = ", ";
        }

        // ---

        // Try textgen as well.
        /*
        ERROR: Unable to load model: request error: https://huggingface.co/AI-Sweden-Models/gpt-sw3-6.7b-v2-instruct-gguf/resolve/main/tokenizer.json: status code 404

        Caused by:
        https://huggingface.co/AI-Sweden-Models/gpt-sw3-6.7b-v2-instruct-gguf/resolve/main/tokenizer.json: status code 404
        */
        /*
        let mut string_context = vec![];
        let ans = generate_answer(&query, &string_context);
        println!("{:?}", ans.unwrap().trim().to_string());
        */
        
        // ---
        
        let _ts_start = chrono::Local::now();
        let q = format!("You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Print the name of document used from the context. Do not repeat the question or references. Today is {date}. Context: {context} \nQuestion: {question}", context=context_str, question=query, date=chrono::Local::now().format("%A, %B %e, %Y"));
        if args.showprompt == true {
            println!("\n{}\n", q);
        }
        //let q = format!("{question}", question=query);
        //let q = format!("Du är en vänlig och hjälpsam AI-assistent. Ditt svar ska vara kortfattat och använda sammanhanget om möjligt. Skriv ut namnet på det dokument som används från sammanhanget. Upprepa inte frågan eller referenserna. Svara på Svenska! Idag är det {date}. Sammanhang: {context}. Fråga: {question}.", context=context_str, question=query, date=chrono::Local::now().format("%A, %B %e, %Y"));
        let ans = run_qmistral(&q);
        let _ts_end = chrono::Local::now();
        //println!("{:?}", ts_end - ts_start);
        println!("\n{}", ans.unwrap().trim().to_string());
    }

    Ok(())
}
