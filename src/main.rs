use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment, Flex, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, BorderType, Clear},
    Frame, Terminal,
};

pub struct App {
    show_add_popup: bool,
    
    input_buffer: String,
    
    connection: Option<String>,
    database: Option<String>,
    collection: Option<String>,
}

fn main() -> io::Result<()> {
    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // app
    let mut app = App {
        show_add_popup: false,
        input_buffer: String::new(),
        connection: None,
        database: None,
        collection: None,
    };

    // main loop
    let result = run(&mut terminal, &mut app);

    // terminal teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.show_add_popup {
                match key.code {
                    KeyCode::Esc => {
                        app.show_add_popup = false;
                        app.input_buffer.clear();
                    }
                    KeyCode::Enter => {
                        app.show_add_popup = false;
                        app.input_buffer.clear();
                    }
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    KeyCode::Char(c) => {
                        app.input_buffer.push(c);
                    }
                    _ => {}
                }
            } else {
                match key.code{
                    KeyCode::Char('q') => break,
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.show_add_popup = true;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    // main content placeholder
    let main_block = Block::default().borders(Borders::ALL);
    render_connections_box(frame, main_area);
    render_status_bar(frame, status_area, app);
    
    if app.show_add_popup {
        render_app_popup(frame, app);
    }
}

fn render_connections_box(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("connections")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hint = Paragraph::new("Ctrl+A to add a new connection")
        .alignment(Alignment::Center);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
        .split(inner);

    frame.render_widget(hint, vertical[1]);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let status_text = format!(
        " conn: {} | db: {} | coll: {} ",
        app.connection.as_deref().unwrap_or("-"),
        app.database.as_deref().unwrap_or("-"),
        app.collection.as_deref().unwrap_or("-"),
    );

    let status_bar = Paragraph::new(Line::from(status_text))
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(status_bar, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0]);

    horizontal[0]
}

fn render_app_popup(frame: &mut Frame, app: &App) {
    let popup_area = centered_rect(60, 5, frame.area());

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("new connection")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = Paragraph::new(format!("{}_", app.input_buffer))
        .style(Style::default().fg(Color::White));

    frame.render_widget(text, inner);
}

