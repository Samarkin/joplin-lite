mod database;
mod decoder;
mod error;

use crate::database::JoplinNotebook;
use database::JoplinDatabase;
use error::JoplinError;
use std::env;

#[macro_use]
extern crate log;

fn print(db: &JoplinDatabase) {
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

fn main() -> Result<(), JoplinError> {
    println!("Welcome to Joplin lite!");

    let mut args = env::args_os();
    let Some(path) = args.nth(1) else {
        return Err(JoplinError::Usage);
    };

    let db = JoplinDatabase::from_path(&path)?;
    println!("Contents of {}:", path.to_string_lossy());
    print(&db);

    Ok(())
}
