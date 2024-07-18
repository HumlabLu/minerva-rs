use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Clone, Debug)]
pub struct GlobalConfig {
    pub oasysdb_dir: PathBuf,
    pub tantivy_dir: PathBuf,
    pub tantivy_chunk_size: usize,
}

pub struct GlobalConfigBuilder {
    oasysdb_dir: Option<PathBuf>,
    tantivy_dir: Option<PathBuf>,
    tantivy_chunk_size: Option<usize>,
}

impl GlobalConfigBuilder {
    pub fn new() -> Self {
        GlobalConfigBuilder {
            oasysdb_dir: None,
            tantivy_dir: None,
            tantivy_chunk_size: None,
        }
    }

    pub fn oasysdb_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.oasysdb_dir = Some(path.into());
        self
    }

    pub fn tantivy_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.tantivy_dir = Some(path.into());
        self
    }

    pub fn tantivy_chunk_size(mut self, size: usize) -> Self {
        self.tantivy_chunk_size = Some(size.into());
        self
    }

    pub fn build(self) -> Result<GlobalConfig, String> {
        Ok(GlobalConfig {
            oasysdb_dir: self.oasysdb_dir.unwrap_or_else(|| PathBuf::from("./oasysdb")),
            tantivy_dir: self.tantivy_dir.unwrap_or_else(|| PathBuf::from("./tantivy")),
            tantivy_chunk_size: self.tantivy_chunk_size.unwrap_or_else(|| 2048usize),
        })
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            oasysdb_dir: PathBuf::from("./db/oasysdb"),
            tantivy_dir: PathBuf::from("./db/tantivy"),
            tantivy_chunk_size: 2048usize,
        }
    }
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: Mutex<Option<GlobalConfig>> = Mutex::new(None);
}

pub fn initialise_globals(config: GlobalConfig) {
    let mut global = GLOBAL_CONFIG.lock().unwrap();
    *global = Some(config);
}

pub fn get_global_config() -> Result<GlobalConfig, String> {
    GLOBAL_CONFIG.lock().unwrap().clone().ok_or_else(|| "Global config not initialised".to_string())
}
