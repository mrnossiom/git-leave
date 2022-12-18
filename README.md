# `git leave`

###### Check for unsaved or uncommitted changes on your machine.

# Usage

Set the `leaveTool.defaultFolder` key in your git global configuration file to use the `--default` or `-d` flag.

```conf
# Your global .gitconfig
[leaveTool]
    defaultFolder = ~/path/to/projects
```

```
USAGE:
    git-leave [OPTIONS] [DIRECTORY]

ARGS:
    <DIRECTORY>    The directory to search in [default: .]

OPTIONS:
    -d, --default    Use git config default folder value for the directory to search in
    -h, --help       Print help information
    -V, --version    Print version information
```

## Thanks

-   **[woobuc/sweep](https://github.com/woobuc/sweep)** for many concept I implemented in my CLI (threads, logging)
