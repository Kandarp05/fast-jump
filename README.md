# fast-jump (fj)

A blazing-fast directory jumper written in Rust. `fj` helps you navigate your filesystem efficiently using multithreaded fuzzy search and a clean TUI.

![demo](demo.gif)

## Features

- **Fast**: Powered by multithreading and the `skim` fuzzy matching algorithm.
- **Smart**: Respects `.gitignore` and hidden files (via `ignore` crate).
- **Interactive**: Clean TUI using `ratatui` and `crossterm`.
- **Simple**: Type `fj` -> `Enter` -> `start typing` to search for directories.

## Installation

### 1. Crates.io (Cargo)
If you have the Rust toolchain installed, this is the easiest method
```bash
cargo install fast-jump
```

### 2. Pre-compiled Binaries (macOS / Linux)
Download the latest release from the [Releases](https://github.com/Kandarp05/fast-jump/releases) page. Extract the archive and move the fj binary to any directory in your `$PATH` (e.g., `~/.local/bin` or `/usr/local/bin`).

### 3. Build from Source
```bash
git clone github.com/Kandarp05/fast-jump
cd fast-jump
cargo install --path .
````


## Setup (Recommended)

> [!IMPORTANT]
>Because `fj` runs as a child process, it cannot natively alter the working directory of your parent shell. **To automatically `cd` into your selected directory, you must use a shell wrapper**.

Ensure the `fj` executable is in your `$PATH`, then add the following function to your shell configuration file (`~/.bashrc` or `~/.zshrc`).

### Bash / Zsh Wrapper
```bash
fj() {
    local target_dir
    target_dir="$(command fj "$@")" || return
    [[ -n "$target_dir" && -d "$target_dir" ]] && builtin cd -- "$target_dir"
}
```

> [!NOTE]
> The script above is written for POSIX-compliant shells. If you use a different shell (like Fish, Nushell, or PowerShell), you will need to write an equivalent wrapper that captures the standard output of fj and passes it to your shell's cd command.

Restart your terminal or run `source ~/.zshrc` to apply the changes

## Usage

Search from your home directory:

```bash
fj
```

Or provide a starting directory:

```bash
fj ~/workspace
```

## Benchmarks

Performance comparable to alternatives with superior UX (tested on 183,998 directories):

| Tool | Time | Range (10 runs) |
|------|------|-----------------|
| **fj (headless)** | 403.3 ms ± 44.3 ms | 338.7 - 457.5 ms |
| **fd + fzf** | 437.2 ms ± 64.4 ms | 350.6 - 544.6 ms |


