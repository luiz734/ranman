use std::path::PathBuf;

fn process_subcommand(cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
    match (cmd) {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manpath = manpath();

    for dir in manpath {
        println!("Found manpage directory: {}", dir.display());
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
