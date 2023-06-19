![Git Leave Logo](assets/logo.png)

###### Check for unsaved or uncommitted changes on your machine.

# Usage

Set the `leaveTool.defaultFolder` key in your git global configuration file to use the `--default` or `-d` flag.

```conf
# Your global .gitconfig
[leaveTool]
    defaultFolder = ~/path/to/projects
```

```
Check for unsaved or uncommitted changes on your machine.

Usage: git-leave [OPTIONS] [DIRECTORY]

Arguments:
  [DIRECTORY]  The directory to search in [default: .]

Options:
  -d, --default            Use git config default folder value for the directory to search in
      --follow-symlinks    Should we follow symlinks
      --show-directories   Should we show the directories we are actually crawling
      --threads <THREADS>  The number of cores to use for crawling [default: number_of_cores]
  -h, --help               Print help
  -V, --version            Print version
```

## Credits

-   **[woobuc/sweep](https://github.com/woobuc/sweep)** for many concept I implemented here (threads, logging)
