use crate::app::App;

pub(crate) fn ranman(app: &mut App) -> crate::Result<()> {
    if let Some(i) = app.current_idx() {
        println!("{} {}", app.entries[i].section, app.entries[i].name);
    } else {
        return Err("no unread pages in the selected sections".into());
        // if let Some(i) = app.next_idx() {
        //     // println!("{} {}", app.entries[i].section, app.entries[i].name);
        //     // eprintln!("not started; run 'ranman start' to mark it as reading");
        // } else {
        //     return Err("no unread pages in the selected sections".into());
        // }
    }
    Ok(())
}
