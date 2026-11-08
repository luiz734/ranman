use crate::App;
use crate::Result;
use crate::app::Status;

pub fn history(app: &App, numbered: bool) -> Result<()> {
    for (i, e) in &app
        .entries_enum()
        .filter(|(_, e)| e.read_or_done())
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
