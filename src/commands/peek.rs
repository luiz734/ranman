use crate::app::App;

pub fn peek(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let i = app
        .next_idx()
        .ok_or("no unread pages in the selected sections")?;
    println!("{} {}", app.entries[i].section, app.entries[i].name);
    Ok(())
}