# sn

A TUI (Terminal User Interface) notes app inspired by Obsidian, built with [Ratatui](https://ratatui.rs/).

## Requirements

- Rust 1.74+ (2021 edition)

## Build

```bash
cargo build --release
```

## Usage

```bash
cargo run
# or
cargo run --release
```

Optional environment variables:

| Variable        | Description                | Default   |
|-----------------|----------------------------|-----------|
| `SN_NOTES_DIR`  | Notes directory            | `./notes` |
| `SN_LEFT_WIDTH` | Left panel width (columns) | `32`      |

## Interface

```
┌─ Notes ──────────────┐┌─ Preview ──────────────────────┐
│ welcome              ││ # Welcome to sn                │
│ ideas                ││                                │
│                      ││ sn is a TUI notes app...       │
└──────────────────────┘└────────────────────────────────┘
 ↑↓/jk notes  [/] scroll  g/G top/bottom  q quit
```

- **Left panel**: note list (`.md` and `.txt` files)
- **Right panel**: content of the selected note

## Keybindings

| Key           | Action                |
|---------------|-----------------------|
| `a`           | Create a new note     |
| `i`           | Edit selected note    |
| `Esc`         | Save and exit edit mode |
| `↑` / `↓`     | Navigate notes        |
| `j` / `k`     | Navigate notes        |
| `[` / `]`     | Scroll note content   |
| `g` / `G`     | First / last note     |
| `q` / `Esc`   | Quit                  |

## License

BSD 2-Clause — see [LICENSE](LICENSE).
