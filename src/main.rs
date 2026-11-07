use rand::prelude::SliceRandom;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::{DirEntry, create_dir_all, read_dir, read_to_string, rename};
use std::io::Error;
use std::ops::DerefMut;
use std::path::PathBuf;

const APP_VERSION: &str = "1";

fn process_subcommand(cmd: &str, app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        "help" => {
            show_help();
            Ok(())
        }
        "info" => {
            info(app)?;
            Ok(())
        }
        "start" => {
            start(app)?;
            Ok(())
        }
        "done" => {
            done(app)?;
            Ok(())
        }
        &_ => {
            unknown_command();
            Ok(())
        }
    }
}

fn show_help() {
    println!("USAGE: help");
}

fn unknown_command() {
    println!("Unknown command");
    show_help();
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

fn generate(man_paths: Vec<PathBuf>, random: bool) -> Vec<Entry> {
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

    if random {
        entries.shuffle(&mut rand::rng())
    }

    entries
}

#[derive(PartialEq, Eq)]
enum Status {
    Done,
    Reading,
    Unread,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Reading => write!(f, "R"),
            Status::Unread => write!(f, "."),
            Status::Done => write!(f, "D"),
        }
    }
}

enum Order {
    Random,
    Alphabetical,
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::Random => write!(f, "random"),
            Order::Alphabetical => write!(f, "alphabetical"),
        }
    }
}

struct Entry {
    status: Status,
    name: String,
    section: String,
}

impl Entry {
    fn read(&mut self) {
        self.status = Status::Reading;
    }
    fn unread(&mut self) {
        self.status = Status::Unread;
    }
    fn done(&mut self) {
        self.status = Status::Done;
    }
}

struct App {
    path: PathBuf,
    order: Order,
    entries: Vec<Entry>,
    sections: Vec<String>,
}

impl App {
    fn save_to_file(&self) -> Result<(), Error> {
        let mut text = String::new();

        text += &format!("#ranman {}\n", APP_VERSION);
        text += &format!("#order {}\n", self.order.to_string());

        for e in &self.entries {
            text += &format!("{} {}({})\n", e.status, e.name, e.section)
        }

        // Save to disk
        let base_dir = self.path.as_path().parent().unwrap();
        let tmp_file = base_dir.iter().as_path().join("list.tmp");

        match create_dir_all(base_dir).and_then(|_| std::fs::write(&tmp_file, &text)) {
            Ok(_) => {
                rename(tmp_file, &self.path)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn count_status(&self, status: Status) -> usize {
        self.entries
            .iter()
            .filter(|e| self.sections.contains(&e.section))
            .filter(|e| e.status == status)
            .count()
    }

    fn sections_string(&self) -> String {
        self.sections
            .iter()
            .fold(String::new(), |acc, s| acc + s + " ")
            .trim()
            .to_string()
    }

    fn next_idx(&self) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.status == Status::Unread && self.selected(e))
    }

    fn current_idx(&self) -> Option<usize> {
        self.entries
            .iter()
            .rposition(|e| e.status == Status::Reading && self.selected(e))
    }

    fn selected(&self, e: &Entry) -> bool {
        self.sections
            .iter()
            .any(|s| e.section.starts_with(s.as_str()))
    }

    fn generate(
        file: PathBuf,
        sections: Vec<String>,
        man_paths: Vec<PathBuf>,
        random: bool,
    ) -> Result<App, Error> {
        let entries = generate(man_paths, random);

        let order = match random {
            true => Order::Random,
            false => Order::Alphabetical,
        };

        Ok(App {
            path: file,
            order,
            entries,
            sections,
        })
    }

    fn from_text(file: PathBuf, text: String, sections: Vec<String>) -> Result<App, Error> {
        let order = text
            .lines()
            .find(|line| line.starts_with("#order"))
            .unwrap_or(&"#order random");
        let order = match order.split_once(" ") {
            Some(("#order", "random")) => Order::Random,
            Some(("#order", "alphabetical")) => Order::Alphabetical,
            _ => Order::Random,
        };

        let entries = text
            .lines()
            .filter(|l| !l.starts_with("#") && !l.is_empty())
            .map(|line| {
                let pair = line.split_once(" ").unwrap();

                let status = match pair.0 {
                    "R" => Status::Reading,
                    "D" => Status::Done,
                    _ => Status::Unread,
                };

                let pair = pair.1.rsplit_once("(").unwrap();
                let name = pair.0.to_string();
                let section = pair.1.strip_suffix(")").unwrap().to_string();

                Entry {
                    status,
                    name,
                    section,
                }
            });

        Ok(App {
            path: file,
            order,
            entries: entries.collect(),
            sections,
        })
    }
}

#[derive(Debug, Clone)]
enum ParsingError {
    EmptyLine,
}
impl Display for ParsingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParsingError::EmptyLine => write!(f, "State file contains empty line"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This should never fail under normal conditions
    let state_path = match state_path() {
        Some(f) => f,
        None => {
            eprintln!("ranman: Can't find state_file location.");
            std::process::exit(1);
        }
    };

    let man_paths = man_paths();

    let sections = vec!["1".to_owned()];

    let mut app = match read_to_string(&state_path) {
        Ok(t) => App::from_text(state_path, t, sections)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            App::generate(state_path, sections, man_paths, true)?
        }
        Err(e) => {
            return Err(e.into());
        }
    };
    // Redundant when just read
    app.save_to_file()?;

