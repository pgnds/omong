use std::io;
use std::process::Command;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// opens `initial` in $EDITOR. Ok(None) means nothing changed.
pub fn edit_text(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    initial: &str,
) -> Result<Option<String>, String> {
    let path = std::env::temp_dir().join(format!("omong-{}.json", std::process::id()));
    std::fs::write(&path, initial).map_err(|e| e.to_string())?;

    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");

    // hand the terminal over to the editor
    disable_raw_mode().map_err(|e| e.to_string())?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| e.to_string())?;

    let status = Command::new(program).args(parts).arg(&path).status();

    // take it back, whatever happened
    let _ = enable_raw_mode();
    let _ = execute!(terminal.backend_mut(), EnterAlternateScreen);
    let _ = terminal.clear(); // forces a full redraw, ratatui thinks the screen is unchanged

    let status = status.map_err(|e| format!("could not start '{}': {}", program, e))?;
    if !status.success() {
        let _ = std::fs::remove_file(&path);
        return Err("editor exited with an error, edit discarded".to_string());
    }

    let edited = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&path);

    Ok(if edited == initial {
        None
    } else {
        Some(edited)
    })
}
