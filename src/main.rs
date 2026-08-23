mod error;
mod decoder;

use std::collections::HashMap;
use error::JoplinError;
use std::ffi::OsStr;
use std::{env, fs};
use crate::decoder::{DecodedNode, DecodedNodeType};

enum TreeNode {
    Root{children: Vec<TreeNode>},
    Note{contents: String},
    Folder{name: String, children: Vec<TreeNode>},
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
            },
            TreeNode::Note { contents } => {
                let first_line = contents.lines().next().unwrap_or("");
                println!("{}{}", prefix, first_line);
            },
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
        children: HashMap<Option<String>, Vec<DecodedNode>>,
    }

    impl Cache {
        fn get_children(&self, parent_id: &Option<String>) -> &Vec<DecodedNode> {
            static EMPTY_VEC: Vec<DecodedNode> = Vec::new();
            self.children.get(parent_id).unwrap_or(&EMPTY_VEC)
        }

        fn get_children_mut(&mut self, parent_id: &Option<String>) -> &mut Vec<DecodedNode> {
            self.children.entry(parent_id.clone()).or_insert_with(Vec::new)
        }

        fn get_tree_node(&self, d: &DecodedNode) -> TreeNode {
            match d.tp {
                DecodedNodeType::Note => TreeNode::Note { contents: d.contents.clone() },
                DecodedNodeType::Folder => TreeNode::Folder {
                    name: d.contents.clone(),
                    children: self.get_children(&Some(d.id.clone())).iter().map(|child| self.get_tree_node(child)).collect(),
                },
            }
        }
    }

    let mut cache = Cache{children: HashMap::new()};

    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension() ==  Some(OsStr::new("md")) {
                match decoder::decode_file(&path) {
                    Ok(decoded_node) => {
                        cache.get_children_mut(&decoded_node.parent_id).push(decoded_node);
                    },
                    Err(err) => {
                        println!("Error reading {}: {}", path.to_string_lossy(), err);
                    },
                }
            }
        }
    }

    let tree = TreeNode::Root {
        children: cache.get_children(&None).iter()
            .map(|d| cache.get_tree_node(d))
            .collect(),
    };

    tree.print("");

    Ok(())
}
