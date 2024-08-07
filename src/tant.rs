use tantivy::collector::{TopDocs, Count};
use tantivy::query::{QueryParser, TermQuery, FuzzyTermQuery, PhraseQuery};
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tantivy::directory::MmapDirectory;
use std::path::Path;
use tantivy::snippet::{Snippet, SnippetGenerator};
use once_cell::sync::Lazy;
use std::fs;
use crate::embedder::{chunk_string};
use crate::global::get_global_config;
use terminal_size::{Width, terminal_size};
use crate::minervadoc::*;
use ulid::Ulid;

static SCHEMA: Lazy<Schema> = Lazy::new(|| {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT | STORED);
    schema_builder.add_u64_field("page_number", STORED);
    schema_builder.add_u64_field("chunk_number", STORED);
    schema_builder.add_text_field("hash_body", STRING | STORED);
    schema_builder.add_text_field("ulid", STRING | STORED);
    schema_builder.build()
});

// Insert the four main fields, and calculate the body_hash
// from the body text.
#[allow(dead_code)]
pub fn insert_document(index: &Index, title: &str, body: &str, page_number: u64, chunk_number: u64) -> tantivy::Result<bool> {
    let schema = index.schema();
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;
    
    let title_field = schema.get_field("title").unwrap();
    let body_field = schema.get_field("body").unwrap();
    let page_number_field = schema.get_field("page_number").unwrap();
    let chunk_number_field = schema.get_field("chunk_number").unwrap();
    let hash_body_field = schema.get_field("hash_body").unwrap();
    let ulid_field = schema.get_field("ulid").unwrap();
    
    //let hash_body_value = blake3::hash(body.as_bytes());
    let hash_body_value_bytes = body.as_bytes();
    //let hash_body_value_bytes = &hash_body_value_bytes[0..=255]; // Might be faster like this?
    let hash_body_value = blake3::hash(hash_body_value_bytes);
    
    //println!("{}", hash_body_value);

    match document_exists(&index, &hash_body_value.to_string()) {
        Ok(exists) => {
            if ! exists {
                //println!("Adding document.");
                let ulid = Ulid::new().to_string();
                let _ = index_writer.add_document(doc!(
                    title_field => title,
                    body_field => body,
                    page_number_field => page_number,
                    chunk_number_field => chunk_number,
                    hash_body_field => hash_body_value.to_string(),
                    ulid_field => ulid
                ));
                index_writer.commit()?;
                return Ok(true);
                //println!("Added.");
            } else {
                return Ok(false);
            }
        }
        Err(e) => println!("Error occurred: {:?}", e)
    }
        
    Ok(false)
}

pub fn insert_document_noc(index: &Index, index_writer: &IndexWriter, title: &str, body: &str, page_number: u64, chunk_number: u64) -> tantivy::Result<bool> {
    let schema = index.schema();
    
    let title_field = schema.get_field("title").unwrap();
    let body_field = schema.get_field("body").unwrap();
    let page_number_field = schema.get_field("page_number").unwrap();
    let chunk_number_field = schema.get_field("chunk_number").unwrap();
    let hash_body_field = schema.get_field("hash_body").unwrap();
    let ulid_field = schema.get_field("ulid").unwrap();

    //let hash_body_value = blake3::hash(body.as_bytes());
    let hash_body_value_bytes = body.as_bytes();
    //let hash_body_value_bytes = &hash_body_value_bytes[0..=255]; // Might be faster like this?
    let hash_body_value = blake3::hash(hash_body_value_bytes);
    
    //println!("{}", hash_body_value);

    match document_exists(&index, &hash_body_value.to_string()) {
        Ok(exists) => {
            if ! exists {
                //println!("Adding document.");
                let ulid = Ulid::new().to_string();
                let _ = index_writer.add_document(doc!(
                    title_field => title,
                    body_field => body,
                    page_number_field => page_number,
                    chunk_number_field => chunk_number,
                    hash_body_field => hash_body_value.to_string(),
                    ulid_field => ulid
                ));
                return Ok(true);
                //println!("Added.");
            } else {
                return Ok(false);
            }
        }
        Err(e) => println!("Error occurred: {:?}", e)
    }
        
    Ok(false)
}

