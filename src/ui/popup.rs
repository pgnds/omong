use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::{App, Overlay, PopupField, Screen};

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0]);

    horizontal[0]
}

pub fn render(frame: &mut Frame, app: &App) {
    match &app.overlay {
        Some(Overlay::AddConnectionPopup { .. }) => render_add_connection(frame, app),
        Some(Overlay::Confirm {
            message,
            yes_selected,
            ..
        }) => render_confirm(frame, message, *yes_selected),
        Some(Overlay::Prompt { .. }) => render_prompt(frame, app),
        Some(Overlay::FilterPrompt { .. }) | None => {}
        Some(Overlay::InformationBox) => render_information_box(frame, app),
    }
}

fn render_add_connection(frame: &mut Frame, app: &App) {
    let Some(Overlay::AddConnectionPopup {
        alias_buffer,
        uri_buffer,
        focus,
        editing_id,
    }) = &app.overlay
    else {
        return;
    };

    let popup_area = centered_rect(60, 10, frame.area());

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(if editing_id.is_some() {
            "edit connection"
        } else {
            "new connection"
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let alias_style = if matches!(focus, PopupField::Alias) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let uri_style = if matches!(focus, PopupField::Uri) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let alias_line = Paragraph::new(format!("alias: {}_", alias_buffer)).style(alias_style);

    let uri = if editing_id.is_some() {
        let mut s = String::from(uri_buffer);
        if let Some(password_offset) = s.find('@')
            && let Some(password_start) = s[..password_offset].rfind(':')
            && !s[password_start..].starts_with("://")
        {
            s.replace_range((password_start + 1)..password_offset, "***")
        };
        s
    } else {
        uri_buffer.clone()
    };
    let uri_line = Paragraph::new(format!("uri: {}_", uri)).style(uri_style);

    frame.render_widget(alias_line, fields[0]);
    frame.render_widget(uri_line, fields[1]);
}

fn render_prompt(frame: &mut Frame, app: &App) {
    let Some(Overlay::Prompt {
        inputs,
        title,
        focus_index,
        ..
    }) = &app.overlay
    else {
        return;
    };

    let height = inputs.len() as u16 + 2;
    let area = centered_rect(50, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints(inputs.iter().map(|_| Constraint::Length(1)))
        .split(inner);

    for (i, (name, input)) in inputs.iter().enumerate() {
        frame.render_widget(
            Paragraph::new(format!("{}: {}_", name, input)).style(Style::default().fg(
                if i == *focus_index {
                    Color::Yellow
                } else {
                    Color::White
                },
            )),
            fields[i],
        )
    }
}

fn render_confirm(frame: &mut Frame, message: &str, yes_selected: bool) {
    let area = centered_rect(50, 5, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let selected = Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);
    let normal = Style::default().fg(Color::White);

    let buttons = Line::from(vec![
        Span::styled("  No  ", if yes_selected { normal } else { selected }),
        Span::raw("    "),
        Span::styled("  Yes  ", if yes_selected { selected } else { normal }),
    ]);

    let text = Text::from(vec![
        Line::from(message.to_string()),
        Line::from(""),
        buttons,
    ]);

    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}

const ALL_SCREENS_KEYS: &[(&str, &str)] = &[
    ("q", "quit (not while a popup is open)"),
    ("?", "help"),
    ("arrow up/down", "move selection"),
];

const CONNECTIONS_KEYS: &[(&str, &str)] = &[
    ("Enter", "connect to instance"),
    ("Ctrl+A", "add connection"),
    ("e", "edit connection"),
    ("d", "delete connection"),
];

const DATABASES_KEYS: &[(&str, &str)] = &[
    ("Enter", "open database"),
    ("a", "new database (first collection required)"),
    ("d", "drop database"),
    ("Esc", "disconnect, back to connections"),
];

const COLLECTIONS_KEYS: &[(&str, &str)] = &[
    ("Enter", "open collection"),
    ("a", "new collection"),
    ("d", "drop collection"),
    ("Esc", "back to databases list"),
];

const DOCUMENTS_KEYS: &[(&str, &str)] = &[
    ("Enter", "open document"),
    ("arrow left/right", "previous/next page"),
    ("/", "filter"),
    ("i", "insert document (opens $EDITOR)"),
    ("d", "drop document"),
    ("Esc", "clear filter if set, otherwise back to collections"),
];

const DETAIL_KEYS: &[(&str, &str)] = &[
    ("Enter or arrow left/right", "expand/collapse field"),
    ("+/-", "expand/collapse all fields"),
    ("e", "edit document (opens $EDITOR)"),
    ("Esc", "back to documents"),
];

fn render_information_box(frame: &mut Frame, app: &App) {
    let mut commands = ALL_SCREENS_KEYS.to_vec();

    match app.screen {
        Screen::Connections => commands.extend_from_slice(CONNECTIONS_KEYS),
        Screen::Databases { .. } => commands.extend_from_slice(DATABASES_KEYS),
        Screen::Collections { .. } => commands.extend_from_slice(COLLECTIONS_KEYS),
        Screen::Documents { .. } => commands.extend_from_slice(DOCUMENTS_KEYS),
        Screen::Detail { .. } => commands.extend_from_slice(DETAIL_KEYS),
    }

    let height = commands.len() as u16 + 2;
    let area = centered_rect(60, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("help".to_string())
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints(commands.iter().map(|_| Constraint::Length(1)))
        .split(inner);

    for (i, (key, action)) in commands.iter().enumerate() {
        frame.render_widget(Paragraph::new(*key), fields[i]);
        frame.render_widget(
            Paragraph::new(*action).alignment(Alignment::Right),
            fields[i],
        );
    }
}
