use rand::prelude::SliceRandom;
use regex::Regex;
use std::collections::BTreeSet;
use std::fs::{DirEntry, read_to_string};
use std::path::PathBuf;

mod app;
mod args;
mod commands;

use crate::app::{App, Entry, Status};
use crate::args::{Args, parse_args, valid_sections};
use crate::commands::done::done;
use crate::commands::help::help;
use crate::commands::history::history;
use crate::commands::info::info;
use crate::commands::peek::peek;
use crate::commands::pile::pile;
use crate::commands::start::start;
use crate::commands::unread::handle_unread;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = run() {
        eprintln!("ranman: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // This should never fail under normal conditions
    let state_path = match state_path() {
        Some(f) => f,
        None => {
            eprintln!("can't find state_file location.");
            std::process::exit(1);
        }
    };

    let man_paths = man_paths();
    let sections = vec!["1".to_owned()];

    let args = parse_args()?;

    let mut app = match read_to_string(&state_path) {
        Ok(t) => App::from_text(state_path, t, sections)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            App::generate(state_path, sections, man_paths, args.sorted)?
        }
        Err(e) => {
            return Err(e.into());
        }
    };
    valid_sections(&app, &args.sections)?;

    // Redundant when just read
    app.save_to_file()?;

    process_subcommand(&args, &mut app)?;
    Ok(())
}

fn process_subcommand(args: &Args, app: &mut App) -> Result<()> {
    match args.cmd.as_deref() {
        None => ranman(app),
        Some("help") => help(),
        Some("info") => info(app),
        Some("start") => start(app),
        Some("history") => history(app, args.numbered),
        Some("peek") => peek(app),
        Some("pile") => pile(app, args.numbered),
        Some("unread") => handle_unread(app, args),
        Some("done") => done(app),
        Some(_) => Err("unknown command".into()),
    }
}

fn man_paths() -> Vec<PathBuf> {
    let output = std::process::Command::new("manpath").output();
    let mut paths: Vec<PathBuf> = vec![];

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for path in stdout.trim().split(':') {
                paths.push(PathBuf::from(path));
            }
        }
        Err(_) => {
            let default_paths = String::from("/usr/share/man:/usr/local/share/man");
            eprintln!(
                "Could not find manpage directory. Defaulting to {}",
                default_paths
            );
            for path in default_paths.trim().split(':') {
                paths.push(PathBuf::from(path));
            }
        }
    }
    paths
}

fn generate(man_paths: Vec<PathBuf>, sorted: bool) -> Vec<Entry> {
    let mut tree: BTreeSet<String> = BTreeSet::new();

    let re = Regex::new(r"man\d+$").unwrap();
    let is_man_subdir =
        |d: &DirEntry| d.path().is_dir() && re.is_match(d.path().as_path().to_str().unwrap());

    let re = Regex::new(r"\.(gz|xz|bz2|zst)$").unwrap();
    let is_compressed_page = |d: &DirEntry| re.is_match(d.path().as_path().to_str().unwrap());

    man_paths
        .iter()
        .filter(|p| p.is_dir())
        .flat_map(|p| p.read_dir())
        .for_each(|r| {
            r.flatten()
                .filter(is_man_subdir)
                .map(|d| d.path())
                .flat_map(|p| p.read_dir())
                .for_each(|r| {
                    r.flatten()
                        .filter(is_compressed_page)
                        .map(|d| {
                            String::from(d.path().to_str().unwrap().rsplit_once(".").unwrap().0)
                        })
                        .for_each(|p| {
                            tree.insert(String::from(p));
                        });
                });
        });

    let mut entries: Vec<Entry> = vec![];
    for item in &tree {
        let base_name = PathBuf::from(item)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let pair = base_name.rsplit_once(".");
        if let Some(x) = pair {
            let entry = Entry {
                name: x.0.to_string(),
                section: x.1.to_string(),
                status: Status::Unread,
            };
            entries.push(entry);
        }
    }

    if sorted {
        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    } else {
        entries.shuffle(&mut rand::rng())
    }

    entries
}

fn ranman(app: &mut App) -> Result<()> {
    if let Some(i) = app.current_idx() {
        println!("{} {}", app.entries[i].section, app.entries[i].name);
    } else {
        if let Some(i) = app.next_idx() {
            println!("{} {}", app.entries[i].section, app.entries[i].name);
            eprintln!("not started; run 'ranman start' to mark it as reading");
        } else {
            return Err("no unread pages in the selected sections".into());
        }
    }
    Ok(())
}

fn state_path() -> Option<PathBuf> {
    if let Some(config_env) = std::env::var_os("RANMAN_FILE") {
        return Some(PathBuf::from(config_env));
    }

    let from_home = PathBuf::from(".local/share/ranman/list");

    if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(xdg_data_home).join(PathBuf::from(from_home)));
    }

    if let Some(home) = std::env::var_os("HOME") {
        return Some(PathBuf::from(home).join(PathBuf::from(from_home)));
    }

    None
}
