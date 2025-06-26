// ===================================================================
// Arch System Suite - v4.0.0 - Main Entry Point
// ===================================================================
mod app;
mod event;
mod ui;
mod actions;
mod components;

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
    if !actions::check_and_install_dependencies().await? {
        println!("Cannot proceed without dependencies. Aborting.");
        return Ok(());
    }
    let mut terminal = init_terminal()?;
    let mut app = App::new();
    event::run_app(&mut terminal, &mut app).await?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
