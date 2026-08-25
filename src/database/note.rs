use crate::decoder::{DecodedMd, DecodedMdBody, DecodedMdType};
use std::sync::Arc;

pub struct JoplinNote {
    contents: String,
}

impl JoplinNote {
    pub(crate) fn from_md(d: &DecodedMd) -> Option<Arc<JoplinNote>> {
        if d.tp == DecodedMdType::Note && d.deleted_time.is_none() {
            Some(Arc::new(JoplinNote {
                contents: match &d.body {
                    DecodedMdBody::Unencrypted(s) => s.clone(),
                    DecodedMdBody::Encrypted(_) => String::from("<ENCRYPTED>"),
                },
            }))
        } else {
            None
        }
    }

    pub fn get_contents(&self) -> &str {
        &self.contents
    }
}
