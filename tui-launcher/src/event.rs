// ===================================================================
// Event Handling Module
// ===================================================================
use crate::app::{Action, App, AppView, MenuItem, Popup, StatefulList};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use tui_input::backend::crossterm::EventHandler;

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

async fn handle_key_event(app: &mut App<'_>, key_event: KeyEvent) -> Result<()> {
    if app.active_popup != Popup::None {
        handle_popup_keys(app, key_event).await?;
        return Ok(());
    }
    if key_event.code == KeyCode::Char('?') {
        app.active_popup = Popup::Help;
        return Ok(());
    }
    
    let action_to_perform = match app.current_view {
        AppView::MainMenu => handle_menu_keys(&mut app.main_menu, key_event.code),
        AppView::Replicator => handle_menu_keys(&mut app.replicator_menu, key_event.code),
        AppView::Cloner => handle_menu_keys(&mut app.cloner_menu, key_event.code),
        AppView::Utilities => handle_menu_keys(&mut app.utilities_menu, key_event.code),
        AppView::ManualInstaller => handle_menu_keys(&mut app.manual_install_menu, key_event.code),
        AppView::HelpManual => {
            if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Esc {
                app.current_view = AppView::MainMenu;
            }
            None
        }
        _ => None,
    };

    if let Some(action) = action_to_perform {
        execute_action(app, action).await?;
    } else if key_event.code == KeyCode::Esc && app.current_view != AppView::MainMenu {
        app.current_view = AppView::MainMenu;
    }

    Ok(())
}

async fn handle_popup_keys(app: &mut App<'_>, key_event: KeyEvent) -> Result<()> {
    match app.active_popup {
        Popup::Help | Popup::Action => {
            app.active_popup = Popup::None;
        }
        Popup::Confirm => match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(action) = app.popup_action.take() {
                    execute_action(app, action).await?;
                }
                app.active_popup = Popup::None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.active_popup = Popup::None;
            }
            _ => {}
        },
        Popup::Input => match key_event.code {
            KeyCode::Enter => {
                if let Some(action) = app.popup_action.take() {
                    execute_action(app, action).await?;
                }
                app.active_popup = Popup::None;
            }
            KeyCode::Esc => {
                app.active_popup = Popup::None;
            }
            _ => {
                app.popup_input.handle_event(&Event::Key(key_event));
            }
        },
        Popup::Select => match key_event.code {
            KeyCode::Char('k') | KeyCode::Up => app.popup_list.previous(),
            KeyCode::Char('j') | KeyCode::Down => app.popup_list.next(),
            KeyCode::Enter => {
                if let Some(action) = app.popup_action.take() {
                    execute_action(app, action).await?;
                }
                app.active_popup = Popup::None;
            }
            KeyCode::Esc => {
                app.active_popup = Popup::None;
            }
            _ => {}
        },
        Popup::None => {}
    }
    Ok(())
}

fn handle_menu_keys<'a>(list: &mut StatefulList<MenuItem<'a>>, key_code: KeyCode) -> Option<Action> {
    match key_code {
        KeyCode::Char('k') | KeyCode::Up => list.previous(),
        KeyCode::Char('j') | KeyCode::Down => list.next(),
        KeyCode::Enter => {
            if let Some(item) = list.selected_item() {
                return Some(item.action.clone());
            }
        }
        _ => {}
    }
    None
}

async fn execute_action(app: &mut App<'_>, action: Action) -> Result<()> {
    match action {
        Action::Quit => app.should_quit = true,
        Action::SetView(view) => app.current_view = view,
        Action::Execute(func) => {
            app.popup_title = "Working...".to_string();
            app.popup_text = "Please wait while the task completes.".to_string();
            app.active_popup = Popup::Action;

            let mut terminal = crate::init_terminal()?;
            terminal.draw(|f| crate::ui::ui(f, app))?;

            match func().await {
                Ok(message) => {
                    app.popup_title = "Success".to_string();
                    app.popup_text = message;
                }
                Err(e) => {
                    app.popup_title = "Error".to_string();
                    app.popup_text = format!("An error occurred: {}", e);
                }
            }
        }
    }
    Ok(())
}
