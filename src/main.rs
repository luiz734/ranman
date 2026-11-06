use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs::{create_dir_all, read_dir, read_to_string, rename};
use std::io::Error;
use std::path::PathBuf;

fn process_subcommand(cmd: &str, app: App) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        "help" => {
            show_help();
            Ok(())
        }
        "info" => {
            info(app);
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

fn manpath() -> Vec<PathBuf> {
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

fn generate(manpath: Vec<PathBuf>) -> Vec<Page> {
    let mut tree: BTreeSet<String> = BTreeSet::new();

    for dir in manpath {
        // Don't need is dir. Error will handle it

        if dir.is_dir() {
            if let Some(a) = read_dir(dir).ok() {
                for b in a {
                    if let Ok(b) = b {
                        let file_name = b.file_name();
                        // Skip directories .../man/mr
                        if !file_name.to_string_lossy().starts_with("man") {
                            continue;
                        }

                        if b.path().is_dir() {
                            if let Some(c) = read_dir(b.path()).ok() {
                                for d in c {
                                    if let Ok(e) = d {
                                        // Without to_string borrow checker complains
                                        let entry = e.file_name().to_string_lossy().to_string();
                                        if let Some(without_gz) = entry.strip_suffix(".gz") {
                                            tree.insert(without_gz.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut pages: Vec<Page> = vec![];
    for item in &tree {
        let pair = item.rsplit_once(".");
        if let Some(x) = pair {
            let page = Page {
                name: x.0.to_string(),
                section: x.1.to_string(),
            };
            pages.push(page);
        }
    }

    pages
}
struct Page {
    name: String,
    section: String,
}

enum Status {
    Reading,
    Done,
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

struct App {
    file: PathBuf,
    content: String,
    read: String,
    order: Order,
    sections: String,
}

//
fn new_app(state_file: PathBuf, content: String) -> App {
    let header = content
        .lines()
        .filter(|line| line.starts_with("#"))
        .collect::<Vec<&str>>();

    let section = header
        .iter()
        .find(|line| line.starts_with("#ranman"))
        .unwrap_or(&"#ranman 1")
        .split_once(" ")
        .unwrap()
        .1
        .to_string();

    let order = header
        .iter()
        .find(|line| line.starts_with("#order"))
        .unwrap_or(&"#order random");

    let order = match order.split_once(" ") {
        Some(("#order", "random")) => Order::Random,
        Some(("#order", "alphabetical")) => Order::Alphabetical,
        _ => Order::Random,
    };

    let total = content.lines().count() - header.len();
    let lines: Vec<&str> = content.lines().skip(header.len()).collect();
    let done = lines.iter().filter(|line| line.starts_with("D")).count();
    let reading = lines.iter().filter(|line| line.starts_with("R")).count();
    let total_counted = done + reading;
    let read = format!("{total_counted}/{total} ({done} done, {reading} reading)");

    App {
        file: state_file,
        content,
        read,
        order,
        sections: section,
    }
}

fn save(file: &PathBuf, content: &str) -> Result<(), Error> {
    let base_dir = file.as_path().parent().unwrap();
    let tmp_file = base_dir.iter().as_path().join("list.tmp");

    match create_dir_all(base_dir).and_then(|_| std::fs::write(&tmp_file, &content)) {
        Ok(_) => {
            rename(tmp_file, &file)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manpath = manpath();

    // This should never fail under normal conditions
    let state_file = match state_path() {
        Some(f) => f,
        None => {
            eprintln!("ranman: Can't find state_file location.");
            std::process::exit(1);
        }
    };

    let text = match read_to_string(&state_file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let pages = generate(manpath);
            let body = pages.iter().fold(String::new(), |acc, item| {
                format!("{}\n. {}({})", acc, item.name, item.section)
            });
            let mut header = String::new();
            header += "#ranman 1\n";
            header += "#order random";
            header += body.as_str();

            match save(&state_file, &header) {
                Ok(_) => {}
                Err(e) => {return Err(e.into());}
            }

            header
        }
        Err(e) => {
            return Err(e.into());
        }
    };

    let app = new_app(state_file, text);

    let subcommand = std::env::args().nth(1);
    match subcommand {
        Some(cmd) => process_subcommand(&cmd, app),
        None => {
            show_help();
            Ok(())
        }
    }
}

fn info(app: App) {
    let mut s = String::new();
    s += &format!("file: {}\n", app.file.as_path().to_string_lossy());
    s += &format!("order: {}\n", app.order);
    s += &format!("sections: {}\n", app.sections);
    s += &format!("read: {}\n", app.read);

    s += "current: XXXXX\n";
    s += "next: XXXXX\n";
    println!("{}", s);
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
