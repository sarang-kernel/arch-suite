// ===================================================================
// Application State Module
// ===================================================================
// This module defines the core data structures that hold the entire
// state of the application. It is the single source of truth.

use crate::actions;
use anyhow::Result;
use ratatui::widgets::ListState;
use std::future::Future;
use std::pin::Pin;
use tui_input::Input;

// --- Enums for State Management ---

/// Represents the different "screens" or "views" of our application.
/// The `current_view` in the `App` struct will determine which view is rendered
/// and how input is handled.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AppView {
    MainMenu,
    HelpManual,
    Replicator,
    Cloner,
    Utilities,
    // A generic view for showing the output of a completed action.
    ActionPopup,
}

/// Represents the current input mode, determining how keyboard events are handled.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputMode {
    /// Normal mode: 'j'/'k' navigate lists, 'Enter' selects.
    Normal,
    /// Editing mode: Keyboard input is captured into a text field.
    Editing,
}

// --- Core Application Structs ---

/// A generic struct to manage the state of any selectable list in the UI.
/// It holds the items and, crucially, a `ListState` which tracks the selected item.
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        let mut list = Self { state: ListState::default(), items };
        list.state.select(Some(0)); // Select first item by default
        list
    }
    pub fn next(&mut self) {
        let i = self.state.selected().map_or(0, |i| {
            if i >= self.items.len() - 1 { 0 } else { i + 1 }
        });
        self.state.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = self.state.selected().map_or(0, |i| {
            if i == 0 { self.items.len() - 1 } else { i - 1 }
        });
        self.state.select(Some(i));
    }
}

/// A function pointer type for actions that can be executed from a menu.
/// It returns a Future that resolves to a String result, which is displayed in a popup.
pub type AppAction = Pin<Box<dyn Future<Output = Result<String>>>>;
pub type ActionFn = fn() -> AppAction;

/// Represents a single, selectable item in our menus.
pub struct MenuItem<'a> {
    pub icon: &'a str,
    pub text: &'a str,
    pub help: &'a str,
    pub action: ActionFn,
}

/// The main application struct. It holds all state needed to run the UI.
pub struct App<'a> {
    // View and Mode
    pub current_view: AppView,
    pub input_mode: InputMode,
    pub show_help_popup: bool,
    pub should_quit: bool,
    
    // UI State for different views
    pub main_menu: StatefulList<MenuItem<'a>>,
    pub replicator_menu: StatefulList<MenuItem<'a>>,
    pub cloner_menu: StatefulList<MenuItem<'a>>,
    pub utilities_menu: StatefulList<MenuItem<'a>>,
    
    // Data for Popups
    pub popup_title: String,
    pub popup_text: String,
    pub input: Input, // From the tui-input crate
}

impl<'a> App<'a> {
    /// Creates the initial state of the application.
    pub fn new() -> Self {
        App {
            current_view: AppView::MainMenu,
            input_mode: InputMode::Normal,
            show_help_popup: false,
            should_quit: false,
            main_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[R]", text: "Replicator (Recommended)", help: "The Replicator captures the 'recipe' for your system to perform a clean, fresh installation on new hardware.", action: || Box::pin(async { Err(anyhow::anyhow!("view:replicator")) }) },
                MenuItem { icon: "[C]", text: "Cloner (Advanced)", help: "The Cloner creates a direct, 1:1 bootable ISO image of your current system. Best for backups or identical hardware.", action: || Box::pin(async { Err(anyhow::anyhow!("view:cloner")) }) },
                MenuItem { icon: "[U]", text: "Utilities & Manual Tools", help: "Essential tools for system maintenance, including a hardware inspector, USB flasher, and manual installation steps.", action: || Box::pin(async { Err(anyhow::anyhow!("view:utilities")) }) },
                MenuItem { icon: "[H]", text: "Main Help", help: "Displays the main, scrollable help manual for the entire application.", action: || Box::pin(async { Err(anyhow::anyhow!("view:help")) }) },
                MenuItem { icon: "[Q]", text: "Quit", help: "Exits the Arch System Suite application.", action: || Box::pin(async { Err(anyhow::anyhow!("quit")) }) },
            ]),
            replicator_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[S]", text: "Create System Snapshot", help: "Gathers package lists, /etc configs, and dotfiles into a single snapshot file.", action: actions::action_create_snapshot },
                MenuItem { icon: "[D]", text: "Deploy from Snapshot", help: "Performs a fresh Arch install and applies a snapshot file to replicate a system.", action: || Box::pin(async { Ok("Deploy not yet implemented.".to_string()) }) },
            ]),
            cloner_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[I]", text: "Create Bootable ISO", help: "Creates a bootable .iso file from the current system state using 'archiso'.", action: || Box::pin(async { Ok("Create ISO not yet implemented.".to_string()) }) },
            ]),
            utilities_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[H]", text: "System Inspector & Prep", help: "Detects CPU/GPU and prepares a list of recommended drivers for installation.", action: || Box::pin(async { Ok("Inspector not yet implemented.".to_string()) }) },
                MenuItem { icon: "[F]", text: "Flash ISO to USB", help: "A safe wrapper around 'dd' to burn any .iso file to a USB drive.", action: || Box::pin(async { Ok("Flasher not yet implemented.".to_string()) }) },
                MenuItem { icon: "[M]", text: "Manual Install Tools", help: "A step-by-step interface for advanced users to partition, format, and install.", action: || Box::pin(async { Ok("Manual Tools not yet implemented.".to_string()) }) },
            ]),
            popup_title: String::new(),
            popup_text: String::new(),
            input: Input::default(),
        }
    }

    /// Resets the input popup state.
    pub fn reset_popup(&mut self) {
        self.input_mode = InputMode::Normal;
        self.popup_title.clear();
        self.input.reset();
    }
}
