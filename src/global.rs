use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Clone, Debug)]
pub struct GlobalConfig {
    pub oasysdb_dir: PathBuf,
    pub tantivy_dir: PathBuf,
}

pub struct GlobalConfigBuilder {
    oasysdb_dir: Option<PathBuf>,
    tantivy_dir: Option<PathBuf>,
}

impl GlobalConfigBuilder {
    pub fn new() -> Self {
        GlobalConfigBuilder {
            oasysdb_dir: None,
            tantivy_dir: None,
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

    pub fn build(self) -> Result<GlobalConfig, String> {
        Ok(GlobalConfig {
            oasysdb_dir: self.oasysdb_dir.unwrap_or_else(|| PathBuf::from("./oasysdb")),
            tantivy_dir: self.tantivy_dir.unwrap_or_else(|| PathBuf::from("./tantivy")),
        })
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            oasysdb_dir: PathBuf::from("./db/oasysdb"),
            tantivy_dir: PathBuf::from("./db/tantivy"),
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

/*
use std::sync::Mutex;
use std::path::PathBuf;
use lazy_static::lazy_static;

pub struct GlobalConfig {
    pub oasysdb_dir: PathBuf,
    pub tantivy_dir: PathBuf,
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: Mutex<GlobalConfig> = Mutex::new(GlobalConfig {
        oasysdb_dir: PathBuf::from("db/oasysdb"),
        tantivy_dir: PathBuf::from("db/tantivy"),
    });
}

pub fn initialise_globals(oasysdb_dir: &str, tantivy_dir: &str) {
    let mut config = GLOBAL_CONFIG.lock().unwrap();
    config.oasysdb_dir = PathBuf::from(oasysdb_dir);
    config.tantivy_dir = PathBuf::from(tantivy_dir);
}
*/
