mod database;
mod decoder;
mod encryption;
mod error;
mod password;

use crate::password::JoplinPasswordProvider;
use database::{JoplinDatabase, JoplinNotebook};
use error::JoplinError;
use std::collections::HashMap;
use std::env;

#[macro_use]
extern crate log;

fn print<P: JoplinPasswordProvider>(db: &JoplinDatabase<P>) {
    fn print_internal(notebook: &JoplinNotebook, prefix: &str) {
        println!("{}{}:", prefix, notebook.get_title());
        let next_prefix = format!("{}  ", prefix);
        for subnotebook in notebook.get_notebooks() {
            print_internal(subnotebook, &next_prefix);
        }
        for note in notebook.get_notes() {
            let first_line = note.get_contents().lines().next().unwrap_or("");
            println!("{}{}", next_prefix, first_line);
        }
    }
    for notebook in db.get_notebooks() {
        print_internal(notebook, "  ");
    }
}

struct ConsolePasswordProvider {
    known_passwords: HashMap<String, String>,
}

impl ConsolePasswordProvider {
    fn new() -> ConsolePasswordProvider {
        ConsolePasswordProvider {
            known_passwords: HashMap::new(),
        }
    }
}

impl JoplinPasswordProvider for ConsolePasswordProvider {
    fn get_password(&mut self, key_id: &str) -> Result<String, JoplinError> {
        if let Some(password) = self.known_passwords.get(key_id) {
            Ok(String::from(password))
        } else {
            let password =
                rpassword::prompt_password(format!("Enter password for key {}:", key_id))?;
            self.known_passwords
                .insert(String::from(key_id), password.clone());
            Ok(password)
        }
    }
}

fn main() -> Result<(), JoplinError> {
    println!("Welcome to Joplin lite!");

    let mut args = env::args_os();
    let Some(path) = args.nth(1) else {
        return Err(JoplinError::Usage);
    };

    let db = JoplinDatabase::from_path_with_password(&path, ConsolePasswordProvider::new())?;
    println!("Contents of {}:", path.to_string_lossy());
    print(&db);

    Ok(())
}
