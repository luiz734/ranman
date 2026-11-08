use crate::{App, Result};

pub(crate) fn done(app: &mut App) -> Result<()> {
    let i = app.current_idx().ok_or("nothing in progress")?;
    app.entries[i].done();
    Ok(app.save_to_file()?)
}
