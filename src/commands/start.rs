use crate::{App, Status, Result};

pub(crate) fn start(app: &mut App) -> Result<()> {
    let i = app
        .next_idx()
        .ok_or("no unread pages in the selected sections")?;
    let mut e = app.entries.remove(i);
    e.read();
    let boundary = app
        .entries
        .iter()
        .position(|e| e.status == Status::Unread)
        .unwrap_or(app.entries.len());
    println!("{} {}", e.section, e.name);
    app.entries.insert(boundary, e);
    Ok(app.save_to_file()?)
}
