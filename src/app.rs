use std::io;
use std::path::{Path, PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use ratatui::widgets::ListState;

use crate::notes::{self, Note};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    CreateNote {
        input: String,
        error: Option<String>,
    },
    EditNote {
        note_index: usize,
        content: String,
        cursor: usize,
        status: Option<String>,
    },
}

pub struct App {
    pub notes_dir: PathBuf,
    pub left_width: u16,
    pub notes: Vec<Note>,
    pub selected: usize,
    pub list_state: ListState,
    pub preview_scroll: u16,
    pub mode: Mode,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let notes_dir = notes::notes_dir();
        let left_width = notes::left_panel_width();
        let notes = notes::load_notes(&notes_dir)?;

        Ok(Self {
            notes_dir,
            left_width,
            notes,
            selected: 0,
            list_state: ListState::default(),
            preview_scroll: 0,
            mode: Mode::Normal,
            should_quit: false,
        })
    }

    pub fn reload_notes(&mut self) -> io::Result<()> {
        let count = self.notes.len();
        self.notes = notes::load_notes(&self.notes_dir)?;

        if self.notes.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.notes.len() {
            self.selected = self.notes.len() - 1;
        } else if count != self.notes.len() {
            // keep selection when possible
        }

        self.preview_scroll = 0;
        Ok(())
    }

    fn reload_notes_and_select(&mut self, path: &Path) -> io::Result<()> {
        self.reload_notes()?;
        if let Some(idx) = self.notes.iter().position(|n| n.path == path) {
            self.selected = idx;
            self.list_state.select(Some(idx));
        }
        Ok(())
    }

    pub fn effective_left_width(&self, term_width: u16) -> u16 {
        let mut width = self.left_width.max(20);
        if term_width > 40 {
            width = width.min(term_width - 20);
        }
        width
    }

    pub fn selected_note(&self) -> Option<&Note> {
        self.notes.get(self.selected)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::CreateNote { .. } => self.handle_create_note_key(key),
            Mode::EditNote { .. } => self.handle_edit_note_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('a') => self.open_create_prompt(),
            KeyCode::Char('i') => self.open_edit_mode(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::SHIFT) => self.select_last(),
            KeyCode::Char('g') => self.select_first(),
            KeyCode::Char('G') => self.select_last(),
            KeyCode::Char(']') => self.scroll_preview_down(),
            KeyCode::Char('[') => self.scroll_preview_up(),
            _ => {}
        }
    }

    fn handle_create_note_key(&mut self, key: KeyEvent) {
        let Mode::CreateNote { input, error } = &mut self.mode else {
            return;
        };

        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                let name = input.clone();
                match notes::create_note(&self.notes_dir, &name) {
                    Ok(path) => {
                        let _ = self.reload_notes_and_select(&path);
                        self.mode = Mode::Normal;
                    }
                    Err(err) => {
                        *error = Some(err.to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                input.pop();
                *error = None;
            }
            KeyCode::Char(c) if !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                input.push(c);
                *error = None;
            }
            _ => {}
        }
    }

    fn handle_edit_note_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.save_and_exit_edit();
            return;
        }

        let Mode::EditNote {
            content,
            cursor,
            status,
            ..
        } = &mut self.mode
        else {
            return;
        };

        match key.code {
            KeyCode::Backspace => {
                delete_before_cursor(content, cursor);
                *status = None;
            }
            KeyCode::Enter => {
                insert_char(content, cursor, '\n');
                *status = None;
            }
            KeyCode::Left => {
                move_cursor_left(content, cursor);
                *status = None;
            }
            KeyCode::Right => {
                move_cursor_right(content, cursor);
                *status = None;
            }
            KeyCode::Up => {
                move_cursor_up(content, cursor);
                *status = None;
            }
            KeyCode::Down => {
                move_cursor_down(content, cursor);
                *status = None;
            }
            KeyCode::Char(c) if !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                insert_char(content, cursor, c);
                *status = None;
            }
            _ => {}
        }
    }

    fn save_and_exit_edit(&mut self) {
        let (note_index, content) = match &self.mode {
            Mode::EditNote {
                note_index,
                content,
                ..
            } => (*note_index, content.clone()),
            _ => return,
        };

        let path = self.notes[note_index].path.clone();
        match notes::save_note(&path, &content) {
            Ok(()) => {
                self.notes[note_index].content = content;
                self.mode = Mode::Normal;
            }
            Err(err) => {
                if let Mode::EditNote { status, .. } = &mut self.mode {
                    *status = Some(err.to_string());
                }
            }
        }
    }

    fn open_create_prompt(&mut self) {
        self.mode = Mode::CreateNote {
            input: String::new(),
            error: None,
        };
    }

    fn open_edit_mode(&mut self) {
        let Some(note) = self.notes.get(self.selected).cloned() else {
            return;
        };

        self.mode = Mode::EditNote {
            note_index: self.selected,
            cursor: note.content.len(),
            content: note.content,
            status: None,
        };
    }

    fn select_next(&mut self) {
        if self.selected + 1 < self.notes.len() {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
            self.preview_scroll = 0;
        }
    }

    fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
            self.preview_scroll = 0;
        }
    }

    fn select_first(&mut self) {
        if !self.notes.is_empty() {
            self.selected = 0;
            self.list_state.select(Some(0));
            self.preview_scroll = 0;
        }
    }

    fn select_last(&mut self) {
        if !self.notes.is_empty() {
            self.selected = self.notes.len() - 1;
            self.list_state.select(Some(self.selected));
            self.preview_scroll = 0;
        }
    }

    fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    pub fn preview_content(&self) -> Option<&str> {
        if let Mode::EditNote { content, note_index, .. } = &self.mode {
            if *note_index == self.selected {
                return Some(content.as_str());
            }
        }
        self.selected_note().map(|note| note.content.as_str())
    }

    pub fn clamp_preview_scroll(&mut self, visible_lines: u16) {
        let Some(content) = self.preview_content() else {
            self.preview_scroll = 0;
            return;
        };

        let total = line_count(content) as u16;
        if total <= visible_lines {
            self.preview_scroll = 0;
        } else {
            let max = total - visible_lines;
            self.preview_scroll = self.preview_scroll.min(max);
        }

        if self.is_editing_selected() {
            if let Mode::EditNote { content, cursor, .. } = &self.mode {
                let cursor_line = cursor_line(content, *cursor) as u16;
                if cursor_line < self.preview_scroll {
                    self.preview_scroll = cursor_line;
                } else if cursor_line >= self.preview_scroll + visible_lines {
                    self.preview_scroll = cursor_line + 1 - visible_lines;
                }
            }
        }
    }

    pub fn create_note_input(&self) -> Option<&str> {
        match &self.mode {
            Mode::CreateNote { input, .. } => Some(input.as_str()),
            _ => None,
        }
    }

    pub fn create_note_error(&self) -> Option<&str> {
        match &self.mode {
            Mode::CreateNote { error, .. } => error.as_deref(),
            _ => None,
        }
    }

    pub fn is_create_prompt_open(&self) -> bool {
        matches!(self.mode, Mode::CreateNote { .. })
    }

    pub fn is_editing_selected(&self) -> bool {
        matches!(
            &self.mode,
            Mode::EditNote { note_index, .. } if *note_index == self.selected
        )
    }

    pub fn is_editing(&self) -> bool {
        matches!(self.mode, Mode::EditNote { .. })
    }

    pub fn edit_status(&self) -> Option<&str> {
        match &self.mode {
            Mode::EditNote { status, .. } => status.as_deref(),
            _ => None,
        }
    }

    pub fn cursor_position_in_preview(&self, inner: ratatui::layout::Rect) -> Option<(u16, u16)> {
        let Mode::EditNote {
            content,
            cursor,
            note_index,
            ..
        } = &self.mode
        else {
            return None;
        };

        if *note_index != self.selected {
            return None;
        }

        let cursor_line = cursor_line(content, *cursor) as u16;
        let cursor_col = cursor_column(content, *cursor) as u16;
        let y = inner.y + cursor_line.saturating_sub(self.preview_scroll);
        let x = inner.x + cursor_col.min(inner.width.saturating_sub(1));

        if y >= inner.y && y < inner.bottom() {
            Some((x, y))
        } else {
            None
        }
    }
}

