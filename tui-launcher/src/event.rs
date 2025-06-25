// ===================================================================
// Event Handling Module
// ===================================================================
// This module is responsible for handling all user input and dispatching
// actions based on the application's current state (view and input mode).

use crate::app::{App, AppAction, AppView, InputMode, MenuItem, StatefulList};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use tui_input::backend::crossterm::EventHandler;

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
    // FIX: Decouple the list from the main app state to resolve borrowing errors.
    // We get the action from the menu handler and then execute it, modifying the app state.
    let action_to_perform = match app.current_view {
        AppView::MainMenu => handle_menu_keys(&mut app.main_menu, key_event.code),
        AppView::Replicator => handle_menu_keys(&mut app.replicator_menu, key_event.code),
        AppView::Cloner => handle_menu_keys(&mut app.cloner_menu, key_event.code),
        AppView::Utilities => handle_menu_keys(&mut app.utilities_menu, key_event.code),
        AppView::HelpManual | AppView::ActionPopup => {
            if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Esc || key_event.code == KeyCode::Enter {
                app.current_view = AppView::MainMenu;
            }
            None // No action to perform
        }
    };

    if let Some(action) = action_to_perform {
        match action().await {
            Ok(message) => {
                app.popup_text = message;
                app.current_view = AppView::ActionPopup;
            }
            Err(e) => {
                let msg = e.to_string();
                if msg == "quit" {
                    app.should_quit = true;
                } else if msg.starts_with("view:") {
                    match msg.as_str() {
                        "view:help" => app.current_view = AppView::HelpManual,
                        "view:replicator" => app.current_view = AppView::Replicator,
                        "view:cloner" => app.current_view = AppView::Cloner,
                        "view:utilities" => app.current_view = AppView::Utilities,
                        _ => {}
                    }
                } else {
                    app.popup_text = format!("Error: {}", e);
                    app.current_view = AppView::ActionPopup;
                }
            }
        }
    } else if key_event.code == KeyCode::Esc {
        // If Esc was pressed in a submenu and no action was returned, go back to main menu.
        if app.current_view != AppView::MainMenu {
            app.current_view = AppView::MainMenu;
        }
    }

    Ok(())
}

/// A generic handler for any menu. It now returns an optional action to be performed.
fn handle_menu_keys<'a>(list: &mut StatefulList<MenuItem<'a>>, key_code: KeyCode) -> Option<fn() -> AppAction> {
    match key_code {
        KeyCode::Char('k') | KeyCode::Up => list.previous(),
        KeyCode::Char('j') | KeyCode::Down => list.next(),
        KeyCode::Enter => {
            if let Some(selected) = list.state.selected() {
                return Some(list.items[selected].action);
            }
        }
        _ => {}
    }
    None
}

/// Handles key events when the app is in Editing (text input) mode.
async fn handle_editing_mode_event(app: &mut App<'_>, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Enter => {
            let user_input = app.input.value().to_string();
            // TODO: Process the input. For now, just display it.
            app.popup_text = format!("You entered: {}", user_input);
            app.current_view = AppView::ActionPopup;
            app.reset_popup();
        }
        KeyCode::Esc => {
            app.reset_popup();
        }
        _ => {
            app.input.handle_event(&Event::Key(key_event));
        }
    }
    Ok(())
}
