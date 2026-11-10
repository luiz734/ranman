# ranman
Track reading progress of man pages.

## First usage

First time run the following;
```bash
ranman start [--sorted] [--sections 1,2,3]
```
- defaults to section 1
- defaults to random (not sorted)

This will generate the progress file at `~/.local/state/ranman/list`.

## Normal usage

- `man ($ranman start)`: mark the next unread page as read and print to stdout.
- `man ($ranman done)`: mark the current read page as read and print the next.
- `man ($ranman)`: print first unread page to stdout.
        

For more advanced options, see `ranman help`.

## Installation
```bash
cargo install --path .
```
or
```bash
cargo build --release
```
Then copy `./target/release/ranman` to your `$PATH`.