fn line_count(content: &str) -> usize {
    content.lines().count().max(1)
}

fn cursor_line(content: &str, cursor: usize) -> usize {
    let index = cursor.min(content.len());
    content[..index].chars().filter(|&c| c == '\n').count()
}

fn cursor_column(content: &str, cursor: usize) -> usize {
    let index = cursor.min(content.len());
    content[..index]
        .rsplit('\n')
        .next()
        .map(|line| line.chars().count())
        .unwrap_or(0)
}

fn position_at(content: &str, line: usize, column: usize) -> usize {
    let mut current_line = 0;
    let mut index = 0;

    for ch in content.chars() {
        if current_line == line {
            let line_start = index;
            let line_end = content[line_start..]
                .find('\n')
                .map(|offset| line_start + offset)
                .unwrap_or(content.len());
            let line_len = content[line_start..line_end].chars().count();
            let col = column.min(line_len);
            return index + content[line_start..line_end].chars().take(col).map(|c| c.len_utf8()).sum::<usize>();
        }

        if ch == '\n' {
            current_line += 1;
        }
        index += ch.len_utf8();
    }

    if current_line == line {
        return content.len();
    }

    content.len()
}

fn insert_char(content: &mut String, cursor: &mut usize, ch: char) {
    let index = (*cursor).min(content.len());
    content.insert(index, ch);
    *cursor = index + ch.len_utf8();
}

