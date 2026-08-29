use crate::decoder::DecodedMasterKey;
use crate::error::JoplinError;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::Deserialize;
use std::sync::Mutex;

enum KeyContents {
    Encrypted {
        salt: Vec<u8>,
        iv: Vec<u8>,
        ciphertext: Vec<u8>,
    },
    Decrypted(Vec<u8>),
}

pub struct JoplinMasterKey {
    contents: Mutex<KeyContents>,
}

#[derive(Deserialize)]
struct EncryptedContent {
    salt: String,
    iv: String,
    ct: String,
}

const ENCRYPTION_METHOD_KEY_V1: u32 = 8;

impl JoplinMasterKey {
    pub fn from_decoded(d: &DecodedMasterKey) -> Result<JoplinMasterKey, JoplinError> {
        if d.encryption_method != ENCRYPTION_METHOD_KEY_V1 {
            return Err(JoplinError::Encryption(format!(
                "unsupported encryption type {}",
                d.encryption_method
            )));
        }
        let content: EncryptedContent = serde_json::from_str(&d.content)?;
        let salt = BASE64_STANDARD.decode(content.salt)?;
        let iv = BASE64_STANDARD.decode(content.iv)?;
        let ciphertext = BASE64_STANDARD.decode(content.ct)?;
        Ok(JoplinMasterKey {
            contents: Mutex::new(KeyContents::Encrypted {
                salt,
                iv,
                ciphertext,
            }),
        })
    }
}
