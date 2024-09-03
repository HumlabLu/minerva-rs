use oasysdb::prelude::*;
use clap::{Parser, Subcommand};
mod database;
use database::{get_db, data_to_record};
mod embedder;
use embedder::{chunk_string, embed_file_txt, embed_file_pdf, embeddings, read_dir_contents, get_embedding_dim};
mod textgen;
//use textgen::{load_model, generate_answer};
use std::path::Path;
mod qmistral;
use qmistral::run_qmistral;
use std::collections::HashMap;
mod tant;
use tant::{search_documents, insert_file, get_index_schema, get_num_documents};
use tantivy::schema::OwnedValue;
mod genaigen;
use genaigen::genai_generate;
mod ollamagen;

// =====================================================================
// Store multiple sizes, eg 256 nd 1024. Then search on the 256,
// but return the longer 1024, so we get "more context", but also
// more specific searching. (A poor man's version of returning the
// chunks that come before and after the found chunk. (Or store
// plus/minus chunks as well when creating chunks, there we have
// all the info...
// =====================================================================

// =====================================================================
// Command line arguments.
// =====================================================================

#[derive(Parser, Debug, Clone)]
#[command(name = "Minerva")]
#[command(about = "Minerva is a RAG", long_about = None)]
struct Args {
    // Filename
    #[arg(short, long, help = "The file to add to the vector database.")]
    pub filename: Option<String>, // Path thingy?

    // Chunk size
    #[clap(long, action, default_value_t = 1024, help = "Chunk size (characters) for vectors.")]
    pub chunksize: usize,

    // Name of the database (collection)
    #[arg(long, default_value = "vectors", help = "Name of the database collection.")]
    pub collection: String,

    #[arg(short, long, help = "Directory with text files to add to the vector database.")]
    pub dirname: Option<String>,

    #[arg(short, long, help = "Directory with text files to add to the tantivy database.")]
    pub tantdirname: Option<String>,

    #[arg(short, long, help = "Maximum distance between vectors.", default_value_t = 0.6500)]
    pub maxdist: f32,

    // The k-nearest neighbours.
    #[clap(short, long, action, default_value_t = 3, help = "The k-nearest neighbours when retreiving vectors.")]
    pub nearest: usize,

    // Query
    #[arg(short, long, help = "The question to answer by the system.")]
    pub query: Option<String>,

    // Keyword
    #[arg(short, long, help = "Keyword to search for in the tantivy database.")]
    pub keyword: Option<String>,

    #[arg(long, short, action, help = "Use Ollama for generation.")]
    pub ollama: bool,

    #[arg(long, short = 'O', default_value = "mistral", help = "Ollama model to use.")]
    pub ollama_model: String,

    // Extra output
    #[arg(long, short, action, help = "Produce superfluous output.")]
    pub verbose: bool,

    #[arg(short = 'p', long, action, help = "Show the prompt.")]
    pub showprompt: bool,
    
    #[arg(short = 'c', long, action, help = "Show the context.")]
    pub showcontext: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    /// List collection.
    List {
    },

    /// Deletes the vector database.
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
    if args.verbose {
        println!("{:?}", &args);
    }
    println!("Embedding dim {}", get_embedding_dim().unwrap());

    // test
    //genai_generate("Why is the sky blue?");
    
    //_ = tanttest();
    let (i, _s) = get_index_schema().unwrap();
    let num_docs = get_num_documents(&i)?;
    println!("Number of documents in the tantivy database: {}", num_docs);

    /*
    let x = fuzzy_search_documents("light").unwrap();
    for (s, _d) in x {
        println!("{}", s);
    }
    */
    // _ = load_model();
    
