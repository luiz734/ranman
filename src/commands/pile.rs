use crate::{App, Status};
use crate::Result;
pub fn pile(app: &App, numbered: bool) -> Result<()> {
    let mut any = false;
    for (i, e) in app
        .entries
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, e)| e.status == Status::Reading)
    {
        if numbered {
            let s = if e.status == Status::Reading { "R" } else { "" };
            println!("{:4} {} {}({})", i + 1, s, e.name, e.section)
        } else {
            println!("{}({})", e.name, e.section)
        }
        any = true;
    }
    if !any {
        eprintln!("Pile is empty.");
    }
    Ok(())
}
