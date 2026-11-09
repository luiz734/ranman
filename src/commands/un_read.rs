use crate::app::Entry;
use crate::args::Args;
use crate::{App, Result, Status};
use regex::Regex;
use std::io::{self, Write};

#[derive(Debug, PartialEq)]
enum UnreadArg {
    All,
    Index(usize),
    Pop(usize),
    Name(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Action {
    Read,
    Unread,
}

impl Action {
    fn applies_to(self, e: &Entry) -> bool {
        match self {
            Action::Read => e.status == Status::Unread,
            Action::Unread => e.status != Status::Unread,
        }
    }

    fn verb(self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Unread => "unread",
        }
    }

    fn pool(self) -> &'static str {
        match self {
            Action::Read => "unread",
            Action::Unread => "read",
        }
    }
}

pub(crate) fn read_or_unread(app: &mut App, args: &Args, action: Action) -> Result<()> {
    let arg = get_arg(args)?;
    let mut indexes = vec_from_arg(app, &arg, action)?;

    if indexes.is_empty() {
        return Err(format!("nothing to {}", action.verb()).into());
    }

    if arg == UnreadArg::All {
        confirm_action(action)?;
    }

    match action {
        Action::Read => read(app, &mut indexes),
        Action::Unread => unread(app, &mut indexes),
    }
}

fn insert_at_boundary(app: &mut App, entry: Entry) {
    let boundary = app
        .entries
        .iter()
        .position(|e| e.status == Status::Unread)
        .unwrap_or(app.entries.len());
    app.entries.insert(boundary, entry);
}

fn read(app: &mut App, indexes: &mut [usize]) -> Result<()> {
    // If we remove index i and j with i < j, the indexes will shift.
    // Remove larger indexes first.
    indexes.sort_unstable();

    for &index in indexes.iter() {
        let mut entry = app.entries.remove(index);
        entry.done();
        println!("read:   {}({})", entry.name, entry.section);
        insert_at_boundary(app, entry);
    }
    app.save_to_file()?;
    Ok(())
}

fn unread(app: &mut App, indexes: &mut [usize]) -> Result<()> {
    indexes.sort_unstable_by(|a, b| b.cmp(a));

    for &index in indexes.iter() {
        let mut entry = app.entries.remove(index);
        entry.unread();
        println!("unread: {}({})", entry.name, entry.section);
        insert_at_boundary(app, entry);
    }
    app.save_to_file()?;
    Ok(())
}

fn vec_from_arg(app: &App, arg: &UnreadArg, action: Action) -> Result<Vec<usize>> {
    match arg {
        // We don't need to call unread for this
        // Keep it for consistency only
        UnreadArg::All => Ok(app
            .entries_enum()
            .filter(|(_, e)| action.applies_to(e))
            .filter(|(_, e)| action == Action::Unread || app.selected(e))
            .map(|(i, _)| i)
            .collect()),

        UnreadArg::Index(n) => {
            if action == Action::Read {
                return Err("--index only works with unread (see 'ranman history -n')".into());
            }
            let i = n.checked_sub(1).ok_or("indices start at 1")?;
            match app.entries.get(i) {
                Some(e) if action.applies_to(e) => Ok(vec![i]),
                _ => Err(format!("no such index: {n} (see 'ranman history -n')").into()),
            }
        }

        UnreadArg::Pop(count) => {
            let candidates = app.entries_enum().filter(|(_, e)| action.applies_to(e));
            Ok(match action {
                Action::Read => candidates
                    .filter(|(_, e)| app.selected(e))
                    .take(*count)
                    .map(|(i, _)| i)
                    .collect(),
                Action::Unread => candidates.rev().take(*count).map(|(i, _)| i).collect(),
            })
        }

        UnreadArg::Name(a) => {
            let re = Regex::new(r"([\w\-_:.\[]+)(\(\w+\))?")?;
            let caps = re.captures(a).ok_or("no match")?;

            let name = caps.get(1).map_or(Some(a.as_str()), |m| Some(m.as_str()));
            let section = caps.get(2).map_or(None, |m| Some(m.as_str()));

            app.entries_enum()
                .filter(|(_, e)| action.applies_to(e) && e.name == name.unwrap())
                .filter(|(_, e)| section.is_none_or(|s| e.section == s))
                .min_by_key(|(_, e)| e.section.as_str())
                .map(|(i, _)| vec![i])
                .ok_or_else(|| format!("not in the {} list: {a}", action.pool()).into())
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
        0 => Err("specify a name, an index, an amount or --all".into()),
        1 => Ok(provided.into_iter().next().unwrap()),
        _ => Err("invalid combination".into()),
    }
}

fn confirm_action(action: Action) -> Result<()> {
    match action {
        Action::Read => {
            println!("This will mark ALL unread pages in the selected sections as read.")
        }
        Action::Unread => println!("This will remove ALL your reading progress."),
    }
    print!("Do You really want to proceed? [y/N]: ");

    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "yes" | "y" => Ok(()),
        _ => Err("canceled by user".into()),
    }
}
