mod decoder;
mod error;

use crate::decoder::{DecodedMd, DecodedMdBody, DecodedMdType};
use error::JoplinError;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::{env, fs};

#[macro_use]
extern crate log;

enum TreeNode {
    Root {
        children: Vec<TreeNode>,
    },
    Note {
        contents: String,
    },
    Folder {
        name: String,
        children: Vec<TreeNode>,
    },
}

impl TreeNode {
    fn print(&self, prefix: &str) {
        let next_prefix = format!("{}  ", prefix);
        match self {
            TreeNode::Root { children } => {
                println!("{}Root:", prefix);
                for child in children {
                    child.print(&next_prefix);
                }
            }
            TreeNode::Note { contents } => {
                let first_line = contents.lines().next().unwrap_or("");
                println!("{}{}", prefix, first_line);
            }
            TreeNode::Folder { name, children } => {
                println!("{}{}:", prefix, name);
                for child in children {
                    child.print(&next_prefix);
                }
            }
        }
    }
}

fn main() -> Result<(), JoplinError> {
    println!("Welcome to Joplin lite!");

    let mut args = env::args_os();
    let Some(path) = args.nth(1) else {
        return Err(JoplinError::Usage);
    };

    struct Cache {
        children: HashMap<Option<String>, Vec<DecodedMd>>,
    }

    impl Cache {
        fn get_children(&self, parent_id: &Option<String>) -> &Vec<DecodedMd> {
            static EMPTY_VEC: Vec<DecodedMd> = Vec::new();
            self.children.get(parent_id).unwrap_or(&EMPTY_VEC)
        }

        fn get_children_mut(&mut self, parent_id: &Option<String>) -> &mut Vec<DecodedMd> {
            self.children
                .entry(parent_id.clone())
                .or_insert_with(Vec::new)
        }

        fn get_tree_node(&self, d: &DecodedMd) -> TreeNode {
            let body = match &d.body {
                DecodedMdBody::Unencrypted(s) => s.clone(),
                DecodedMdBody::Encrypted(_) => String::from("<ENCRYPTED>"),
            };
            match d.tp {
                DecodedMdType::Note => TreeNode::Note { contents: body },
                DecodedMdType::Folder => TreeNode::Folder {
                    name: body,
                    children: self
                        .get_children(&Some(d.id.clone()))
                        .iter()
                        .map(|child| self.get_tree_node(child))
                        .collect(),
                },
            }
        }
    }

    let mut cache = Cache {
        children: HashMap::new(),
    };

    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("md")) {
                match DecodedMd::from_file(&path) {
                    Ok(decoded_node) => {
                        cache
                            .get_children_mut(&decoded_node.parent_id)
                            .push(decoded_node);
                    }
                    Err(err) => {
                        println!("Error reading {}: {}", path.to_string_lossy(), err);
                    }
                }
            }
        }
    }

    let tree = TreeNode::Root {
        children: cache
            .get_children(&None)
            .iter()
            .map(|d| cache.get_tree_node(d))
            .collect(),
    };

    tree.print("");

    Ok(())
}
