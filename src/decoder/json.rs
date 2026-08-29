use crate::error::JoplinError;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize)]
pub struct DecodedMasterKey {
    pub encryption_method: u32,
    pub id: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct DecodedJson {
    #[serde(rename = "masterKeys", default)]
    pub master_keys: Vec<DecodedMasterKey>,
}

impl DecodedJson {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<DecodedJson, JoplinError> {
        Ok(serde_json::from_reader(File::open(path)?)?)
    }

    #[cfg(test)]
    pub fn from_str(s: &str) -> Result<DecodedJson, JoplinError> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::decoder::DecodedJson;

    #[test]
    fn test_simple_deserialization() {
        let json = DecodedJson::from_str(
            r##"{
                "masterKeys": [
                    {
                        "encryption_method": 7,
                        "id": "main-key",
                        "content": "CIPHERTEXT"
                    }
                ],
                "some_other_field": "With other value"
            }"##,
        )
            .unwrap();
        assert_eq!(json.master_keys.len(), 1);
        assert_eq!(json.master_keys[0].encryption_method, 7);
        assert_eq!(json.master_keys[0].id, "main-key");
        assert_eq!(json.master_keys[0].content, "CIPHERTEXT");
    }

    #[test]
    fn test_no_keys() {
        let json = DecodedJson::from_str(
            r##"{
                "masterKeys": [],
                "some_other_field": "With other value"
            }"##,
        )
            .unwrap();
        assert_eq!(json.master_keys.len(), 0);
    }

    #[test]
    fn test_missing_field() {
        let json = DecodedJson::from_str(
            r##"{
                "some_other_field": "With other value"
            }"##,
        )
            .unwrap();
        assert_eq!(json.master_keys.len(), 0);
    }

    #[test]
    fn test_empty_json() {
        let json = DecodedJson::from_str("{}").unwrap();
        assert_eq!(json.master_keys.len(), 0);
    }
}
