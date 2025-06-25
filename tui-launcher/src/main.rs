// ===================================================================
// Arch System Suite - v3.2.0 - Main Entry Point
// ===================================================================
// This file is the main entry point. Its sole responsibility is to
// initialize the application state, set up the terminal, and start
// the main event loop. All complex logic is delegated to other modules.

mod app;
mod event;
mod ui;
mod actions;

use anyhow::Result;
use app::App;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use std::io::{self, stdout, Stdout};

#[tokio::main]
async fn main() -> Result<()> {
    // First, check for external command-line dependencies before starting the TUI.
    if !actions::check_and_install_dependencies().await? {
        println!("Cannot proceed without dependencies. Aborting.");
        return Ok(());
    }

    // Set up the terminal for TUI rendering.
    let mut terminal = init_terminal()?;
    let mut app = App::new();

    // Run the main application loop.
    event::run_app(&mut terminal, &mut app).await?;

    // Restore the terminal to its original state upon exit.
    restore_terminal(&mut terminal)?;
    Ok(())
}

/// Initializes the terminal for TUI mode.
fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restores the terminal to its original state.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
