use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::notes::BrowserEntry;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let left_width = app.effective_left_width(area.width);

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(left_width), Constraint::Min(0)])
        .split(root[0]);

    render_notes_panel(frame, columns[0], app);
    render_preview_panel(frame, columns[1], app);
    render_status_bar(frame, root[1], app);

    if app.is_create_prompt_open() {
        render_create_prompt(frame, app);
    }
}

fn render_notes_panel(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title(app.notes_panel_title())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if app.entries.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(Span::styled(
                "No notes in:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                app.current_dir.display().to_string(),
                Style::default().add_modifier(Modifier::DIM),
            )),
        ])
        .block(block)
        .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let (label, entry_style) = match entry {
                BrowserEntry::Parent => ("../".to_string(), Style::default().fg(Color::DarkGray)),
                BrowserEntry::Directory { name, .. } => {
                    (format!("{name}/"), Style::default().fg(Color::Yellow))
                }
                BrowserEntry::Note(note) => (note.name.clone(), Style::default()),
            };

            let style = if i == app.selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                entry_style
            };

            ListItem::new(label).style(style)
        })
        .collect();

    app.list_state.select(Some(app.selected));
    let list = List::new(items).block(block).highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);

    let footer_y = area.bottom().saturating_sub(2);
    if footer_y > area.top() {
        let footer = Paragraph::new(format!("{}/{}", app.selected + 1, app.entries.len()))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(
            footer,
            Rect {
                x: area.x + 1,
                y: footer_y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
        );
    }
}

fn render_preview_panel(frame: &mut Frame, area: Rect, app: &mut App) {
    let inner_height = area.height.saturating_sub(2);
    app.clamp_preview_scroll(inner_height);

    let editing = app.is_editing_selected();
    let title = app.preview_title();
    let border_color = if editing { Color::Yellow } else { Color::Cyan };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);

    if app.selected_note().is_none() {
        let message = if app.entries.is_empty() {
            "Select or create a note."
        } else {
            match app.entries.get(app.selected) {
                Some(BrowserEntry::Directory { name, .. }) => {
                    frame.render_widget(
                        Paragraph::new(format!("Folder: {name}/\nPress l or Enter to open."))
                            .style(Style::default().fg(Color::DarkGray))
                            .block(block),
                        area,
                    );
                    return;
                }
                Some(BrowserEntry::Parent) => {
                    frame.render_widget(
                        Paragraph::new("Parent directory\nPress l or Enter to go up.")
                            .style(Style::default().fg(Color::DarkGray))
                            .block(block),
                        area,
                    );
                    return;
                }
                _ => "Select a note to preview.",
            }
        };

        frame.render_widget(
            Paragraph::new(message)
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        );
        return;
    }

    let Some(content) = app.preview_content() else {
        return;
    };

    if content.is_empty() && !editing {
        let empty =
            Paragraph::new("(empty file)").style(Style::default().add_modifier(Modifier::DIM));
        frame.render_widget(empty.block(block), area);
        return;
    }

    let paragraph = if editing {
        Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.preview_scroll, 0))
    } else {
        let text = crate::markdown::render(content);
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.preview_scroll, 0))
    };

    let total = if editing {
        line_count(content) as u16
    } else {
        crate::markdown::line_count(content) as u16
    };

    frame.render_widget(paragraph, area);
    if total > inner_height && inner.height > 0 {
        let end = (app.preview_scroll + inner_height).min(total);
        let scroll_info = format!("Line {}-{} of {}", app.preview_scroll + 1, end, total);
        frame.render_widget(
            Paragraph::new(scroll_info).style(Style::default().fg(Color::DarkGray)),
            Rect {
                x: inner.x,
                y: inner.bottom().saturating_sub(1),
                width: inner.width,
                height: 1,
            },
        );
    }

    if editing {
        if let Some(status) = app.edit_status() {
            frame.render_widget(
                Paragraph::new(status).style(Style::default().fg(Color::Red)),
                Rect {
                    x: inner.x,
                    y: inner.y,
                    width: inner.width.min(status.len() as u16 + 1),
                    height: 1,
                },
            );
        }

        if let Some((x, y)) = app.cursor_position_in_preview(inner) {
            frame.set_cursor_position((x, y));
        }
    }
}

fn line_count(content: &str) -> usize {
    content.lines().count().max(1)
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let help = if app.is_create_prompt_open() {
        "Enter create  Esc cancel"
    } else if app.is_editing() {
        "Esc save & exit  arrows move  type to edit"
    } else {
        "↑↓/jk navigate  h/l parent/enter  a new  i edit  [/] scroll  q quit"
    };

    let help = Paragraph::new(Span::styled(
        help,
        Style::default().add_modifier(Modifier::DIM),
    ));
    frame.render_widget(help, area);
}

fn render_create_prompt(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 7, frame.area());
    frame.render_widget(Clear, area);

    let input = app.create_note_input().unwrap_or("");
    let block = Block::default()
        .title(" New note ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let current_dir = app.current_dir.display().to_string();
    let label = Paragraph::new(format!("Note name (in {current_dir}):"))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(
        label,
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        },
    );

    let input_line = format!("{input}_");
    let input_widget = Paragraph::new(input_line).style(Style::default().fg(Color::White));
    frame.render_widget(
        input_widget,
        Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        },
    );

    let message = app
        .create_note_error()
        .map(|err| Line::from(Span::styled(err, Style::default().fg(Color::Red))))
        .unwrap_or_else(|| {
            Line::from(Span::styled(
                "Creates a .md file in the current folder",
                Style::default().fg(Color::DarkGray),
            ))
        });

    frame.render_widget(
        Paragraph::new(message),
        Rect {
            x: inner.x,
            y: inner.y + 2,
            width: inner.width,
            height: 1,
        },
    );

    let cursor_x = inner.x + input.len() as u16;
    let cursor_y = inner.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_width = area.width.saturating_mul(percent_x) / 100;
    let popup_height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;

    Rect {
        x,
        y,
        width: popup_width.max(20),
        height: popup_height,
    }
}
