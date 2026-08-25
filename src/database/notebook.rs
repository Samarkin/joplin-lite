use crate::database::Cache;
use crate::database::note::JoplinNote;
use crate::decoder::{DecodedMd, DecodedMdBody, DecodedMdType};
use std::sync::Arc;

pub struct JoplinNotebook {
    title: String,
    notebooks: Vec<JoplinNotebook>,
    notes: Vec<Arc<JoplinNote>>,
}

impl JoplinNotebook {
    pub(crate) fn from_md(d: &DecodedMd, cache: &Cache) -> Option<JoplinNotebook> {
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
            Some(JoplinNotebook {
                title,
                notebooks,
                notes,
            })
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
