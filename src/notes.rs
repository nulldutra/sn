use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Note {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Clone, Debug)]
pub enum BrowserEntry {
    Parent,
    Directory { name: String, path: PathBuf },
    Note(Note),
}

pub fn notes_dir() -> PathBuf {
    std::env::var("SN_NOTES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("notes")
        })
}

pub fn left_panel_width() -> u16 {
    std::env::var("SN_LEFT_WIDTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(32)
}

pub fn list_directory(root: &Path, dir: &Path) -> io::Result<Vec<BrowserEntry>> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    let mut entries = Vec::new();

    if dir != root {
        entries.push(BrowserEntry::Parent);
    }

    let mut directories = Vec::new();
    let mut notes = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().into_owned();

        if path.is_dir() {
            directories.push(BrowserEntry::Directory {
                name: file_name,
                path,
            });
        } else if is_note_file(&path) {
            notes.push(BrowserEntry::Note(load_note_file(&path)?));
        }
    }

    directories.sort_by(|a, b| entry_name(a).cmp(entry_name(b)));
    notes.sort_by(|a, b| entry_name(a).cmp(entry_name(b)));

    entries.extend(directories);
    entries.extend(notes);

    Ok(entries)
}

pub fn create_note(dir: &Path, name: &str) -> io::Result<PathBuf> {
    let name = sanitize_name(name);
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Note name cannot be empty",
        ));
    }

    fs::create_dir_all(dir)?;
    let path = dir.join(format!("{name}.md"));

    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "A note with this name already exists",
        ));
    }

    fs::write(&path, "")?;
    Ok(path)
}

pub fn save_note(path: &Path, content: &str) -> io::Result<()> {
    fs::write(path, content)
}

pub fn delete_note(path: &Path) -> io::Result<()> {
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Note file not found",
        ));
    }
    fs::remove_file(path)
}

pub fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn relative_dir(root: &Path, dir: &Path) -> String {
    dir.strip_prefix(root)
        .unwrap_or(dir)
        .to_string_lossy()
        .replace('\\', "/")
}

fn load_note_file(path: &Path) -> io::Result<Note> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();
    let content = fs::read_to_string(path).unwrap_or_default();

    Ok(Note {
        name,
        path: path.to_path_buf(),
        content,
    })
}

fn is_note_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "md" || ext == "txt")
}

fn entry_name(entry: &BrowserEntry) -> &str {
    match entry {
        BrowserEntry::Parent => "..",
        BrowserEntry::Directory { name, .. } => name,
        BrowserEntry::Note(note) => &note.name,
    }
}

fn sanitize_name(name: &str) -> String {
    name.trim()
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_notes_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sn-notes-test-{nanos}"))
    }

    #[test]
    fn lists_directories_before_notes() {
        let root = temp_notes_root();
        let work = root.join("work");
        fs::create_dir_all(&work).unwrap();
        fs::write(work.join("task.md"), "# Task").unwrap();
        fs::write(root.join("welcome.md"), "# Welcome").unwrap();

        let entries = list_directory(&root, &root).unwrap();
        assert!(matches!(
            entries.first(),
            Some(BrowserEntry::Directory { .. })
        ));
        assert!(entries
            .iter()
            .any(|entry| matches!(entry, BrowserEntry::Note(_))));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn parent_entry_appears_in_subdirectories() {
        let root = temp_notes_root();
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();

        let entries = list_directory(&root, &nested).unwrap();
        assert!(matches!(entries.first(), Some(BrowserEntry::Parent)));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn delete_note_removes_file() {
        let root = temp_notes_root();
        fs::create_dir_all(&root).unwrap();
        let path = root.join("gone.md");
        fs::write(&path, "bye").unwrap();

        delete_note(&path).unwrap();
        assert!(!path.exists());

        let _ = fs::remove_dir_all(root);
    }
}
