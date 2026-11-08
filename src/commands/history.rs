use crate::{App};
use crate::app::Status;
use crate::Result;

pub fn history(app: &App, numbered: bool) -> Result<()> {
    for (i, e) in &app
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.status != Status::Unread)
        .rev()
        .collect::<Vec<_>>()
    {
        if numbered {
            let s = if e.status == Status::Reading {
                "R"
            } else {
                "D"
            };
            println!("{:4} {} {}({})", i + 1, s, e.name, e.section)
        } else {
            println!("{}({})", e.name, e.section)
        }
    }
    Ok(())
}