    let subcommand = std::env::args().nth(1);
    match subcommand {
        Some(cmd) => process_subcommand(&cmd, &mut app),
        None => ranman(&mut app),
    }
}

#[derive(Default)]
struct SectionInfo {
    total: usize,
    read: usize,
}

impl Display for SectionInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.read, self.total)
    }
}

fn info(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let mut s = String::new();
    s += &format!("file: {}\n", app.path.as_path().to_string_lossy());
    s += &format!("order: {}\n", app.order);
    s += &format!("sections: {}\n", app.sections_string());

    let done = app.count_status(Status::Done);
    let reading = app.count_status(Status::Reading);
    let unread = app.count_status(Status::Unread);
    let total = done + reading + unread;
    s += &format!(
        "read: {}/{} ({} done, {} reading)\n",
        done + reading,
        total,
        done,
        reading
    );

    if let Some(i) = app.current_idx() {
        s += &format!(
            "current: {}({})\n",
            app.entries[i].name, app.entries[i].section
        );
    } else {
        s += "current: -\n";
    }

    if let Some(i) = app.next_idx() {
        s += &format!(
            "next: {}({})\n",
            app.entries[i].name, app.entries[i].section
        );
    } else {
        s += "next: (no next)\n";
    }

    let mut by_section: BTreeMap<&str, SectionInfo> = BTreeMap::new();
    for e in &app.entries {
        let s = by_section.entry(e.section.as_str()).or_default();
        s.total += 1;
        if e.status != Status::Unread {
            s.read += 1;
        }
    }

    s += "by section (* = selected):\n";
    for (k, v) in by_section {
        s += &format!("  {k} {v}\n")
    }

    println!("{}", s);

    Ok(())
}

fn start(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let i = app
        .next_idx()
        .ok_or("no unread pages in the selected sections")?;
    let mut e = app.entries.remove(i);
    e.read();
    let boundary = app
        .entries
        .iter()
        .position(|e| e.status == Status::Unread)
        .unwrap_or(app.entries.len());
    println!("{} {}", e.section, e.name);
    app.entries.insert(boundary, e);
    Ok(app.save_to_file()?)
}
fn done(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let i = app.current_idx().ok_or("nothing in progress")?;
    app.entries[i].done();
    Ok(app.save_to_file()?)
}

fn ranman(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(i) = app.current_idx() {
        println!("{} {}", app.entries[i].section, app.entries[i].name);
    } else {
        if let Some(i) = app.next_idx() {
            print!("{} {}", app.entries[i].section, app.entries[i].name);
            eprintln!("(not started)")
        } else {
            println!("Nothing to read!")
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
