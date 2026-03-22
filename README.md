# fast-jump (fj)

A blazing-fast directory jumper written in Rust. `fj` helps you navigate your filesystem efficiently using fuzzy search and a TUI interface.

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
cargo build --release
```

The binary will be available at `target/release/fj`.

## Usage

`fj` is designed to work with a shell wrapper function to change your directory automatically.

### Shell Wrapper (Bash / Zsh)

Add the following function to your `~/.bashrc` or `~/.zshrc`:

```bash
function fj() {
    # Path to the compiled binary
    local binary_path="$HOME/my-projects/fast-jump/target/release/fj"

    # Run fj
    "$binary_path" "$@"

    # Path to the temp file used by fj
    local temp_file="/tmp/fj_target" # Adjust if your TMPDIR is different

    # Check if a selection was made
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

