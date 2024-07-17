use tantivy::TantivyDocument;
use tantivy::schema::Value;
use crate::get_index_schema;
use std::fmt;


// One "unit" of information from the DB.
// Should be used by the oasysdb as well, with applicable try_from
// implementations.
pub struct MinervaDoc {
    pub title: String,
    pub body: String,
    pub page_num: u64,
    pub chunk_num: u64,
    pub hash_body: String,
}

impl TryFrom<TantivyDocument> for MinervaDoc {
    type Error = Box<dyn std::error::Error>;

    fn try_from(doc: TantivyDocument) -> Result<Self, Self::Error> {
        let (_index, schema) = get_index_schema()?;
        Ok(MinervaDoc {
            title: doc.get_first(schema.get_field("title")?)
                .and_then(|v| v.as_str())
                .ok_or("Missing title")?
                .to_string(),
            body: doc.get_first(schema.get_field("body")?)
                .and_then(|v| v.as_str())
                .ok_or("Missing body")?
                .to_string(),
            page_num: doc.get_first(schema.get_field("page_number")?)
                .and_then(|v| v.as_u64())
                .ok_or("Missing page_number")?,
            chunk_num: doc.get_first(schema.get_field("chunk_number")?)
                .and_then(|v| v.as_u64())
                .ok_or("Missing chunk_number")?,
            hash_body: doc.get_first(schema.get_field("hash_body")?)
                .and_then(|v| v.as_str()).ok_or("Missing hash_body")?
                .to_string(),
        })
    }
}

impl TryFrom<&TantivyDocument> for MinervaDoc {
    type Error = Box<dyn std::error::Error>;
    
    fn try_from(doc: &TantivyDocument) -> Result<Self, Self::Error> {
        let (_index, schema) = get_index_schema()?;
        Ok(MinervaDoc {
            title: doc.get_first(schema.get_field("title")?)
                .and_then(|v| v.as_str())
                .ok_or("Missing title")?
                .to_string(),
            body: doc.get_first(schema.get_field("body")?)
                .and_then(|v| v.as_str())
                .ok_or("Missing body")?
                .to_string(),
            page_num: doc.get_first(schema.get_field("page_number")?)
                .and_then(|v| v.as_u64())
                .ok_or("Missing page_number")?,
            chunk_num: doc.get_first(schema.get_field("chunk_number")?)
                .and_then(|v| v.as_u64())
                .ok_or("Missing chunk_number")?,
            hash_body: doc.get_first(schema.get_field("hash_body")?)
                .and_then(|v| v.as_str()).ok_or("Missing hash_body")?
                .to_string(),
        })
    }
}

impl TryFrom<&MinervaDoc> for TantivyDocument {
    type Error = Box<dyn std::error::Error>;

    fn try_from(minerva_doc: &MinervaDoc) -> Result<Self, Self::Error> {
        let (_index, schema) = get_index_schema().unwrap();

        let mut doc = TantivyDocument::default();
        doc.add_text(schema.get_field("title")?, &minerva_doc.title);
        doc.add_text(schema.get_field("body")?, &minerva_doc.body);
        doc.add_u64(schema.get_field("page_number")?, minerva_doc.page_num);
        doc.add_u64(schema.get_field("chunk_number")?, minerva_doc.chunk_num);
        doc.add_text(schema.get_field("hash_body")?, &minerva_doc.hash_body);

        Ok(doc)
    }
}

// This one just dumps all the fields, without truncating.
impl fmt::Display for MinervaDoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}/{}", self.title, self.page_num, self.chunk_num)?;
        write!(f, "    {}", &self.body)
    }
}
