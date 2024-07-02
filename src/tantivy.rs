use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tempfile::TempDir;
use std::path::Path;
use tantivy::snippet::{Snippet, SnippetGenerator};
use once_cell::sync::Lazy;

static SCHEMA: Lazy<Schema> = Lazy::new(|| {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    schema_builder.add_u64_field("page_number", STORED);
    schema_builder.add_u64_field("chunk_number", STORED);
    schema_builder.build()
});

pub fn tanttest() -> tantivy::Result<()> {
    let index_path = Path::new("db/tantivy");
    println!("Index path: {:?}", index_path);
    
    let schema = &*SCHEMA;    
    let index = Index::open_in_dir(&index_path).unwrap_or_else(|_| {
        Index::create_in_dir(&index_path, schema.clone()).unwrap()
    });
    
    let mut index_writer: IndexWriter = index.writer(50_000_000)?; // 50 MB buffer for indexing.
    let title = schema.get_field("title").unwrap();
    println!("title: {:?}", title);
    let body = schema.get_field("body").unwrap();
    println!("body: {:?}", body);
    let page_number = schema.get_field("page_number").unwrap();
    println!("page_number: {:?}", page_number);
    let chunk_number = schema.get_field("chunk_number").unwrap();
    println!("chunk_number: {:?}", chunk_number);
    
    let mut old_man_doc = TantivyDocument::default();
    old_man_doc.add_text(title, "The Old Man and the Sea");
    old_man_doc.add_text(
        body,
        "He was an old man who fished alone in a skiff in the Gulf Stream and he had gone \
         eighty-four days now without taking a fish.",
    );
    old_man_doc.add_u64(page_number, 28);
    index_writer.add_document(old_man_doc)?;
    
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
    ))?;
    
    index_writer.commit()?; // Finish processing the queue.

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

    let (_score, doc_address) = searcher
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

    Ok(())
}

