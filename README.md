# fast-jump (fj)

A blazing-fast directory jumper written in Rust. `fj` helps you navigate your filesystem efficiently using fuzzy search and a TUI interface.

![demo](demo.gif)

## Features

- 🚀 **Fast**: Powered by multithreading and the `skim` fuzzy matching algorithm.
- 🔍 **Smart**: Respects `.gitignore` and hidden files (via `ignore` crate).
- 🖥️ **Interactive**: Clean TUI using `ratatui` and `crossterm`.
- ⌨️ **Simple**: Just type to search, Arrow keys to navigate, Enter to jump.

## Installation

### From Source

Ensure you have Rust installed.

```bash
git clone https://github.com/yourusername/fast-jump.git
cd fast-jump
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/fj` (make sure `~/.cargo/bin` is in your `$PATH`).

## Usage

`fj` is designed to work with a shell wrapper function to change your directory automatically.

### Shell Wrapper (Bash / Zsh)

Add the following function to your `~/.bashrc` or `~/.zshrc`:

```bash
function fj() {
    ~/.cargo/bin/fj "$@"
    local temp_file="/tmp/fj_target"
    if [ -f "$temp_file" ]; then
        local target_dir=$(cat "$temp_file")
        if [ -d "$target_dir" ]; then
            cd "$target_dir"
        fi
        rm -f "$temp_file"
    fi
}
```

Now reload your shell (`source ~/.zshrc`) and simply run to search in your home directory:

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

**fj advantages**: Single binary, integrated fuzzy matching, interactive TUI, respects `.gitignore`

