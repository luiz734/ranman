use crate::args::Args;
use crate::{App, Result, Status};
use regex::Regex;
use std::io;
use std::io::Write;

#[derive(Debug, PartialEq)]
enum UnreadArg {
    All,
    Index(usize),
    Pop(usize),
    Name(String),
}

pub(crate) fn handle_unread(app: &mut App, args: &Args) -> Result<()> {
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

fn unread(app: &mut App, indexes: &mut Vec<usize>) -> Result<()> {
    // If we remove index i and j with i < j, the indexes will shift.
    // Remove larger indexes first.
    indexes.sort_unstable_by(|a, b| b.cmp(a));

    for index in indexes {
        let mut entry = app.entries.remove(*index);
        entry.unread();
        let boundary = app
            .entries
            .iter()
            .position(|e| e.status == Status::Unread)
            .unwrap_or(app.entries.len());
        println!("unread: {}({})", entry.name, entry.section);
        app.entries.insert(boundary, entry);
    }
    app.save_to_file()?;
    Ok(())
}

fn vec_from_arg(app: &App, arg: &UnreadArg) -> Result<Vec<usize>> {
    match arg {
        // We don't need to call unread for this
        // Keep it for consistency only
        UnreadArg::All => Ok(app
            .entries_enum()
            .filter(|(_, e)| e.read_or_done())
            .map(|(i, _)| i)
            .collect()),

        UnreadArg::Index(n) => {
            let i = n.checked_sub(1).ok_or("indices start at 1")?;
            match app.entries.get(i) {
                Some(e) if e.read_or_done() => Ok(vec![i]),
                _ => Err(format!("no such index: {n} (see 'ranman history -n')").into()),
            }
        }

        UnreadArg::Pop(count) => Ok(app
            .entries_enum()
            .filter(|(_, e)| e.read_or_done())
            .rev()
            .take(*count)
            .map(|(i, _)| i)
            .collect()),

        UnreadArg::Name(a) => {
            let re = Regex::new(r"([\w\-_:.]+)(\(\w+\))?")?;
            let caps = re.captures(a).ok_or("no match")?;

            let name = caps.get(1).map_or(Some(a.as_str()), |m| Some(m.as_str()));
            let section = caps.get(2).map_or(None, |m| Some(m.as_str()));

            app.entries_enum()
                .filter(|(_, e)| e.read_or_done() && e.name == name.unwrap())
                .filter(|(_, e)| section.is_none_or(|s| e.section == s))
                .min_by_key(|(_, e)| e.section.as_str())
                .map(|(i, _)| vec![i])
                .ok_or_else(|| format!("not in the read list: {a}").into())
        }
    }
}

fn get_arg(args: &Args) -> Result<UnreadArg> {
    let mut provided: Vec<UnreadArg> = vec![];

    if args.all {
        provided.push(UnreadArg::All);
    }

    if let Some(n) = args.index {
        provided.push(UnreadArg::Index(n));
    }

    if let Some(p) = args.positional.first() {
        if let Ok(n) = p.parse::<usize>() {
            provided.push(UnreadArg::Pop(n));
        } else {
            provided.push(UnreadArg::Name(p.clone()));
        }
    }

    match provided.len() {
        0 => Err("specify a name, an index, amount to pop or --all"
            .to_string()
            .into()),
        1 => Ok(provided.into_iter().next().unwrap()),
        _ => Err("invalid combination".into()),
    }
}

fn confirm_action() -> Result<()> {
    println!("This will remove ALL your reading progress.");
    print!("Do You really want to proceed? [y/N]: ");

    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "yes" | "y" => Ok(()),
        _ => Err("canceled by user".into()),
    }
}