// Takes ownership of the doc!
#[allow(dead_code)]
pub fn insert_doc(index: &Index, mut tdoc: TantivyDocument) -> tantivy::Result<()> {
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    // Calculate the body_hash.
    let schema = index.schema();
    let body_field = schema.get_field("body").unwrap();
    let hash_body_field = schema.get_field("hash_body").unwrap();
    /*for x in tdoc.get_all(body_field) {
        println!("{:?}", x);
    }*/
    if let Some(x) = tdoc.get_first(body_field) { // We only process the first one.
        // https://github.com/quickwit-oss/tantivy/pull/2071 for as_str() info.
        let hash_body_value = x.as_str().unwrap();
        let hash_body_value_bytes = hash_body_value.as_bytes();
        let hash_body_value_bytes = &hash_body_value_bytes[0..=255]; // Might be faster like this?
        let hash_body_value = blake3::hash(hash_body_value_bytes);

        match document_exists(&index, &hash_body_value.to_string()) {
            Ok(exists) => {
                if ! exists {
                    println!("Adding document.");
                    tdoc.add_text(hash_body_field, hash_body_value);
                    index_writer.add_document(tdoc)?;
                    index_writer.commit()?;
                }
            }
            Err(e) => println!("Error occurred: {:?}", e)
        }
        
    } // Else add hash 0 or something? Can this happen?
    
    Ok(())
}

// We need to be able to define the DB path as well...
// Or have it as an argument here!
pub fn get_index_schema() -> tantivy::Result<(Index, Schema)> {
    let config = get_global_config().unwrap();
    let path = Path::new(&config.tantivy_dir);
    if !path.exists() {
        println!("Creating directory: {:?}", path);
        fs::create_dir_all(path).expect("Cannot create oasysdb directory.");
    }
    let schema = &*SCHEMA;
    let directory = MmapDirectory::open(path)?;
    let index = Index::open_or_create(directory, schema.clone())?;
    Ok((index, schema.clone()))
}

// index should be a parameter, because we want to know where
// we store it the document. Called from main.rs.
pub fn insert_file<P: AsRef<Path>>(index: &Index, path: P) -> tantivy::Result<u64> {
    let path_ref = path.as_ref();
    let filename_str = path_ref.to_str().ok_or_else(|| {
        tantivy::TantivyError::InvalidArgument(format!("Invalid path: {:?}", path_ref))
    })?;
    let contents = fs::read_to_string(path_ref).expect("Cannot read file.");

    let config = get_global_config().unwrap();
    let tantivy_chunk_size = config.tantivy_chunk_size;
    
    let chunks = chunk_string(&contents, tantivy_chunk_size); // Arbitrary value.
    let mut chunk_counter = 0u64;
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;
    for chunk in chunks {
        let inserted = insert_document_noc(&index, &index_writer, filename_str, &chunk, 0, chunk_counter).unwrap();
        //println!("Inserted chunk {}", chunk_counter);
        if inserted {
            chunk_counter += 1;
        }
    }
    index_writer.commit()?;
    
    Ok(chunk_counter)
}

// Needs testing/work.
// Probably not on an Index but on an IndexWriter so we check before
// inserting in a loop?
// unique_id needs to be a hash on the title+body, or something.
// Adapted from https://github.com/quickwit-oss/tantivy/blob/main/examples/deleting_updating_documents.rs
pub fn document_exists(index: &Index, hash_body: &str) -> tantivy::Result<bool> {
    //let reader: IndexReader = index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into()?;
    let reader = index.reader()?;
    let schema = index.schema();

    let hash_body_field = schema.get_field("hash_body")?;
    let hash_body_term = Term::from_field_text(hash_body_field, hash_body);
    let term_query = TermQuery::new(hash_body_term, IndexRecordOption::Basic);
    
    let searcher = reader.searcher();
    let top_docs = searcher.search(&term_query, &TopDocs::with_limit(1))?;

   // println!("{:?}", top_docs);
    
    Ok(!top_docs.is_empty())
}

pub fn get_num_documents(index: &Index) -> tantivy::Result<u64> {
    //let reader: IndexReader = index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into()?;
    let reader = index.reader()?;
    let searcher = reader.searcher();
    Ok(searcher.segment_readers().iter().map(|segment_reader| segment_reader.num_docs() as u64).sum())
}

