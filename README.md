<p align="center">
 <img alt="git-leave logo" src="https://raw.githubusercontent.com/mrnossiom/git-leave/main/assets/logo.png">
</p>

<p align="center"><strong>
Check for unsaved or uncommitted changes on your machine
</strong></p>

<p align="center">
  <img alt="Nix Powered" src="https://img.shields.io/badge/Nix-Powered-blue?logo=nixos" />
  <a href="https://mrnossiom.cachix.org">
    <img alt="Cachix Cache" src="https://img.shields.io/badge/cachix-mrnossiom-blue.svg" />
  </a>
  <a href="https://wakatime.com/badge/github/mrnossiom/git-leave">
    <img alt="Time spent" src="https://wakatime.com/badge/github/mrnossiom/git-leave.svg" />
  </a>
</p>

</p>

# Installation

<details>
  <summary>With <code>cargo</code> via <code>crates.io</code></summary>

Install from repository with cargo:

```sh
cargo install git-leave
```

You will also need `openssl` library in path, which you can install over you prefered package manager.

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

```
Check for unsaved or uncommitted changes on your machine

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

# Config

Set the `leaveTool.defaultFolder` key in your git global configuration file to use the `--default` or `-d` flag.

In `.config/git/config`, or any other git config file:
```conf
[leaveTool]
    defaultFolder = ~/path/to/projects
```

# Credits

- **[woobuc/sweep](https://github.com/woobuc/sweep)** for many concepts I implemented here (e.g. threads, logging)

---

Work is licensed under [`CECILL-2.1`](https://choosealicense.com/licenses/cecill-2.1/), a French OSS license that allows modification and distribution of the software while requiring the same license for derived works.

