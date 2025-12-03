<p align="center">
 <img alt="git-leave logo" src="https://raw.githubusercontent.com/mrnossiom/git-leave/main/assets/logo.png">
</p>

<p align="center"><strong>
Check for unsaved or uncommitted changes on your machine
</strong></p>

<p align="center">
  <img alt="Nix Powered" src="https://img.shields.io/badge/Nix-Powered-blue?logo=nixos" />
  <a href="https://crates.io/crates/git-leave">
    <img alt="git-leave crates.io version" src="https://img.shields.io/crates/v/git-leave">
  </a>
  <a href="https://matrix.to/#/#git-leave:wiro.world">
    <img alt="Matrix room at #git-leave:wiro.world" src="https://img.shields.io/badge/Matrix-%23git--leave%3Awiro.world-white?logo=matrix">
  </a>
</p>

# Installation

<details>
  <summary>With <code>cargo</code> via <code>crates.io</code></summary>

Install from repository with cargo:

```sh
cargo install git-leave
```

</details>

<details>
  <summary>With <code>nix</code> flakes</summary>

A `flake.nix` is available which means that you can use `github:mrnossiom/git-leave` as a flake identifier, so you can.

- import this repository in your flake inputs

  ```nix
  {
    git-leave.url = "github:mrnossiom/git-leave";
    git-leave.inputs.nixpkgs.follows = "nixpkgs";
  }
  ```

  Add the package to your [NixOS](https://nixos.org/) or [Home Manager](https://github.com/nix-community/home-manager) packages depending on your installation.

- use with `nix shell`/`nix run` for temporary testing

  e.g. `nix shell github:mrnossiom/git-leave`

- use with `nix profile` for imperative installation

  e.g. `nix profile install github:mrnossiom/git-leave`

Package is reachable through `packages.${system}.default` or `packages.${system}.git-leave`.

</details>

# Usage

```text
Check for unsaved or uncommitted changes on your machine

Usage: git-leave [OPTIONS] [DIRECTORY]

Arguments:
  [DIRECTORY]  Directory to search in [default: .]

Options:
  -d, --default            Use default folder specified in git config for the directory to search in
      --follow-symlinks    Follow symlinks
      --show-directories   Show the directories we are actually crawling
      --threads <THREADS>  Number of cores to use for crawling [default: <num_cpus>]
      --check <CHECK>      Override checks to run on found repositories [possible values: dirty, ahead-branches, no-upstream-branches]
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version
```

## Examples

- To check all repos under the current directory

  ```sh
  git leave
  ```

- To check all repos under the specified directory

  ```sh
  git leave path/to/directory
  ```
- To check all repos under the default directory (see config)

  ```sh
  git leave --default
  ```

# Checks

- `dirty`: Whether the repository has a dirty working copy
- `ahead-branches`: List all branches that are ahead of their remote
- `no-upstream-branches`: List all branches with no upstream

# Config

Set the `leaveTool.defaultFolder` key in your git global configuration file to use the `--default` or `-d` flag.

In your global git config file (e.g. `.config/git/config`):

```git-config
[git_leave]
    # Folder used when the `--default` flag is provided
    defaultFolder = ~/path/to/projects
    # Override checks to run on repositories.
    # This is used when checks report false positives for your setup. (e.g. Jujutsu)
    #
    # You can get the list in `--help`
    checks = dirty
    checks = ahead-branches
```

# Credits

- **[woobuc/sweep](https://github.com/woobuc/sweep)** for many concepts I implemented here (e.g. threads, logging)

---

This work is licensed under [`CeCILL-2.1`](https://choosealicense.com/licenses/cecill-2.1), a strong copyleft French OSS license. This license allows modification and distribution of the software while requiring the same license for derived works.
