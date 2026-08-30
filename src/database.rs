mod master_key;
mod note;
mod notebook;

use crate::JoplinError;
use crate::decoder::{DecodedJson, DecodedMd};
use crate::password::{JoplinEnvPasswordProvider, JoplinPasswordProvider};
pub use master_key::JoplinMasterKey;
pub use notebook::JoplinNotebook;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

struct Cache {
    children: HashMap<Option<String>, Vec<DecodedMd>>,
}

impl Cache {
    fn new() -> Cache {
        Cache {
            children: HashMap::new(),
        }
    }

    fn get_children(&self, parent_id: &Option<String>) -> &Vec<DecodedMd> {
        static EMPTY_VEC: Vec<DecodedMd> = Vec::new();
        self.children.get(parent_id).unwrap_or(&EMPTY_VEC)
    }

    fn get_children_mut(&mut self, parent_id: &Option<String>) -> &mut Vec<DecodedMd> {
        self.children.entry(parent_id.clone()).or_default()
    }
}

pub struct JoplinDatabase<P: JoplinPasswordProvider = JoplinEnvPasswordProvider> {
    master_keys: HashMap<String, JoplinMasterKey>,
    notebooks: Vec<JoplinNotebook>,
    password_provider: Mutex<P>,
}

impl JoplinDatabase<JoplinEnvPasswordProvider> {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<JoplinDatabase<JoplinEnvPasswordProvider>, JoplinError> {
        Self::from_path_with_password(path, JoplinEnvPasswordProvider::new())
    }
}

impl<PasswordProvider: JoplinPasswordProvider> JoplinDatabase<PasswordProvider> {
    pub fn from_path_with_password<P: AsRef<Path>>(
        path: P,
        password_provider: PasswordProvider,
    ) -> Result<JoplinDatabase<PasswordProvider>, JoplinError> {
        let path = path.as_ref();
        let mut cache = Cache::new();
        let mut master_keys = HashMap::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.file_stem().is_some_and(|s| s.len() == 32)
                && path.extension().is_some_and(|s| s == "md")
            {
                match DecodedMd::from_file(&path) {
                    Ok(decoded_node) => {
                        cache
                            .get_children_mut(&decoded_node.parent_id)
                            .push(decoded_node);
                    }
                    Err(err) => {
                        let filename = path
                            .file_name()
                            .map(OsStr::to_string_lossy)
                            .unwrap_or_default();
                        warn!("Error reading {}: {}", filename, err);
                    }
                }
            } else if path.file_name().is_some_and(|s| s == "info.json") {
                match DecodedJson::from_file(&path) {
                    Ok(decoded_info) => {
                        info!("Loaded {} master key(s)", decoded_info.master_keys.len());
                        for decoded_key in decoded_info.master_keys {
                            let key = JoplinMasterKey::from_decoded(&decoded_key)?;
                            master_keys.insert(decoded_key.id, key);
                        }
                    }
                    Err(err) => {
                        let filename = path
                            .file_name()
                            .map(OsStr::to_string_lossy)
                            .unwrap_or_default();
                        warn!("Error reading {}: {}", filename, err);
                    }
                }
            }
        }

        let notebooks = cache
            .get_children(&None)
            .iter()
            .flat_map(|d| JoplinNotebook::from_md(d, &cache))
            .collect();
        Ok(JoplinDatabase {
            master_keys,
            notebooks,
            password_provider: Mutex::new(password_provider),
        })
    }

    pub fn get_notebooks(&self) -> &Vec<JoplinNotebook> {
        &self.notebooks
    }
}
