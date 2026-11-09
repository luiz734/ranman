use crate::Result;

pub(crate) fn help() -> Result<()> {
    println!("{MSG}");
    Ok(())
}

const MSG: &str = r#"Usage: ranman [command] [options]

Track reading progress of man pages. Should be used with the command 'man'

  man ($ranman start)   mark the next unread page as read and print to stdout
  man ($ranman done)    mark the current read page as read and print the next
                        unread, if any
  man ($ranman)         first unread page is printed to stdout

Each page is stored in $RANMAN_FILE and can be at any of the following states:
  [R]ead          being read; can be more than one
  [D]one          user finished reading
  [.] (Unread)    not read yet

Sections can be specified using --section sec1,sec2, ... for example:
  ranman --section 1,2

To see all available sections run ranman info; enabling section 1 will also
enable 1p, 1ssl, and so on; the default value is 1; pages are obtained by
scanning the directories in the output of 'manpath'. To add more pages, see
the ranman scan.

Environment variables:
  RANMAN_FILE:        where the state will be stored
                      If not provided, will be looked in the following order:
                       - $XDG_DATA_HOME/.local/share/ranman/list
                       - $HOME/.local/share/ranman/list
Global Options:
  -s, --sections S... sections to pick from: --sections 1,3,8; defaults to 1
Commands:
  (none)              page currently being read in format "<section> <name>"
                        stdout: the current [R]ead page
  done                mark current as done and print the next, if any
                        stdout: the next [R]ead page
  help                show this help message
  history             read pages, newest first
  info                summarize the state file

                        -n, --numbered          show line numbers
  peek                peek at the next unread   page, without consuming it
  pile                pages marked as [R]ead
                        -n, --numbered          show line numbers
  rescan              scan for new or removed man pages
                        --delete-progress       mark all entries as [U]nread
                        --prune                 remove dangling entries
                        --sorted                pages will be sorted
                        --update-unread         also applies sorting to unread pages
  start               mark the next unread page of the selected sections as
                      reading and print it
                        stdout: the new [R]ead page
                      random by default, but accepts [--sorted]
  [un]read            mark page(s) as read or unread
                        --all                   target all valid pages
                        -i, --index N           target at the given index
                        [count]                 target [count] from top
                        [name]                  man page name; also supports
                                                section; e.g: clone(2)"#;