pub fn _tanttest() -> tantivy::Result<()> {
    let index_path = Path::new("db/tantivy");
    println!("Index path: {:?}", index_path);
    
    let schema = &*SCHEMA;
    let directory = MmapDirectory::open(Path::new(index_path))?;
    let index = Index::open_or_create(directory, schema.clone())?;

    let num_docs = get_num_documents(&index)?;
    println!("Number of documents in the index: {}", num_docs);
    
    insert_document(&index, "Another title with Mice", "Example body text \n\
Longer string.", 1, 1)?;

    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();
    let page_number = schema.get_field("page_number").unwrap();
    let chunk_number = schema.get_field("chunk_number").unwrap();
    /*
    println!("title: {:?}", title);
    println!("body: {:?}", body);
    println!("page_number: {:?}", page_number);
    println!("chunk_number: {:?}", chunk_number);
     */
    
    let mut old_man_doc = TantivyDocument::default();
    old_man_doc.add_text(title, "The Old Man and the Sea");
    old_man_doc.add_text(
        body,
        "He was an old man who fished alone in a skiff in the Gulf Stream and he had gone \
         eighty-four days now without taking a fish.",
    );
    old_man_doc.add_u64(page_number, 28);
    old_man_doc.add_u64(chunk_number, 8);
    insert_doc(&index, old_man_doc).unwrap();

    insert_document(&index, "Frankenstein",
        "You will rejoice to hear that no disaster has accompanied the commencement of an \
             enterprise which you have regarded with such evil forebodings.  I arrived here \
             yesterday, and my first task is to assure my dear sister of my welfare and \
             increasing confidence in the success of my undertaking.",
        42, 128).unwrap();
        
    let _reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![title, body]);

    let query = query_parser.parse_query("sea whale")?;
    //let query = query_parser.parse_query("prometheus")?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    let snippet_generator = SnippetGenerator::create(&searcher, &*query, body)?;

    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        println!("retrieved {score}: {}", retrieved_doc.to_json(&schema));
        
        let snippet = snippet_generator.snippet_from_doc(&retrieved_doc);
        println!("snippet: {:?}", snippet); //.to_html());
    }
    let query = query_parser.parse_query("title:sea^20 body:whale^70")?;

    let (_score, _doc_address) = searcher
        .search(&query, &TopDocs::with_limit(1))?
        .into_iter()
        .next()
        .unwrap();
    //println!("{_score} {doc_address:?}"); ?

    /*
    let explanation = query.explain(&searcher, doc_address)?;
    println!("explanation: {}", explanation.to_pretty_json());
    */

    let query = query_parser.parse_query("title:Mice")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        println!("retrieved {score}: {}", retrieved_doc.to_json(&schema));
    }

    let num_docs = get_num_documents(&index)?;
    println!("Number of documents in the index: {}", num_docs);

    Ok(())
}

#[allow(dead_code)]
fn str_to_terms(text: &str, field: Field) -> Vec<Term> {
    text.split_whitespace()
        .map(|word| Term::from_field_text(field, word))
        .collect()
}

#[allow(dead_code)]
pub fn phrase_search_documents(query_str: &str, limit: usize) -> tantivy::Result<Vec<(f32, TantivyDocument, Option<Snippet>, String)>> {

    let (index, schema) = get_index_schema().unwrap();
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let body_field = schema.get_field("body").unwrap();
    let title_field = schema.get_field("title").unwrap();
    let chunk_number_field = schema.get_field("chunk_number").unwrap();

    let word_tuples = str_to_terms(query_str, body_field);
    
    let mut phrase_query = PhraseQuery::new(word_tuples);
    phrase_query.set_slop(2);

    // Search
    let top_docs = searcher.search(&phrase_query, &TopDocs::with_limit(limit))?;

    let mut documents = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

        // Create an info "metadata" string.
        let chunk_number = retrieved_doc.get_first(chunk_number_field).unwrap();
        let title = retrieved_doc.get_first(title_field).unwrap();
        //println!("{:?}/{:?}", extract_string(title).unwrap(), extract_u64(chunk_number).unwrap());
        let info = format!("{}/{}", extract_string(title).unwrap(), extract_u64(chunk_number).unwrap());
        documents.push((score, retrieved_doc, None, info));
    }

    Ok(documents)
}

pub fn search_documents(query_str: &str, limit: usize) -> tantivy::Result<Vec<(f32, TantivyDocument, Option<Snippet>, String)>> {
    // with_limit() does not accept 0.
    if limit == 0 {
        return Ok(vec![]);
    }

    let index_path = Path::new("db/tantivy");
    
    let schema = &*SCHEMA;
    let directory = MmapDirectory::open(index_path)?;
    let index = Index::open_or_create(directory, schema.clone())?;
    
    let title_field = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();
    let chunk_number_field = schema.get_field("chunk_number").unwrap();
    
    let reader = index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into()?;
    let searcher = reader.searcher();
    
    let query_parser = QueryParser::for_index(&index, vec![title_field, body]);
    let query = query_parser.parse_query(query_str)?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

    let snippet_generator = SnippetGenerator::create(&searcher, &*query, body)?;
    
    let mut documents = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        //println!("doc: {:?}", retrieved_doc);
        let snippet = snippet_generator.snippet_from_doc(&retrieved_doc);
        //println!("snippet: {:?}", snippet.to_html());
        //println!("custom highlighting: {}", highlight(&snippet));

        // Create an info "metadata" string.
        let chunk_number = retrieved_doc.get_first(chunk_number_field).unwrap();
        let title = retrieved_doc.get_first(title_field).unwrap();
        //println!("{:?}/{:?}", extract_string(title).unwrap(), extract_u64(chunk_number).unwrap());
        let info = format!("{}/{}", extract_string(title).unwrap(), extract_u64(chunk_number).unwrap());
        documents.push((score, retrieved_doc, Some(snippet), info));
    }
    
    Ok(documents)
}

