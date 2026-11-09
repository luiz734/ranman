use crate::{App, Result};

pub(crate) fn done(app: &mut App) -> Result<()> {
    let i = app.current_idx().ok_or("nothing in progress")?;
    app.entries[i].done();

    if let Some(i) = app.current_idx() {
        println!("{} {}", app.entries[i].section, app.entries[i].name);
    }

    app.save_to_file()?;
    Ok(())
}
