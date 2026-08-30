use crate::decoder::DecodedMasterKey;
use crate::encryption::{ENCRYPTION_METHOD_KEY_V1, EncryptedKeyV1};
use crate::error::JoplinError;
use crate::password::JoplinPasswordProvider;
use std::sync::Mutex;

enum KeyContents {
    EncryptedV1(EncryptedKeyV1),
    Decrypted(Vec<u8>),
}

pub struct JoplinMasterKey {
    id: String,
    contents: Mutex<KeyContents>,
}

impl JoplinMasterKey {
    pub fn from_decoded(d: &DecodedMasterKey) -> Result<JoplinMasterKey, JoplinError> {
        if d.encryption_method == ENCRYPTION_METHOD_KEY_V1 {
            Ok(JoplinMasterKey {
                id: d.id.clone(),
                contents: Mutex::new(KeyContents::EncryptedV1(EncryptedKeyV1::from_json(
                    &d.content,
                )?)),
            })
        } else {
            Err(JoplinError::Encryption(format!(
                "unsupported encryption type {}",
                d.encryption_method
            )))
        }
    }

    pub fn get_key<P: JoplinPasswordProvider>(
        &self,
        password_provider: &mut P,
    ) -> Result<Vec<u8>, JoplinError> {
        let mut contents = self.contents.lock().unwrap();
        match &*contents {
            KeyContents::EncryptedV1(key) => {
                let password = password_provider.get_password(&self.id)?;
                let decrypted = key.decrypt(password.as_bytes())?;
                *contents = KeyContents::Decrypted(decrypted.clone());
                Ok(decrypted)
            }
            KeyContents::Decrypted(d) => Ok(d.clone()),
        }
    }
}
