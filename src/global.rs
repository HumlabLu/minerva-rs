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
