use crate::args::Args;
use crate::{App, Status};
use std::io;
use std::io::Write;

#[derive(Debug, PartialEq)]
enum UnreadArg {
    All,
    Index(usize),
    Pop(usize),
    Name(String),
}

pub fn handle_unread(app: &mut App, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let arg = get_arg(args)?;
    let mut indexes = vec_from_arg(app, &arg)?;

    if indexes.is_empty() {
        return Err("no candidates to unread".to_owned().into());
    }

    if arg == UnreadArg::All {
        confirm_action()?;
    }

    unread(app, &mut indexes)
}

fn unread(app: &mut App, indexes: &mut Vec<usize>) -> Result<(), Box<dyn std::error::Error>> {
    // If we remove index i and j with i < j, the indexes will shift.
    // Remove larger indexes first.
    indexes.sort_by(|a, b| b.cmp(a));

    for index in indexes {
        let mut e = app.entries.remove(*index);
        e.unread();
        let boundary = app
            .entries
            .iter()
            .position(|e| e.status == Status::Unread)
            .unwrap_or(app.entries.len());
        let label = format!("{}({})", e.name, e.section);
        app.entries.insert(boundary, e);
        println!("unread: {}", label);
    }
    app.save_to_file()?;
    Ok(())
}

fn vec_from_arg(app: &App, arg: &UnreadArg) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    match arg {
        // We don't need to call unread for this
        // Keep it for consistency only
        UnreadArg::All => Ok(app
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.status != Status::Unread)
            .map(|(i, _)| i)
            .collect::<Vec<usize>>()),

        UnreadArg::Index(n) => {
            let i = n.checked_sub(1).ok_or("indices start at 1")?;
            match app.entries.get(i) {
                Some(e) if e.status != Status::Unread => Ok(vec![i]),
                _ => Err(format!("no such index: {n} (see 'ranman history -n')").into()),
            }
        }

        UnreadArg::Pop(count) => {
            let v = app
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.status != Status::Unread)
                .rev()
                .take(*count)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            Ok(v)
        }

        UnreadArg::Name(a) => {
            let (name, sec) = match a.rsplit_once('(') {
                Some((n, s)) => (n, s.strip_suffix(')')),
                None => (a.as_str(), None),
            };

            match app
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.status != Status::Unread && e.name == name)
                .filter(|(_, e)| sec.is_none_or(|s| e.section == s))
                .min_by_key(|(_, e)| e.section.as_str())
            {
                Some((i, _)) => Ok(vec![i]),
                None => Err(format!("not in the read list: {a}").into()),
            }
        }
    }
}

fn get_arg(args: &Args) -> Result<UnreadArg, Box<dyn std::error::Error>> {
    let mut provided: Vec<UnreadArg> = vec![];

    if args.all {
        provided.push(UnreadArg::All);
    }

    if let Some(n) = args.index {
        provided.push(UnreadArg::Index(n));
    };

    if let Some(p) = args.positional.first() {
        if let Ok(n) = p.parse::<usize>() {
            provided.push(UnreadArg::Pop(n));
        } else {
            provided.push(UnreadArg::Name(p.to_owned()));
        }
    }

    match provided.len() {
        0 => Err("specify a name, an index, amount to pop or --all"
            .to_string()
            .into()),
        1 => Ok(provided.into_iter().next().unwrap()),
        _ => Err("invalid combination".to_string().into()),
    }
}

fn confirm_action() -> Result<(), Box<dyn std::error::Error>> {
    println!("This will remove ALL your reading progress.");
    print!("Do You really want to proceed? [y/N]: ");

    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "yes" | "y" => Ok(()),
        _ => Err("Canceled by user".to_string().into()),
    }
}
