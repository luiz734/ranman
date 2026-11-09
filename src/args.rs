use crate::app::App;
use std::collections::HashSet;
use std::fmt;

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
    pub(crate) sorted: bool,
    pub(crate) sections: Option<Vec<String>>,
}

pub(crate) fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
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
                let values = values.split(",").collect::<Vec<_>>();
                // valid_sections(app, &values)?;
                args.sections = Some(values.iter().map(|s| s.to_string()).collect());
            }
            "--sorted" => {
                if let Some(cmd) = &args.cmd
                    && (cmd == "start" || cmd == "update")
                {
                    args.sorted = true;
                } else {
                    return Err(("--sorted can only be used with start or update").into());
                }
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

pub fn valid_sections(app: &App, cmd_sections: &Option<Vec<String>>) -> Result<(), ArgError> {
    // TODO: refactor
    // Only check if --section was provided
    let Some(_) = cmd_sections else { return Ok(()) };

    let mut all_sections = HashSet::<&str>::new();
    for e in app.entries.iter().map(|e| &e.section) {
        all_sections.insert(e);
    }
    for s in cmd_sections.as_ref().unwrap() {
        if !all_sections.contains(s.as_str()) {
            return Err(ArgError::BadSection(format!(
                "section \"{}\" is invalid",
                s
            )));
        }
    }
    Ok(())
}
