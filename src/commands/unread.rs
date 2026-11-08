use crate::args::Args;
use crate::{App, Status};
use std::io;
use std::io::Write;

enum UnreadArg {
    All,
    Index(usize),
    Pop(usize),
    Name(String),
}

pub fn handle_unread(app: &mut App, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let arg = get_arg(args)?;
    let mut indexes = vec_from_arg(app, arg)?;

    // If we remove index i and j with i < j, the indexes will shift.
    // Remove larger indexes first.
    indexes.sort_by(|a, b| b.cmp(a));

    unread(app, indexes)
}

fn unread(app: &mut App, indexes: Vec<usize>) -> Result<(), Box<dyn std::error::Error>> {
    if !indexes.is_sorted_by(|a, b| a > b) {
        return Err("Indexes are unsorted (not user fault).".into());
    }
    for index in indexes {
        let mut e = app.entries.remove(index);
        e.unread();
        let boundary = app
            .entries
            .iter()
            .position(|e| e.status == Status::Unread)
            .unwrap_or(app.entries.len());
        let label = format!("{}({})", e.name, e.section);
        app.entries.insert(boundary, e);
        app.save_to_file()?;
        println!("unread: {}", label);
    }
    Ok(())
}

fn vec_from_arg(app: &App, arg: UnreadArg) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    match arg {
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
                .take(count)
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
    if args.all {
        confirm_action()?;
        return Ok(UnreadArg::All);
    }

    if let Some(n) = args.index {
        return Ok(UnreadArg::Index(n));
    };

    let p = args
        .positional
        .first()
        .ok_or("specify a name, an index, amount to pop or --all")?;

    if let Some(n) = p.parse::<usize>().ok() {
        return Ok(UnreadArg::Pop(n));
    };

    Ok(UnreadArg::Name(p.to_owned()))
}

fn confirm_action() -> Result<(), Box<dyn std::error::Error>> {
    println!("This will remove ALL your reading progress.");
    print!("Do You really want to proceed? [y/N]: ");

    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "yes" | "Y" | "y" => Ok(()),
        _ => Err("Canceled by user".to_string().into()),
    }
}
