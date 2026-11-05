fn process_subcommand(cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
    match (cmd) {
        "help" => { show_help(); Ok(()) },
        &_ => { unknown_command(); Ok(()) }
    }
}

fn show_help() {
    println!("USAGE: help");
}

fn unknown_command() {
    println!("Unknown command");
    show_help();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subcommand = std::env::args().nth(1);
    match subcommand {
        Some(cmd) => process_subcommand(&cmd),
        None => {
            show_help();
            Ok(())
        },
    }
}
