use crate::JoplinError;
use crate::decoder::{DecodedMd, DecodedMdBody, DecodedMdType};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

struct Cache {
    children: HashMap<Option<String>, Vec<DecodedMd>>,
}

impl Cache {
    fn new() -> Cache {
        Cache { children: HashMap::new() }
    }

    fn get_children(&self, parent_id: &Option<String>) -> &Vec<DecodedMd> {
        static EMPTY_VEC: Vec<DecodedMd> = Vec::new();
        self.children.get(parent_id).unwrap_or(&EMPTY_VEC)
    }

    fn get_children_mut(&mut self, parent_id: &Option<String>) -> &mut Vec<DecodedMd> {
        self.children
            .entry(parent_id.clone())
            .or_insert_with(Vec::new)
    }
}

pub struct JoplinNote {
    contents: String,
}

impl JoplinNote {
    fn from_md(d: &DecodedMd) -> Option<Arc<JoplinNote>> {
        if d.tp == DecodedMdType::Note && d.deleted_time.is_none() {
            Some(Arc::new(JoplinNote {
                contents: match &d.body {
                    DecodedMdBody::Unencrypted(s) => s.clone(),
                    DecodedMdBody::Encrypted(_) => String::from("<ENCRYPTED>"),
                }
            }))
        } else {
            None
        }
    }

    pub fn get_contents(&self) -> &str {
        &self.contents
    }
}

pub struct JoplinNotebook {
    title: String,
    notebooks: Vec<JoplinNotebook>,
    notes: Vec<Arc<JoplinNote>>,
}

impl JoplinNotebook {
    fn from_md(d: &DecodedMd, cache: &Cache) -> Option<JoplinNotebook> {
        if d.tp == DecodedMdType::Folder && d.deleted_time.is_none() {
            let mut notebooks = Vec::new();
            let mut notes = Vec::new();
            for c in cache.get_children(&Some(d.id.clone())) {
                if let Some(note) = JoplinNote::from_md(c) {
                    notes.push(note);
                }
                if let Some(note) = JoplinNotebook::from_md(c, cache) {
                    notebooks.push(note);
                }
            }
            let title = match &d.body {
                DecodedMdBody::Unencrypted(s) => s.clone(),
                DecodedMdBody::Encrypted(_) => String::from("<ENCRYPTED>"),
            };
            Some(JoplinNotebook { title, notebooks, notes })
        } else {
            None
        }
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_notebooks(&self) -> &Vec<JoplinNotebook> {
        &self.notebooks
    }

    pub fn get_notes(&self) -> &Vec<Arc<JoplinNote>> {
        &self.notes
    }
}

pub struct JoplinDatabase {
    notebooks: Vec<JoplinNotebook>,
}

impl JoplinDatabase {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<JoplinDatabase, JoplinError> {
        let path = path.as_ref();
        let mut cache = Cache::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.file_stem().is_some_and(|s| s.len() == 32) && path.extension().is_some_and(|s| s == "md") {
                match DecodedMd::from_file(&path) {
                    Ok(decoded_node) => {
                        cache.get_children_mut(&decoded_node.parent_id).push(decoded_node);
                    }
                    Err(err) => {
                        warn!("Error reading {}: {}", path.to_string_lossy(), err);
                    }
                }
            }
        }

        let notebooks = cache.get_children(&None)
            .iter()
            .flat_map(|d| JoplinNotebook::from_md(d, &cache))
            .collect();
        Ok(JoplinDatabase { notebooks })
    }

    pub fn get_notebooks(&self) -> &Vec<JoplinNotebook> {
        &self.notebooks
    }
}