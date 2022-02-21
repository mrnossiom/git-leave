# Git Leave

> Check for unsaved or uncommitted changes on your machine.

---

# Usage

Set the `leaveTool.defaultFolder` key in your git global configuration file to use the `--default flag`.

```
USAGE:
    git-leave [OPTIONS] [DIRECTORY]

ARGS:
    <DIRECTORY>    The directory to search in [default: .]

OPTIONS:
    -d, --default    Use git config default folder value for the directory to search in
    -h, --help       Print help information
    -n, --no-trim    Don't trim output path (may result in weird behavior on screen)
    -V, --version    Print version information
```
