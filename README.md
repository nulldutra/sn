# sn

A TUI (Terminal User Interface) notes app inspired by Obsidian, built with [Ratatui](https://ratatui.rs/).

## Requirements

- Rust 1.74+ (2021 edition)

## Build

```bash
cargo build --release
```

## Releases

Prebuilt binaries for Linux, macOS, and Windows are attached to [GitHub Releases](https://github.com/nulldutra/sn/releases).

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

![](./.assets/image.png)

- **Left panel**: folder browser with notes (`.md` and `.txt` files)
- **Right panel**: rendered Markdown preview (raw text while editing)

## Keybindings

| Key           | Action                    |
|---------------|---------------------------|
| `a`           | Create a new note         |
| `i`           | Edit selected note        |
| `d`           | Delete selected note      |
| `Esc`         | Save and exit edit mode   |
| `h` / `←`     | Go to parent directory    |
| `l` / `→`     | Enter folder              |
| `Enter`       | Enter folder / go up      |
| `↑` / `↓`     | Navigate list             |
| `j` / `k`     | Navigate list             |
| `[` / `]`     | Scroll note content       |
| `g` / `G`     | First / last item         |
| `q` / `Esc`   | Quit                      |

## License

BSD 2-Clause — see [LICENSE](LICENSE).
