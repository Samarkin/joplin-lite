use super::JoplinDatabase;
use crate::decoder::{DecodedMd, DecodedMdBody, DecodedMdType};
use crate::encryption::EncryptedStringV1;
use crate::error::JoplinError;
use crate::password::JoplinPasswordProvider;
use std::cell::OnceCell;
use std::sync::{Arc, Mutex};

struct EncryptedData {
    key_id: String,
    cipher_text: Vec<EncryptedStringV1>,
}

pub struct JoplinNote {
    encrypted: Mutex<Option<EncryptedData>>,
    decrypted: OnceCell<String>,
}

impl JoplinNote {
    fn from_encrypted(data: &str) -> Result<JoplinNote, JoplinError> {
        if data.get(0..3) != Some("JED") {
            return Err(JoplinError::Encryption(String::from("malformed encryption header")));
        }
        if data.get(3..5) == Some("01") {
            let Some(length) = data.get(5..11) else {
                return Err(JoplinError::Encryption(String::from("malformed encryption metadata: too short")));
            };
            let Ok(length) = u32::from_str_radix(length, 16) else {
                return Err(JoplinError::Encryption(String::from("malformed encryption metadata: failed to parse length")));
            };
            if length < 3 {
                return Err(JoplinError::Encryption(String::from("malformed encryption metadata: parsed length is too small")));
            }
            let Some(encryption_method) = data.get(11..13) else {
                return Err(JoplinError::Encryption(String::from("malformed encryption metadata: too short")));
            };
            // TODO: Verify encryption method
            let key_id_length = length as usize - 2;
            let Some(key_id) = data.get(13..13 + key_id_length) else {
                return Err(JoplinError::Encryption(String::from("malformed encryption metadata: too short")));
            };
            let key_id = String::from(key_id);
            let mut cipher_text = Vec::new();
            let mut cursor = 13 + key_id_length;
            while let Some(chunk_length) = data.get(cursor..cursor + 6) {
                let Ok(chunk_length) = u32::from_str_radix(chunk_length, 16) else {
                    return Err(JoplinError::Encryption(String::from("malformed encryption chunk: failed to parse length")));
                };
                cursor += 6;
                let Some(chunk_cipher_text) = data.get(cursor..cursor + chunk_length as usize) else {
                    return Err(JoplinError::Encryption(String::from("malformed encryption ciphertext: failed to parse length")));
                };
                cipher_text.push(EncryptedStringV1::from_json(chunk_cipher_text)?);
                cursor += chunk_length as usize;
            }
            Ok(JoplinNote {
                encrypted: Mutex::new(Some(EncryptedData {
                    key_id,
                    cipher_text,
                })),
                decrypted: OnceCell::new(),
            })
        } else {
            Err(JoplinError::Encryption(String::from("unsupported encryption version")))
        }
    }

    fn from_decrypted(body: &str) -> JoplinNote {
        JoplinNote {
            encrypted: Mutex::new(None),
            decrypted: OnceCell::from(String::from(body)),
        }
    }

    pub(crate) fn from_md(d: &DecodedMd) -> Result<Option<Arc<JoplinNote>>, JoplinError> {
        if d.tp == DecodedMdType::Note && d.deleted_time.is_none() {
            let note = match &d.body {
                DecodedMdBody::Unencrypted(s) => Self::from_decrypted(s),
                DecodedMdBody::Encrypted(data) => Self::from_encrypted(data)?
            };
            Ok(Some(Arc::new(note)))
        } else {
            Ok(None)
        }
    }

    fn decrypt<P: JoplinPasswordProvider>(db: &JoplinDatabase<P>, data: &EncryptedData) -> Result<String, JoplinError> {
        let key = db.get_master_key(&data.key_id)?;
        let mut bytes = Vec::new();
        for cipher_text in &data.cipher_text {
            bytes.extend_from_slice(&cipher_text.decrypt(key.as_bytes())?);
        }
        String::from_utf16le(&bytes)
            .map_err(|_| JoplinError::Encryption(String::from("malformed encryption: data is not a valid UTF-16 string")))
    }

    pub fn get_contents<P: JoplinPasswordProvider>(&self, db: &JoplinDatabase<P>) -> Result<&str, JoplinError> {
        let mut encrypted_guard = self.encrypted.lock().unwrap();
        if let Some(s) = self.decrypted.get() {
            return Ok(s);
        }
        // if decrypted is not set, encrypted must be
        let encrypted = encrypted_guard.as_ref().unwrap();
        let decrypted = Self::decrypt(db, encrypted)?;
        *encrypted_guard = None; // encrypted data is not needed anymore, we can save some memory
        Ok(self.decrypted.get_or_init(|| decrypted))
    }
}
