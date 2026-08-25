use crate::error::JoplinError;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize)]
pub struct DecodedMasterKey {
    pub encryption_method: u32,
    pub id: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct DecodedJson {
    #[serde(rename = "masterKeys")]
    pub master_keys: Vec<DecodedMasterKey>,
}

impl DecodedJson {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<DecodedJson, JoplinError> {
        Ok(serde_json::from_reader(File::open(path)?)?)
    }
}
