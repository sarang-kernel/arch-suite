// ===================================================================
// Event Handling Module
// ===================================================================
// This module is responsible for handling all user input and dispatching
// actions based on the application's current state (view and input mode).

use crate::app::{App, AppView, InputMode};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;

/// The core event loop of the application.
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App<'_>) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| crate::ui::ui(f, app))?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(app, key).await?;
                }
            }
        }
    }
    Ok(())
}

/// The main dispatcher for key events. It routes events based on the current input mode.
async fn handle_key_event(app: &mut App<'_>, key_event: KeyEvent) -> Result<()> {
    if key_event.code == KeyCode::Char('?') {
        app.show_help_popup = !app.show_help_popup;
        return Ok(());
    }
    if app.show_help_popup {
        app.show_help_popup = false;
        return Ok(());
    }
    match app.input_mode {
        InputMode::Normal => handle_normal_mode_event(app, key_event).await?,
        InputMode::Editing => handle_editing_mode_event(app, key_event).await?,
    }
    Ok(())
}

/// Handles key events when the app is in Normal (navigation) mode.
async fn handle_normal_mode_event(app: &mut App<'_>, key_event: KeyEvent) -> Result<()> {
    match app.current_view {
        AppView::MainMenu => handle_main_menu_keys(app, key_event.code).await?,
        AppView::HelpManual | AppView::ActionPopup => {
            if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Esc {
                app.current_view = AppView::MainMenu;
            }
        }
    }
    Ok(())
}

/// Handles key events for the Main Menu specifically.
async fn handle_main_menu_keys(app: &mut App<'_>, key_code: KeyCode) -> Result<()> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('k') | KeyCode::Up => app.main_menu.previous(),
        KeyCode::Char('j') | KeyCode::Down => app.main_menu.next(),
        KeyCode::Enter => {
            if let Some(selected) = app.main_menu.state.selected() {
                let action = app.main_menu.items[selected].action;
                match action().await {
                    Ok(message) => {
                        app.popup_text = message;
                        app.current_view = AppView::ActionPopup;
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg == "quit" {
                            app.should_quit = true;
                        } else if msg == "help" {
                            app.current_view = AppView::HelpManual;
                        } else {
                            app.popup_text = format!("Error: {}", e);
                            app.current_view = AppView::ActionPopup;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handles key events when the app is in Editing (text input) mode.
async fn handle_editing_mode_event(_app: &mut App<'_>, _key_event: KeyEvent) -> Result<()> {
    // This is where you would use the `tui-input` crate to handle input.
    // Example:
    // match _key_event.code {
    //     KeyCode::Enter => {
    //         let user_input = _app.input.value().to_string();
    //         // ... process the input ...
    //         _app.reset_popup();
    //     }
    //     KeyCode::Esc => {
    //         _app.reset_popup();
    //     }
    //     _ => {
    //         _app.input.handle_event(&Event::Key(_key_event));
    //     }
    // }
    Ok(())
}