// There is as_str() and as_u64() as well...
// https://docs.rs/tantivy/latest/tantivy/schema/document/trait.Value.html#method.as_str
fn extract_string(value: &OwnedValue) -> Option<String> {
    match value {
        OwnedValue::Str(s) => Some(s.clone()),
        _ => None,
    }
}

fn extract_u64(value: &OwnedValue) -> Option<u64> {
    match value {
        OwnedValue::U64(u) => Some(*u),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn fuzzy_search_documents(query_str: &str) -> tantivy::Result<Vec<(f32, TantivyDocument, Option<Snippet>, String)>> {
    let (index, schema) = get_index_schema().unwrap();
    
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let body_field = schema.get_field("body").unwrap();
    let title_field = schema.get_field("title").unwrap();
    let chunk_number_field = schema.get_field("chunk_number").unwrap();

    let term = Term::from_field_text(body_field, query_str);
    let query = FuzzyTermQuery::new(term, 1, true); // 1 is edit distance.

    let (top_docs, count) = searcher.search(&query, &(TopDocs::with_limit(3), Count)).unwrap();
    println!("Count {}", count);
    let mut documents = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

        let chunk_number = retrieved_doc.get_first(chunk_number_field).unwrap();
        let title = retrieved_doc.get_first(title_field).unwrap();
        let info = format!("{}/{}", extract_string(title).unwrap(), extract_u64(chunk_number).unwrap());
        
        documents.push((score, retrieved_doc, None, info)); // No snippets in fuzzy search.
    }

    Ok(documents)
}

#[allow(dead_code)]
fn highlight(snippet: &Snippet) -> String {
    let mut result = String::new();
    let mut start_from = 0;

    for fragment_range in snippet.highlighted() {
        result.push_str(&snippet.fragment()[start_from..fragment_range.start]);
        result.push_str(" --> ");
        result.push_str(&snippet.fragment()[fragment_range.clone()]);
        result.push_str(" <-- ");
        start_from = fragment_range.end;
    }

    result.push_str(&snippet.fragment()[start_from..]);
    result
}

// TODO: Make a struct with the schema values.
/*
#[derive(Debug, Clone)]
pub struct MinervaDoc {
    pub title: String,
    pub body: String,
    pub page_num: u64,
    pub chunk_num: u64,
    pub hash_body: String
    }
    Plus an into() or from() function for TantivyDoc to MinervaDoc?
*/
pub fn print_contents() -> Result<(), Box<dyn std::error::Error>> {
    let (index, schema) = get_index_schema().unwrap();
    
    // Get field accessors
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();
    let _page_number = schema.get_field("page_number").unwrap();
    let _chunk_number = schema.get_field("chunk_number").unwrap();
    let _hash_body = schema.get_field("hash_body").unwrap();

    // Create a searcher
    let reader = index.reader()?;
    let searcher = reader.searcher();

    // Search for all documents
    let query_parser = tantivy::query::QueryParser::for_index(&index, vec![title, body]);
    let query = query_parser.parse_query("*")?; // Use the "all" query!

    // Collect all documents (adjust the number if you have a large database)
    let top_docs = searcher.search(&query, &TopDocs::with_limit(1000000))?;

    let width = terminal_size().map(|(Width(w), _)| w).unwrap_or(80);
    let body_length = width.saturating_sub(8); // Subtract some space for padding.

    // Print each document.
    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        let minerva_doc: MinervaDoc = (&retrieved_doc)
            .try_into()
            .expect("Cannot convert TantivyDoc to MinervaDoc!");
        //println!("{}", &minerva_doc);
        
        let body_text = &minerva_doc.body
            .replace('\n', " ")
            .replace('\r', "");
        let truncated_body = body_text.chars().take(body_length.into()).collect::<String>();
        
        println!("{} {}/{}", minerva_doc.title,
            minerva_doc.page_num,
            minerva_doc.chunk_num
        );
        println!("    {}", truncated_body);

        //println!("Hash Body: {:?}", retrieved_doc.get_first(hash_body).unwrap());
    }

    Ok(())
}

pub fn delete_all_documents() -> Result<(), Box<dyn std::error::Error>> {
    let (index, _schema) = get_index_schema().unwrap();
    
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    index_writer.delete_all_documents()?;
    index_writer.commit()?;

    Ok(())
}
