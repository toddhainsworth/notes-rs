extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate open;
extern crate touch;

use chrono::offset::Local;
use chrono::DateTime;

use std::env;
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::process;

use touch::exists;

const NOTES_DIR_NAME: &'static str = ".notes";
const DEFAULT_EDITOR: &'static str = "vim";

fn main() {
    let args: Vec<String> = env::args().collect();

    let notes_dir = get_notes_dir();

    // TODO: Move a bunch of these matches to if-let's
    // Init command
    if args.len() == 2 && args[1] == "init" {
        if !exists(notes_dir.as_str()) {
            if let Err(e) = touch::dir::create(notes_dir.as_str()) {
                eprintln!("Failed to create notes directory: {}", e);
                process::exit(1);
            }
        } else {
            eprintln!("Notes folder already exists");
            process::exit(1)
        }
    // Opening a note (new or existing)
    } else if args.len() == 2 {
        let note_path = get_note_path(args[1].clone());
        let editor = get_editor_from_env();
        let mut cmd = std::process::Command::new(&editor);
        cmd.arg(&note_path);

        match cmd.spawn() {
            Ok(mut ch) => {
                if let Err(e) = ch.wait() {
                    eprintln!("Failed to open note {}: {}", note_path, e);
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Could not spawn editor process \"{}\": {}", editor, e);
                process::exit(1);
            }
        }
    // Otherwise; show the list of notes
    } else {
        let notes_dir = get_notes_dir();
        match std::fs::read_dir(&notes_dir) {
            Ok(dir_entries) => {
                dir_entries.for_each(|e| {
                    // TODO: get rid of this unwrap
                    let en = e.unwrap();

                    if let Ok(created) = get_created_at(&en) {
                        println!("{} - {:?}", created.format("%d/%m/%Y %T"), en.path());
                    }
                });
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Failed to read notes from {}: {}", notes_dir, e);
                process::exit(1);
            }
        }
    }

    process::exit(0);
}

fn get_notes_dir() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/{}", home.display(), Path::new(NOTES_DIR_NAME).display())
}

fn get_note_path(note_name: String) -> String {
    format!("{}/{}", get_notes_dir(), note_name)
}

fn get_created_at(dir_entry: &DirEntry) -> Result<DateTime<Local>, io::Error> {
    let datetime: DateTime<Local> = dir_entry.metadata()?.created()?.into();
    Ok(datetime)
}

fn get_editor_from_env() -> String {
    match env::var("EDITOR") {
        Ok(editor) => editor,
        Err(_) => String::from(DEFAULT_EDITOR),
    }
}
