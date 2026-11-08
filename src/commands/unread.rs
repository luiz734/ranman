use std::io;
use std::io::Write;
use crate::{App, Status};
use crate::args::Args;

pub fn handle_unread(app: &mut App, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut indexes: Vec<usize> = match args.all {
        true => {
            print!(
                "This will remove ALL your reading progress.\nDo You really want to proceed? [y/N]: "
            );
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "yes" | "Y" | "y" => app
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.status != Status::Unread)
                    .map(|(i, _)| i)
                    .collect::<Vec<usize>>(),
                _ => return Err("Canceled by user".to_string().into()),
            }
        }
        false => vec![find_target(app, args)?],
    };
    // If we remove index I < J before the indexes will shift.
    // Remove larger indexes first.
    indexes.sort();
    indexes.reverse();
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


fn find_target(app: &mut App, args: &Args) -> Result<usize, Box<dyn std::error::Error>> {
    match args.index {
        Some(n) => {
            let i = n.checked_sub(1).ok_or("indices start at 1")?;
            match app.entries.get(i) {
                Some(e) if e.status != Status::Unread => Ok(i),
                _ => Err(format!("no such index: {n} (see 'ranman history -n')").into()),
            }
        }
        None => {
            // TODO: if it is just a number behaves like the old "pop".
            // pop N change the last N entries marked as "R" to "U"

            if let Some(a) = args.positional.first() {
                let (name, sec) = match a.rsplit_once('(') {
                    Some((n, s)) => (n, s.strip_suffix(')')),
                    None => (a.as_str(), None),
                };

                app.entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.status != Status::Unread && e.name == name)
                    .filter(|(_, e)| sec.is_none_or(|s| e.section == s))
                    .min_by_key(|(_, e)| e.section.as_str())
                    .map(|(i, e)| Ok(i))
                    .ok_or_else(|| format!("not in the read list: {a}"))?
            } else {
                Err("remove needs a page name, --index N or --all".into())
            }
        }
    }
}
