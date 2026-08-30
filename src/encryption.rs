mod generic;

use aes_gcm::Aes256Gcm;
use generic::EncryptedData;
use sha2::Sha512;

pub type EncryptedKeyV1 = EncryptedData<Aes256Gcm, Sha512, 220000>;
pub type EncryptedStringV1 = EncryptedData<Aes256Gcm, Sha512, 3>;
pub const ENCRYPTION_METHOD_KEY_V1: u32 = 8;
pub const ENCRYPTION_METHOD_STRING_V1: u32 = 10;