    // This is the saved DB, containing different collections.
    let mut db = get_db();
    let mut collection = db.get_collection(&args.collection).unwrap_or_else(|_| {
        println!("Creating a new empty collection.");
        let  config = Config::default();
        //config.distance = Distance::Cosine;
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

    // This code can be simplified by putting a single file in a vector and
    // also treating it as "-d".
    // Separate functions for the tantivy database?
    if let Some(dirname) = &args.dirname {
        let mut records = vec![];
        let filenames = read_dir_contents(dirname).unwrap();
        for filename in filenames {
            let filename_str = filename.clone().into_os_string().into_string().unwrap();
            print!("Reading {}", filename_str); // Check extension here maybe...

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

    if let Some(dirname) = &args.tantdirname {
        let filenames = read_dir_contents(dirname).unwrap();
        let (index, _schema) = get_index_schema().unwrap();
        for filename in filenames {
            let filename_str = filename.clone().into_os_string().into_string().unwrap();
            print!("Reading {}...", filename_str); // Check extension here maybe...
            let num = insert_file(&index, &filename).unwrap();
            println!("added {}.", num);
        }
    }
            
    if let Some(filename) = &args.filename {
        let path = Path::new(filename);
        let mut chunked_data: Option<Vec<String>> = None;
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "txt" {
                    println!("Chunking text file.");
                    chunked_data = Some(embed_file_txt(filename, args.chunksize).expect("File does not exist?"));
                } else if ext == "pdf" {
                    println!("Chunking PDF file.");
                    chunked_data = Some(embed_file_pdf(filename, args.chunksize).expect("File does not exist?"));
                }
            }
        }
        if let Some(data) = chunked_data {
            println!("Creating embeddings.");
            let vectors = embeddings(data.clone()).expect("Cannot create embeddings.");
            let mut records = vec![];
            let mut chunk_counter = 0usize;
            println!("Storing embeddings.");
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
    println!("Size of vector database {}.", collection.len());
    
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

    // This searches in the tantivy database.
    let mut keyword_context = String::new();
    if let Some(keyword) = &args.keyword {
        println!("Keyword {}", &keyword);

        let x = search_documents(&keyword).unwrap();
        for (_s, d, _snippet) in x {
            //println!("{:?}", snippet.fragment());
            //keyword_context += snippet.fragment()
            //println!("{:?}", d.field_values()[1].value.as_str().unwrap());
            //keyword_context += d.field_values()[1].value.as_text().unwrap_or(""); //.as_str().unwrap();
            keyword_context += match &d.field_values()[1].value {
                OwnedValue::Str(s) => s,
                _ => {
                    println!("Warning: Expected text field, found different type");
                    ""
                }
            };
        }
    }
    println!("{}", keyword_context);
    
    // Search for the nearest neighbours.
    if let Some(query) = &args.query {
        println!("Asking \"{}\"", &query);
        
        let data = chunk_string(query, args.chunksize);
        //println!("{:?}", data); // Only if verbose!
        let vectors = embeddings(data).expect("Cannot create embeddings.");
        let v = vectors.get(0).expect("uh");
        let embedded_query = Vector((&v).to_vec());
        //dbg!("{}", &embedded_query);
        let result = collection.search(&embedded_query, args.nearest).unwrap();
        //let result = collection.true_search(&embedded_query, args.nearest).unwrap();

        for res in &result {
            let hm = md_to_hashmap(&res.data).unwrap();
            let filename = md_to_str(hm.get("filename").unwrap()).unwrap();
            let chunk_nr = md_to_str(hm.get("ccnt").unwrap()).unwrap();
            let dist = res.distance;
            print!("{dist:.4} | {filename}/{chunk_nr}");
            if dist < args.maxdist {
                println!(" *");
            } else {
                println!(" | filtered");
            }
        }

        let mut context_str = String::new();
        let result: Vec<SearchResult> = result.into_iter().filter(|s| s.distance < args.maxdist).collect();
        if result.len() == 0 && keyword_context.len() == 0 {
            println!("All results have been filtered :-(");
            context_str = "Use any knowledge you have.".to_string();
        } else {
            context_str += &("(document \"keywords\", with contents:".to_owned() + &keyword_context + ")");
        }
        
        // Double, cache the results in the first iteration.
        let mut sep = "";
        for res in &result {
            let hm = md_to_hashmap(&res.data).unwrap();
            let filename = md_to_str(hm.get("filename").unwrap()).unwrap();
            let chunk_nr = md_to_str(hm.get("ccnt").unwrap()).unwrap();
            let text = md_to_str(hm.get("text").unwrap()).unwrap();
            
            if args.showcontext == true {
                println!("  {}\n", text);
            }
            context_str += &(sep.to_owned() + "\n(document:\"" + &filename + "/" + &chunk_nr + "\", with contents:" + &text + ")");
            sep = ", ";
        }

        let _ts_start = chrono::Local::now();

        if args.ollama == false {
            let mut q = format!("You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not make up facts. Print the name of document used from the context. Do not repeat the question or references. Do not invent answers or references. Today is {date}. Context: {context} \nQuestion: {question}", context=context_str, question=query, date=chrono::Local::now().format("%A, %B %e, %Y"));
            if q.len() > 4096 { // Come to think of it, those might be tokens...
                println!("Prompt longer than 4096, truncating.");
                q = q[0..=4095].to_string();
            }
            
            //let q = format!("{question}", question=query);
            //let q = format!("Du är en vänlig och hjälpsam AI-assistent. Ditt svar ska vara kortfattat och använda sammanhanget om möjligt. Skriv ut namnet på det dokument som används från sammanhanget. Upprepa inte frågan eller referenserna. Svara på Svenska! Idag är {date}. Sammanhang: {context}. Fråga: {question}.", context=context_str, question=query, date=chrono::Local::now().format("%A, %B %e, %Y"));
            let ans = run_qmistral(&q);
            let _ts_end = chrono::Local::now();
            //println!("{:?}", ts_end - ts_start);
            println!("\n{}", ans.unwrap().trim().to_string());
        } else {
            // We create a system message and a qustion.
            let sys_message = format!("You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not make up facts. Print the name of document used from the context. Do not repeat the question or references. Do not invent answers or references. Today is {date}. Context: {context}", context=context_str, date=chrono::Local::now().format("%A, %B %e, %Y"));
            
            let q = format!("Question: {question}", question=query);

            if args.showprompt == true {
                println!("\n{}\n", sys_message);
            }

            let _ = genai_generate(&sys_message, &q, &args.ollama_model);

            /*
            let mut q = format!("You are a friendly and helpful AI assistant. Your answer should be to the point and use the context if possible. Do not make up facts. Print the name of document used from the context. Do not repeat the question or references. Do not invent answers or references. Today is {date}. Context: {context} \nQuestion: {question}", context=context_str, question=query, date=chrono::Local::now().format("%A, %B %e, %Y"));
            let sys_message = "";
            let _ = ollama_generate(&sys_message, &q, &args.ollama_model);
            */
        }
    }

    Ok(())
}
