use crate::app::{App, Entry, Status};
use crate::args::Args;
use crate::{Result, generate, man_paths};
use rand::RngExt;
use std::collections::HashSet;

pub(crate) fn rescan(app: &mut App, args: &Args) -> Result<()> {
    let scanned = generate(man_paths(), args.sorted);

    let old = std::mem::take(&mut app.entries);
    let (entries, added, removed) =
        combine(old, scanned, args.keep_progress, args.prune, args.sorted);
    app.entries = entries;

    app.save_to_file()?;
    println!(
        "added {added}, removed {removed}, total {}",
        app.entries.len()
    );
    Ok(())
}

pub(crate) fn key(e: &Entry) -> (&str, &str) {
    (&e.name, &e.section)
}

fn combine(
    mut old: Vec<Entry>,
    new: Vec<Entry>,
    keep_progress: bool,
    prune: bool,
    sorted: bool,
) -> (Vec<Entry>, usize, usize) {
    let mut removed = 0;

    if prune {
        let on_disk: HashSet<_> = new.iter().map(key).collect();
        let before = old.len();
        old.retain(|e| e.status != Status::Unread || on_disk.contains(&key(e)));
        removed = before - old.len();
    }

    if !keep_progress {
        for e in &mut old {
            e.status = Status::Unread;
        }
    }

    let additions: Vec<Entry> = {
        let known: HashSet<_> = old.iter().map(key).collect();
        new.into_iter()
            .filter(|e| !known.contains(&key(e)))
            .collect()
    };

    let added = additions.len();
    place_new_entries(&mut old, additions, sorted);
    (old, added, removed)
}

fn place_new_entries(old: &mut Vec<Entry>, additions: Vec<Entry>, sorted: bool) {
    let boundary = old
        .iter()
        .position(|e| e.status == Status::Unread)
        .unwrap_or(old.len());

    if sorted {
        old.extend(additions);
        old[boundary..].sort();
    } else {
        let mut rng = rand::rng();
        for e in additions {
            let at = rng.random_range(boundary..=old.len());
            old.insert(at, e);
        }
    }
}
