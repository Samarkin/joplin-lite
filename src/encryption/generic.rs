use crate::error::JoplinError;
use aes_gcm::aead::array::typenum::Unsigned;
use aes_gcm::aead::{Aead, Key, KeyInit, KeySizeUser, Nonce};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use pbkdf2::hmac::EagerHash;
use pbkdf2::pbkdf2_hmac;
use serde::Deserialize;
use std::marker::PhantomData;

pub struct EncryptedData<
    Algo: Aead + KeyInit + KeySizeUser,
    Hasher: EagerHash,
    const ITERATIONS: u32,
> {
    salt: Vec<u8>,
    iv: Nonce<Algo>,
    ciphertext: Vec<u8>,
    _phantom: PhantomData<Hasher>,
}

#[derive(Deserialize)]
struct EncryptedContent {
    salt: String,
    iv: String,
    ct: String,
}

impl<Algo: Aead + KeyInit + KeySizeUser, Hasher: EagerHash, const ITERATIONS: u32>
    EncryptedData<Algo, Hasher, ITERATIONS>
{
    pub fn from_json(json: &str) -> Result<Self, JoplinError> {
        let content: EncryptedContent = serde_json::from_str(json)?;
        let salt = BASE64_STANDARD.decode(content.salt)?;
        let iv = BASE64_STANDARD.decode(content.iv)?;
        let iv = Nonce::<Algo>::try_from(iv.as_slice())
            .map_err(|_| JoplinError::Encryption(String::from("invalid iv length")))?;
        let ciphertext = BASE64_STANDARD.decode(content.ct)?;
        if ciphertext.len() < Algo::TagSize::USIZE {
            return Err(JoplinError::Encryption(String::from(
                "invalid ciphertext length",
            )));
        }
        Ok(EncryptedData {
            salt,
            iv,
            ciphertext,
            _phantom: PhantomData,
        })
    }

    pub fn decrypt(&self, password: &[u8]) -> Result<Vec<u8>, JoplinError> {
        let mut key = Key::<Algo>::default();
        pbkdf2_hmac::<Hasher>(password, self.salt.as_slice(), ITERATIONS, &mut key);
        let cipher = Algo::new(&key);
        let decrypted = cipher.decrypt(&self.iv, self.ciphertext.as_slice())?;
        Ok(decrypted)
    }
}
