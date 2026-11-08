use crate::app::App;
use crate::Result;
pub fn peek(app: &App) -> Result<()> {
    let i = app
        .next_idx()
        .ok_or("no unread pages in the selected sections")?;
    println!("{} {}", app.entries[i].section, app.entries[i].name);
    Ok(())
}