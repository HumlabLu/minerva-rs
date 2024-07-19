use tantivy::TantivyDocument;
use tantivy::schema::Value;
use crate::tant::get_index_schema;
use std::fmt;
use oasysdb::metadata::Metadata;
use oasysdb::collection::SearchResult;

// One "unit" of information from the DB.
// Should be used by the oasysdb as well, with applicable try_from
// implementations.
pub struct MinervaDoc {
    pub title: String,
    pub body: String,
    pub page_num: u64,
    pub chunk_num: u64,
    pub hash_body: String,
    pub ulid: String,
    pub score: f32, // Both distance and tantivy score...
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
            ulid: doc.get_first(schema.get_field("ulid")?)
                .and_then(|v| v.as_str()).ok_or("Missing ulid")?
                .to_string(),
            score: 0.0,
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
            ulid: doc.get_first(schema.get_field("ulid")?)
                .and_then(|v| v.as_str()).ok_or("Missing ulid")?
                .to_string(),
            score: 0.0,
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
        doc.add_text(schema.get_field("ulid")?, &minerva_doc.ulid);

        Ok(doc)
    }
}

// This one just dumps all the fields, without truncating.
impl fmt::Display for MinervaDoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}/{} {}", self.title, self.page_num, self.chunk_num, self.ulid)?;
        write!(f, "    {}", &self.body)
    }
}

impl fmt::Debug for MinervaDoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MinervaDoc")
            .field("title", &self.title)
            .field("body", &self.body)
            .field("page_num", &self.page_num)
            .field("chunk_num", &self.chunk_num)
            .field("ulid", &self.ulid)
            .field("hash_body", &self.hash_body)
            .finish()
    }
}

// Oasysdb

#[derive(Debug)]
pub enum ConversionError {
    MissingField(String),
    InvalidType(String),
}

/*
            let filename = md_to_str(hm.get("filename").unwrap()).unwrap();
            let chunk_nr = md_to_str(hm.get("ccnt").unwrap()).unwrap();
            let text = md_to_str(hm.get("text").unwrap()).unwrap();

*/
impl TryFrom<&SearchResult> for MinervaDoc {
    type Error = ConversionError;

    fn try_from(result: &SearchResult) -> Result<Self, Self::Error> {
        if let Metadata::Object(map) = &result.data {
            let title = match map.get("filename") {
                Some(Metadata::Text(value)) => value.clone(),
                _ => return Err(ConversionError::InvalidType("filename".to_string())),
            };
            
            let body = match map.get("text") {
                Some(Metadata::Text(value)) => value.clone(),
                _ => return Err(ConversionError::InvalidType("text".to_string())),
            };

            let page_num = 0;
            
            let chunk_num = match map.get("ccnt") {
                Some(Metadata::Integer(value)) => *value as u64,
                _ => return Err(ConversionError::InvalidType("ccnt".to_string())),
            };

            let hash_body = "?".to_string();
            
            let ulid = match map.get("ulid") {
                Some(Metadata::Text(value)) => value.clone(),
                _ => return Err(ConversionError::InvalidType("ulid".to_string())),
            };

            let score = 0.0;

            Ok(MinervaDoc {
                title,
                body,
                page_num,
                chunk_num,
                hash_body,
                ulid,
                score,
            })
        } else {
            Err(ConversionError::InvalidType("data".to_string()))
        }
    }
}
