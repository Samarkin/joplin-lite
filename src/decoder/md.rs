use crate::error::JoplinError;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub enum DecodedMdType {
    Note,
    Folder,
}

#[derive(Debug)]
pub enum DecodedMdBody {
    Unencrypted(String),
    Encrypted(String),
}

pub struct DecodedMd {
    pub body: DecodedMdBody,
    pub tp: DecodedMdType,
    pub id: String,
    pub parent_id: Option<String>,
}

impl DecodedMd {
    pub fn from_file(path: &PathBuf) -> Result<DecodedMd, JoplinError> {
        let s = fs::read_to_string(path)?;
        let id = path.file_stem().unwrap().to_string_lossy();
        Self::from_string(&s, &id)
    }

    pub fn from_string(s: &str, filename: &str) -> Result<DecodedMd, JoplinError> {
        let mut body = vec![];
        let lines = s.lines();
        let mut it = lines.rev();
        let mut tp = None;
        let mut id = None;
        let mut parent_id = None;
        let mut encryption_applied = false;
        let mut encryption_cipher_text = None;

        while let Some(line) = it.next() {
            if line.is_empty() {
                while let Some(line) = it.next() {
                    body.push(line);
                }
                body.reverse();
                break;
            }
            let Some(idx) = line.find(":") else {
                return Err(JoplinError::Decode(String::from("invalid property format")));
            };
            let key = line[..idx].trim();
            let value = line[idx + 1..].trim();
            if key == "type_" {
                match u32::from_str(value) {
                    Ok(1) => tp = Some(DecodedMdType::Note),
                    Ok(2) => tp = Some(DecodedMdType::Folder),
                    Ok(n) => {
                        return Err(JoplinError::Decode(format!("unsupported md type: {}", n)));
                    }
                    Err(err) => {
                        return Err(JoplinError::Decode(format!(
                            "failed to parse md type: {}",
                            err
                        )));
                    }
                };
            } else if key == "id" {
                if value == filename {
                    id = Some(value.to_string());
                } else {
                    return Err(JoplinError::Decode(format!(
                        "mismatched id: {} instead of {}",
                        value, filename
                    )));
                }
            } else if key == "parent_id" {
                if !value.is_empty() {
                    parent_id = Some(value.to_string());
                }
            } else if key == "encryption_applied" {
                if value == "1" {
                    encryption_applied = true;
                }
            } else if key == "encryption_cipher_text" {
                if !value.is_empty() {
                    encryption_cipher_text = Some(value.to_string());
                }
            } else {
                warn!("Unsupported property: {}", key);
            }
        }

        let Some(tp) = tp else {
            return Err(JoplinError::Decode(String::from("missing node type")));
        };

        let Some(id) = id else {
            return Err(JoplinError::Decode(String::from("missing id")));
        };

        let body = if encryption_applied {
            let Some(encryption_cipher_text) = encryption_cipher_text else {
                return Err(JoplinError::Decode(String::from(
                    "malformed encryption: missing cipher text",
                )));
            };
            if !body.is_empty() {
                return Err(JoplinError::Decode(String::from(
                    "malformed encryption: unexpected body",
                )));
            }
            DecodedMdBody::Encrypted(encryption_cipher_text)
        } else {
            DecodedMdBody::Unencrypted(body.join("\n"))
        };

        Ok(DecodedMd {
            body,
            id,
            parent_id,
            tp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn test_decode() {
        let md = DecodedMd::from_string(
            r"Hello, world!
This is my note

id: 51751e55e0f1be9c63b15aff2b3ee9f9
parent_id: ff2b800a09d1268748273a24017b99e6
unsupported_field: unsupported_value
type_: 1",
            "51751e55e0f1be9c63b15aff2b3ee9f9",
        );

        let md = md.unwrap();
        assert_matches!(md.body, DecodedMdBody::Unencrypted(s) if s == "Hello, world!\nThis is my note");
    }

    #[test]
    fn test_decode_encrypted() {
        let md = DecodedMd::from_string(
            r"id: 51751e55e0f1be9c63b15aff2b3ee9f9
encryption_cipher_text: JED_ENCRYPTED_BYTES
parent_id: ff2b800a09d1268748273a24017b99e6
encryption_applied: 1
type_: 1",
            "51751e55e0f1be9c63b15aff2b3ee9f9",
        );

        let md = md.unwrap();
        assert_matches!(md.body, DecodedMdBody::Encrypted(s) if s == "JED_ENCRYPTED_BYTES");
    }
}
