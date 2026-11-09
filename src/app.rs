use crate::generate;
use std::fmt::{Display, Formatter};
use std::fs::{create_dir_all, rename};
use std::io::Error;
use std::iter::Enumerate;
use std::path::PathBuf;
use std::slice::Iter;

const APP_VERSION: &str = "1";

pub enum Order {
    Random,
    Alphabetical,
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::Random => write!(f, "random"),
            Order::Alphabetical => write!(f, "alphabetical"),
        }
    }
}

pub struct Entry {
    pub status: Status,
    pub name: String,
    pub section: String,
}

impl Entry {
    pub(crate) fn read(&mut self) {
        self.status = Status::Reading;
    }
    pub(crate) fn unread(&mut self) {
        self.status = Status::Unread;
    }
    pub(crate) fn done(&mut self) {
        self.status = Status::Done;
    }
    pub(crate) fn read_or_done(&self) -> bool {
        self.status != Status::Unread
    }
}

pub struct App {
    pub path: PathBuf,
    pub order: Order,
    pub entries: Vec<Entry>,
    sections: Vec<String>,
}

impl App {
    pub(crate) fn entries_enum(&self) -> Enumerate<Iter<'_, Entry>> {
        self.entries.iter().enumerate()
    }

    pub(crate) fn save_to_file(&self) -> Result<(), Error> {
        let mut text = String::new();

        text += &format!("#ranman {}\n", APP_VERSION);
        text += &format!("#order {}\n", self.order.to_string());

        for e in &self.entries {
            text += &format!("{} {}({})\n", e.status, e.name, e.section)
        }

        // Save to disk
        let base_dir = self.path.as_path().parent().unwrap();
        let tmp_file = base_dir.iter().as_path().join("list.tmp");

        match create_dir_all(base_dir).and_then(|_| std::fs::write(&tmp_file, &text)) {
            Ok(_) => {
                rename(tmp_file, &self.path)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn count_status(&self, status: Status) -> usize {
        self.entries
            .iter()
            .filter(|e| self.selected(e))
            .filter(|e| e.status == status)
            .count()
    }

    pub(crate) fn sections_string(&self) -> String {
        self.sections
            .iter()
            .fold(String::new(), |acc, s| acc + s + " ")
            .trim()
            .to_string()
    }

    pub(crate) fn next_idx(&self) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.status == Status::Unread && self.selected(e))
    }

    pub(crate) fn current_idx(&self) -> Option<usize> {
        self.entries
            .iter()
            .rposition(|e| e.status == Status::Reading && self.selected(e))
    }

    pub fn selected(&self, e: &Entry) -> bool {
        self.sections
            .iter()
            .any(|s| e.section.starts_with(s.as_str()))
    }

    pub(crate) fn generate(
        file: PathBuf,
        sections: Vec<String>,
        man_paths: Vec<PathBuf>,
        sorted: bool,
    ) -> Result<App, Error> {
        let entries = generate(man_paths, sorted);

        let order = match sorted {
            true => Order::Alphabetical,
            false => Order::Random,
        };

        Ok(App {
            path: file,
            order,
            entries,
            sections,
        })
    }

    pub(crate) fn from_text(
        file: PathBuf,
        text: String,
        sections: Vec<String>,
    ) -> Result<App, Error> {
        let order = text
            .lines()
            .find(|line| line.starts_with("#order"))
            .unwrap_or(&"#order random");
        let order = match order.split_once(" ") {
            Some(("#order", "random")) => Order::Random,
            Some(("#order", "alphabetical")) => Order::Alphabetical,
            _ => Order::Random,
        };

        let entries = text
            .lines()
            .filter(|l| !l.starts_with("#") && !l.is_empty())
            .map(|line| {
                let pair = line.split_once(" ").unwrap();

                let status = match pair.0 {
                    "R" => Status::Reading,
                    "D" => Status::Done,
                    _ => Status::Unread,
                };

                let pair = pair.1.rsplit_once("(").unwrap();
                let name = pair.0.to_string();
                let section = pair.1.strip_suffix(")").unwrap().to_string();

                Entry {
                    status,
                    name,
                    section,
                }
            });

        Ok(App {
            path: file,
            order,
            entries: entries.collect(),
            sections,
        })
    }
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Done,
    Reading,
    Unread,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Reading => write!(f, "R"),
            Status::Unread => write!(f, "."),
            Status::Done => write!(f, "D"),
        }
    }
}