fn delete_before_cursor(content: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let prev = content
        .char_indices()
        .take_while(|(index, _)| *index < *cursor)
        .map(|(index, _)| index)
        .last()
        .unwrap_or(0);

    content.replace_range(prev..*cursor, "");
    *cursor = prev;
}

fn move_cursor_left(content: &str, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    *cursor = content
        .char_indices()
        .take_while(|(index, _)| *index < *cursor)
        .map(|(index, _)| index)
        .last()
        .unwrap_or(0);
}

fn move_cursor_right(content: &str, cursor: &mut usize) {
    if *cursor >= content.len() {
        return;
    }

    *cursor = content[*cursor..]
        .char_indices()
        .nth(1)
        .map(|(offset, _)| *cursor + offset)
        .unwrap_or(content.len());
}

fn move_cursor_up(content: &str, cursor: &mut usize) {
    let line = cursor_line(content, *cursor);
    if line == 0 {
        return;
    }

    let column = cursor_column(content, *cursor);
    *cursor = position_at(content, line - 1, column);
}

fn move_cursor_down(content: &str, cursor: &mut usize) {
    let line = cursor_line(content, *cursor);
    let column = cursor_column(content, *cursor);
    let next = position_at(content, line + 1, column);
    if next != content.len() || line + 1 < line_count(content) {
        *cursor = next;
    }
}

pub fn poll_event() -> io::Result<Option<Event>> {
    if event::poll(std::time::Duration::from_millis(100))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_helpers_track_lines_and_columns() {
        let text = "hello\nworld";
        assert_eq!(cursor_line(text, 7), 1);
        assert_eq!(cursor_column(text, 7), 1);
    }

    #[test]
    fn insert_and_delete_update_cursor() {
        let mut text = String::from("ab");
        let mut cursor = 1;
        insert_char(&mut text, &mut cursor, 'x');
        assert_eq!(text, "axb");
        delete_before_cursor(&mut text, &mut cursor);
        assert_eq!(text, "ab");
    }
}
