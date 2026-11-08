use std::collections::HashSet;
use std::fmt;
use crate::app::App;

#[derive(Debug, Clone)]
pub enum ArgError {
    BadSection(String),
}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgError::BadSection(section) => write!(f, "bad section: {section}"),
        }
    }
}
impl std::error::Error for ArgError {}

#[derive(Default)]
pub struct Args {
    pub(crate) cmd: Option<String>,
    pub(crate) positional: Vec<String>,
    pub(crate) numbered: bool,
    pub(crate) index: Option<usize>,
    pub(crate) all: bool,
    sections: Option<Vec<String>>,
}


pub(crate) fn parse_args(app: &App) -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = Args::default();
    let mut it = std::env::args().skip(1);

    while let Some(a) = it.next() {
        match a.as_str() {
            "--numbered" | "-n" => args.numbered = true,
            "--all" => args.all = true,
            "--index" | "-i" => {
                let index = it.next().ok_or("--index needs a number")?;
                args.index = Some(index.parse().map_err(|_| format!("bad index: {index}"))?);
            }
            "--sections" => {
                let values = it.next().ok_or("usage is --sections a,b,c")?;
                let values = values.split(",").collect();
                valid_sections(app, &values)?;
                args.sections = Some(values.iter().map(|s| s.to_string()).collect());
            }

            s if s.starts_with('-') && s.len() > 1 => {
                return Err(format!("unknown option {s}").into());
            }
            _ if args.cmd.is_none() => args.cmd = Some(a),
            _ => args.positional.push(a),
        }
    }

    Ok(args)
}

fn valid_sections(app: &App, cmd_sections: &Vec<&str>) -> Result<(), ArgError> {
    let mut all_sections = HashSet::<&str>::new();
    for e in app.entries.iter().map(|e| &e.section) {
        all_sections.insert(e);
    }
    for s in cmd_sections {
        if !all_sections.contains(s) {
            return Err(ArgError::BadSection(format!(
                "section \"{}\" is invalid",
                s
            )));
        }
    }
    Ok(())
}
