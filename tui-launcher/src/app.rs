// ===================================================================
// Application State Module
// ===================================================================
// This module defines the core data structures that hold the entire
// state of the application.

use crate::actions;
use anyhow::Result;
use ratatui::widgets::ListState;
use std::future::Future;
use std::pin::Pin;
use tui_input::Input;

// --- Enums for State Management ---
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AppView {
    MainMenu,
    HelpManual,
    ActionPopup,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputMode {
    Normal,
    Editing,
}

// --- Core Application Structs ---
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        let mut list = Self { state: ListState::default(), items };
        list.state.select(Some(0));
        list
    }
    pub fn next(&mut self) { let i = self.state.selected().map_or(0, |i| if i >= self.items.len() - 1 { 0 } else { i + 1 }); self.state.select(Some(i)); }
    pub fn previous(&mut self) { let i = self.state.selected().map_or(0, |i| if i == 0 { self.items.len() - 1 } else { i - 1 }); self.state.select(Some(i)); }
}

pub type AppAction = Pin<Box<dyn Future<Output = Result<String>>>>;
pub type ActionFn = fn() -> AppAction;

pub struct MenuItem<'a> {
    pub icon: &'a str,
    pub text: &'a str,
    pub help: &'a str,
    pub action: ActionFn,
}

pub struct App<'a> {
    pub current_view: AppView,
    pub input_mode: InputMode,
    pub show_help_popup: bool,
    pub should_quit: bool,
    pub main_menu: StatefulList<MenuItem<'a>>,
    pub popup_title: String,
    pub popup_text: String,
    pub input: Input,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            current_view: AppView::MainMenu,
            input_mode: InputMode::Normal,
            show_help_popup: false,
            should_quit: false,
            main_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[R]", text: "Replicator (Recommended)", help: "Captures the 'recipe' of your system to perform a clean, fresh installation on new hardware.", action: actions::action_create_snapshot },
                MenuItem { icon: "[C]", text: "Cloner (Advanced)", help: "Creates a direct, 1:1 bootable ISO image of your current system. Best for backups or identical hardware.", action: || Box::pin(async { Ok("Cloner not yet implemented.".to_string()) }) },
                MenuItem { icon: "[U]", text: "Utilities & Manual Tools", help: "Essential tools for system maintenance, including a hardware inspector, USB flasher, and manual installation steps.", action: || Box::pin(async { Ok("Utilities not yet implemented.".to_string()) }) },
                MenuItem { icon: "[H]", text: "Main Help", help: "Displays the main, scrollable help manual for the entire application.", action: || Box::pin(async { Err(anyhow::anyhow!("help")) }) },
                MenuItem { icon: "[Q]", text: "Quit", help: "Exits the Arch System Suite application.", action: || Box::pin(async { Err(anyhow::anyhow!("quit")) }) },
            ]),
            popup_title: String::new(),
            popup_text: String::new(),
            input: Input::default(),
        }
    }

    pub fn reset_popup(&mut self) {
        self.input_mode = InputMode::Normal;
        self.popup_title.clear();
        self.input.reset();
    }
}
