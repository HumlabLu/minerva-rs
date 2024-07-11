use tantivy::collector::{TopDocs, Count};
use tantivy::query::{QueryParser, TermQuery, FuzzyTermQuery};
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tantivy::directory::MmapDirectory;
use std::path::Path;
use tantivy::snippet::{Snippet, SnippetGenerator};
use once_cell::sync::Lazy;
use std::fs;
use crate::embedder::{chunk_string};
use crate::global::get_global_config;

static SCHEMA: Lazy<Schema> = Lazy::new(|| {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT | STORED);
    schema_builder.add_u64_field("page_number", STORED);
    schema_builder.add_u64_field("chunk_number", STORED);
    schema_builder.add_text_field("hash_body", STRING | STORED);
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

    //let hash_body_value = blake3::hash(body.as_bytes());
    let hash_body_value_bytes = body.as_bytes();
    //let hash_body_value_bytes = &hash_body_value_bytes[0..=255]; // Might be faster like this?
    let hash_body_value = blake3::hash(hash_body_value_bytes);
    
    //println!("{}", hash_body_value);

    match document_exists(&index, &hash_body_value.to_string()) {
        Ok(exists) => {
            if ! exists {
                //println!("Adding document.");
                let _ = index_writer.add_document(doc!(
                    title_field => title,
                    body_field => body,
                    page_number_field => page_number,
                    chunk_number_field => chunk_number,
                    hash_body_field => hash_body_value.to_string()
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

    //let hash_body_value = blake3::hash(body.as_bytes());
    let hash_body_value_bytes = body.as_bytes();
    //let hash_body_value_bytes = &hash_body_value_bytes[0..=255]; // Might be faster like this?
    let hash_body_value = blake3::hash(hash_body_value_bytes);
    
    //println!("{}", hash_body_value);

    match document_exists(&index, &hash_body_value.to_string()) {
        Ok(exists) => {
            if ! exists {
                //println!("Adding document.");
                let _ = index_writer.add_document(doc!(
                    title_field => title,
                    body_field => body,
                    page_number_field => page_number,
                    chunk_number_field => chunk_number,
                    hash_body_field => hash_body_value.to_string()
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
// we store it the document.
pub fn insert_file<P: AsRef<Path>>(index: &Index, path: P) -> tantivy::Result<u64> {
    let path_ref = path.as_ref();
    let filename_str = path_ref.to_str().ok_or_else(|| {
        tantivy::TantivyError::InvalidArgument(format!("Invalid path: {:?}", path_ref))
    })?;
    let contents = fs::read_to_string(path_ref).expect("Cannot read file.");

    let chunks = chunk_string(&contents, 2048); // Arbitrary value.
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
    
    /*
    let mut index_writer: IndexWriter = index.writer(50_000_000)?; // 50 MB buffer for indexing.
    index_writer.add_document(doc!(
        title => "Of Mice and Men",
        body => "A few miles south of Soledad, the Salinas River drops in close to the hillside \
            bank and runs deep and green. The water is warm too, for it has slipped twinkling \
            over the yellow sands in the sunlight before reaching the narrow pool. On one \
            side of the river the golden foothill slopes curve up to the strong and rocky \
            Gabilan Mountains, but on the valley side the water is lined with trees—willows \
            fresh and green with every spring, carrying in their lower leaf junctures the \
            debris of the winter’s flooding; and sycamores with mottled, white, recumbent \
            limbs and branches that arch over the pool",
        page_number => 42u64
    ))?;
    */
    /*
    index_writer.add_document(doc!(
        title => "Frankenstein",
        title => "The Modern Prometheus",
        body => "You will rejoice to hear that no disaster has accompanied the commencement of an \
             enterprise which you have regarded with such evil forebodings.  I arrived here \
             yesterday, and my first task is to assure my dear sister of my welfare and \
             increasing confidence in the success of my undertaking.",
        body => "Another body?",
        page_number => 42u64,
        chunk_number => 128u64
    ))?;*/
    
    //index_writer.commit()?; // Finish processing the queue.

    /*
    drop(index_writer);
    insert_doc(&index, old_man_doc).unwrap();
    */
    
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

pub fn search_documents(query_str: &str, limit: usize) -> tantivy::Result<Vec<(f32, TantivyDocument, Option<Snippet>)>> {
    // with_limit() does not accept 0.
    if limit == 0 {
        return Ok(vec![]);
    }

    let index_path = Path::new("db/tantivy");
    
    let schema = &*SCHEMA;
    let directory = MmapDirectory::open(index_path)?;
    let index = Index::open_or_create(directory, schema.clone())?;
    
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();
    
    let reader = index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into()?;
    let searcher = reader.searcher();
    
    let query_parser = QueryParser::for_index(&index, vec![title, body]);
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
        documents.push((score, retrieved_doc, Some(snippet)));
    }
    
    Ok(documents)
}

#[allow(dead_code)]
pub fn fuzzy_search_documents(query_str: &str) -> tantivy::Result<Vec<(f32, TantivyDocument, Option<Snippet>)>> {
    let (index, schema) = get_index_schema().unwrap();
    
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let body_field = schema.get_field("body").unwrap();
    
    let term = Term::from_field_text(body_field, query_str);
    let query = FuzzyTermQuery::new(term, 2, true);

    let (top_docs, count) = searcher.search(&query, &(TopDocs::with_limit(3), Count)).unwrap();
    println!("Count {}", count);
    let mut documents = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        documents.push((score, retrieved_doc, None)); // No snippets in fuzzy search.
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
