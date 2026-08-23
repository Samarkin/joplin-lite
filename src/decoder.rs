use crate::error::JoplinError;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub enum DecodedNodeType {
    Note,
    Folder,
}

pub struct DecodedNode {
    pub contents: String,
    pub tp: DecodedNodeType,
    pub id: String,
    pub parent_id: Option<String>,
    pub props: HashMap<String, String>,
}

pub fn decode_file(path: &PathBuf) -> Result<DecodedNode, JoplinError>  {
    let s = fs::read_to_string(path)?;
    let mut contents = vec![];
    let lines = s.lines();
    let mut it = lines.rev();
    let mut props = HashMap::new();
    let mut tp = None;
    let mut id = None;
    let mut parent_id = None;
    let mut encryption_applied = false;

    while let Some(line) = it.next() {
        if line.is_empty() {
            while let Some(line) = it.next() {
                contents.push(line);
            }
            contents.reverse();
            break;
        }
        let Some(idx) = line.find(":") else {
            return Err(JoplinError::Decode(String::from("invalid property format")));
        };
        let key = line[..idx].trim();
        let value = line[idx + 1..].trim();
        if key == "type_" {
            match u32::from_str(value) {
                Ok(1) => tp = Some(DecodedNodeType::Note),
                Ok(2) => tp = Some(DecodedNodeType::Folder),
                Ok(n) => return Err(JoplinError::Decode(format!("unsupported node type: {}", n))),
                Err(err) => return Err(JoplinError::Decode(format!("failed to parse node type: {}", err))),
            };
        } else if key == "id" {
            if value == path.file_stem().unwrap().to_string_lossy() {
                id = Some(value.to_string());
            } else {
                return Err(JoplinError::Decode(format!("mismatched node id: {}", value)));
            }
        } else if key == "parent_id" {
            if !value.is_empty() {
                parent_id = Some(value.to_string());
            }
        } else if key == "encryption_applied" {
            if value == "1" {
                encryption_applied = true;
            }
        } else {
            props.insert(key.to_string(), value.to_string());
        }
    }

    let Some(tp) = tp else {
        return Err(JoplinError::Decode(String::from("missing node type")));
    };

    let Some(id) = id else {
        return Err(JoplinError::Decode(String::from("missing id")));
    };

    let contents = if encryption_applied {
        if !contents.is_empty() {
            return Err(JoplinError::Decode(String::from("malformed encryption")));
        }
        String::from("<ENCRYPTED>")
    } else {
        contents.join("\n")
    };

    Ok(DecodedNode{ contents, id, parent_id, tp, props })
}
