use crate::Result;

pub(crate) fn help() -> Result<()> {
    println!("{MSG}");
    Ok(())
}

const MSG: &str = r#"Usage: ranman [command] [options]

This tool was made to be used with "man": man $(ranman). It tracks the progress of man pages.
Pages can be at one of the following state [R]ead [D]one [U]nread;
you may mark multiple pages as read.

ENVIRONMENT:
  - $RANMAN_FILE:    where the state will be stored
                     If not provided, will be looked in the following order:
                       - $XDG_DATA_HOME/.local/share/ranman/list
                       - $HOME/.local/share/ranman/list
COMMANDS:
  (none)              the page you are reading in format "<section> <name>"
  done                mark current as done; the previous one on the pile becomes current
  help                show this help message
  info                summarize the state file
  history             read pages, newest first       [-n|--numbered]
  peek                peek at the next unread page, without consuming it
  pile                pages still in progress        [-n|--numbered]
  start               mark the next unread page of the selected sections as reading and print it;
                      random by default, but accepts [--sorted]
  unread  [count|name]
                      mark page as unread
                        [--all]            see options
                        [-i|--index] N     see options
                        [count]            target COUNT from top
                        [name]             man page name; also supports section: clone(2)
OPTIONS:
  --all               all valid targets for that context
  -i, --index N       target at the given index
  -n, --numbered      numbered output
  -s, --sections S... sections to pick from: --sections 1,3,8; defaults to 1
  --sorted            sort the entries; can only be used with start and update"#;
