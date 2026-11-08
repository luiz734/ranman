use crate::App;

use crate::Result;

pub fn done(app: &mut App) -> Result<()> {
    let i = app.current_idx().ok_or("nothing in progress")?;
    app.entries[i].done();
    Ok(app.save_to_file()?)
}