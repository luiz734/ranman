use std::collections::BTreeMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use crate::app::{App, Status};
use crate::Result;

#[derive(Default)]
pub struct SectionInfo {
    total: usize,
    read: usize,
}

impl Display for SectionInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.read, self.total)
    }
}


pub fn info(app: &App) -> Result<()> {
    let mut s = String::new();
    let p = 9usize;
    s += &format!("{:p$} {}\n", "file:", app.path.as_path().to_string_lossy());
    s += &format!("{:p$} {}\n", "order:", app.order);
    s += &format!("{:p$} {}\n", "sections:", app.sections_string());

    let done = app.count_status(Status::Done);
    let reading = app.count_status(Status::Reading);
    let unread = app.count_status(Status::Unread);
    let total = done + reading + unread;
    s += &format!(
        "{:p$} {}/{} ({} done, {} reading)\n",
        "read:",
        done + reading,
        total,
        done,
        reading
    );

    if let Some(i) = app.current_idx() {
        s += &format!(
            "{:p$} {}({})\n",
            "current:", app.entries[i].name, app.entries[i].section
        );
    } else {
        s += &format!("{:p$} -\n", "current:");
    }

    if let Some(i) = app.next_idx() {
        s += &format!(
            "{:p$} {}({})\n",
            "next:", app.entries[i].name, app.entries[i].section
        );
    } else {
        s += &format!("{:p$} -\n", "next:");
    }

    let mut by_section: BTreeMap<&str, SectionInfo> = BTreeMap::new();
    for e in &app.entries {
        let s = by_section.entry(e.section.as_str()).or_default();
        s.total += 1;
        if e.read_or_done() {
            s.read += 1;
        }
    }

    s += "by section (* = selected):\n";
    for (k, v) in by_section {
        s += &format!("  {:6} {}\n", k, v)
    }

    print!("{}", s);

    Ok(())
}