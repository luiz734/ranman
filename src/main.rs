use std::fs::read_dir;
use std::path::PathBuf;

fn process_subcommand(cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        "help" => {
            show_help();
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
struct Page {
    name: String,
    section: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manpath = manpath();
    let mut pages: Vec<Page> = vec![];

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
                                            let pair= without_gz.rsplit_once(".");
                                            if let Some(x) = pair {
                                                let page = Page {
                                                    name: x.0.to_string(),
                                                    section: x.1.to_string(),
                                                };
                                                pages.push(page);
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
    }

    let subcommand = std::env::args().nth(1);
    match subcommand {
        Some(cmd) => process_subcommand(&cmd),
        None => {
            show_help();
            Ok(())
        }
    }
}
