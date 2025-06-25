// ===================================================================
// Arch System Suite - v3.1.0 - Main Application Source
// ===================================================================
// This is a pure-Rust application for managing Arch Linux systems.
// It uses the `ratatui` library to create a professional and intuitive
// terminal user interface.

// --- Crate Imports ---
// We import necessary components from our dependencies listed in Cargo.toml.
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::{self, stdout, Stdout, Write};
use std::process::{Command, Stdio};
use textwrap::wrap;

// --- Application Model & State Management ---

/// Represents the different "screens" or "views" of our application.
/// The `current_view` in the `App` struct will determine which view is rendered.
#[derive(Clone, Copy, PartialEq, Debug)]
enum AppView {
    MainMenu,
    HelpManual,
    // As you build out features, you'll add more views here.
    // e.g., Replicator, Cloner, Utilities
}

/// A generic struct to manage the state of any selectable list in the UI.
/// It holds the items and, crucially, a `ListState` which tracks the selected item.
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    /// Creates a new list with a given set of items.
    fn with_items(items: Vec<T>) -> Self {
        let mut list = Self { state: ListState::default(), items };
        list.state.select(Some(0)); // Select the first item by default.
        list
    }

    /// Moves the selection to the next item, wrapping around at the end.
    fn next(&mut self) {
        let i = self.state.selected().map_or(0, |i| {
            if i >= self.items.len() - 1 { 0 } else { i + 1 }
        });
        self.state.select(Some(i));
    }

    /// Moves the selection to the previous item, wrapping around at the beginning.
    fn previous(&mut self) {
        let i = self.state.selected().map_or(0, |i| {
            if i == 0 { self.items.len() - 1 } else { i - 1 }
        });
        self.state.select(Some(i));
    }
}

/// Represents a single, selectable item in our main menu.
/// It bundles the display text with the help text and the action to perform.
struct MenuItem<'a> {
    icon: &'a str,
    text: &'a str,
    help: &'a str,
    /// A function pointer that takes the app state and performs an action.
    /// This is a clean way to bind behavior directly to menu items.
    action: fn(&mut App) -> Result<()>,
}

/// The main application struct. It holds all the state needed to run the UI.
struct App<'a> {
    current_view: AppView,
    show_help_popup: bool,
    should_quit: bool,
    main_menu: StatefulList<MenuItem<'a>>,
}

impl<'a> App<'a> {
    /// Creates the initial state of the application.
    fn new() -> Self {
        App {
            current_view: AppView::MainMenu,
            show_help_popup: false,
            should_quit: false,
            main_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[R]", text: "Replicator (Recommended)", help: "The Replicator captures the 'recipe' for your system (packages, configs) into a single file. Use this to perform a clean, fresh installation on new hardware that perfectly matches your setup.", action: |_| { /* TODO: Switch to Replicator view */ Ok(()) } },
                MenuItem { icon: "[C]", text: "Cloner (Advanced)", help: "The Cloner creates a direct, 1:1 bootable ISO image of your current system's disk. This is best for creating a full backup or moving your exact OS to identical hardware.", action: |_| { /* TODO: Switch to Cloner view */ Ok(()) } },
                MenuItem { icon: "[U]", text: "Utilities & Manual Tools", help: "A collection of essential tools for system maintenance, including a hardware inspector, USB flasher, and manual installation steps.", action: |_| { /* TODO: Switch to Utilities view */ Ok(()) } },
                MenuItem { icon: "[H]", text: "Main Help", help: "Displays the main, scrollable help manual for the entire application.", action: |app| { app.current_view = AppView::HelpManual; Ok(()) } },
                MenuItem { icon: "[Q]", text: "Quit", help: "Exits the Arch System Suite application.", action: |app| { app.should_quit = true; Ok(()) } },
            ]),
        }
    }
}

// --- Main Application Entry & Loop ---

/// The main entry point for the application.
/// The `#[tokio::main]` attribute sets up the asynchronous runtime.
#[tokio::main]
async fn main() -> Result<()> {
    // First, check for external command-line dependencies before starting the TUI.
    if !check_and_install_dependencies().await? {
        println!("Cannot proceed without dependencies. Aborting.");
        return Ok(());
    }

    // Set up the terminal for TUI rendering.
    let mut terminal = init_terminal()?;
    let mut app = App::new();

    // Run the main application loop.
    run_app(&mut terminal, &mut app).await?;

    // Restore the terminal to its original state upon exit.
    restore_terminal(&mut terminal)?;
    Ok(())
}

/// The core event loop of the application.
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        // Draw the UI on each iteration.
        terminal.draw(|f| ui(f, app))?;

        // Wait for a key press event.
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key presses, not releases.
                if key.kind == KeyEventKind::Press {
                    handle_input(key.code, app).await?;
                }
            }
        }
    }
    Ok(())
}

/// Dispatches key events to the correct handler based on the current view.
async fn handle_input(key_code: KeyCode, app: &mut App) -> Result<()> {
    // Global keybinding: '?' toggles the help pop-up.
    if key_code == KeyCode::Char('?') {
        app.show_help_popup = !app.show_help_popup;
        return Ok(());
    }
    // If the help pop-up is open, any key press will close it.
    if app.show_help_popup {
        app.show_help_popup = false;
        return Ok(());
    }

    // Route input to the handler for the currently active view.
    match app.current_view {
        AppView::MainMenu => handle_main_menu_input(key_code, app).await?,
        AppView::HelpManual => {
            if key_code == KeyCode::Char('q') || key_code == KeyCode::Esc {
                app.current_view = AppView::MainMenu;
            }
        }
    }
    Ok(())
}

/// Handles key presses specifically for the MainMenu view.
async fn handle_main_menu_input(key_code: KeyCode, app: &mut App) -> Result<()> {
    match key_code {
        KeyCode::Char('q') => app.should_quit = true,
        // Vim-style navigation
        KeyCode::Char('k') | KeyCode::Up => app.main_menu.previous(),
        KeyCode::Char('j') | KeyCode::Down => app.main_menu.next(),
        // Select an item
        KeyCode::Enter => {
            if let Some(selected) = app.main_menu.state.selected() {
                let action = app.main_menu.items[selected].action;
                // Execute the action associated with the selected menu item.
                action(app)?;
            }
        }
        _ => {}
    }
    Ok(())
}

// --- Dependency Management ---

/// Checks for required external command-line tools and prompts to install them if missing.
/// This makes the application self-contained and user-friendly on fresh systems.
async fn check_and_install_dependencies() -> Result<bool> {
    let deps = ["gum", "arch-install-scripts", "pacman-contrib", "gptfdisk", "dosfstools", "e2fsprogs", "archiso", "rsync", "pciutils"];
    let mut missing_deps = Vec::new();

    println!("Checking dependencies...");
    for dep in &deps {
        let status = Command::new("pacman").arg("-Q").arg(dep).stdout(Stdio::null()).stderr(Stdio::null()).status()?;
        if !status.success() {
            missing_deps.push(*dep);
        }
    }

    if missing_deps.is_empty() {
        println!("✅ All dependencies are satisfied.");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Ok(true);
    }

    println!("\n⚠️ The following required packages are missing: {}", missing_deps.join(", "));
    print!("Would you like to install them now with sudo? (y/N) ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        println!("Attempting to install missing packages...");
        let mut args = vec!["-Syu", "--noconfirm", "--needed"];
        args.extend_from_slice(&missing_deps);
        let mut child = Command::new("sudo").args(&args).spawn().context("Failed to run sudo pacman. Do you have sudo privileges?")?;
        let status = child.wait().await?;
        if status.success() {
            println!("✅ Dependencies installed successfully.");
            Ok(true)
        } else {
            println!("❌ Failed to install dependencies. Please try installing them manually.");
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

// --- UI Rendering ---

/// The main UI drawing function. It delegates to other functions based on the current view.
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let main_layout = Layout::default().constraints([Constraint::Percentage(100)]).split(f.size());

    match app.current_view {
        AppView::MainMenu => render_main_menu(f, app, main_layout[0]),
        AppView::HelpManual => render_help_manual(f, main_layout[0]),
    }

    if app.show_help_popup {
        render_help_popup(f, app);
    }
}

/// Renders the Main Menu screen.
fn render_main_menu<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let chunks = Layout::default().direction(Direction::Vertical).margin(2)
        .constraints([Constraint::Length(8), Constraint::Min(0), Constraint::Length(1)]).split(area);
    
    let art = Paragraph::new(ASCII_ART).style(Style::default().fg(Color::Rgb(110, 125, 224))).alignment(Alignment::Center);
    f.render_widget(art, chunks[0]);

    let items: Vec<ListItem> = app.main_menu.items.iter()
        .map(|i| ListItem::new(format!("{} {}", i.icon, i.text)).style(Style::default().fg(Color::White)))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .highlight_style(Style::default().bg(Color::Rgb(60, 60, 90)).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[1], &mut app.main_menu.state);

    let status = Paragraph::new("v3.1.0 | Press 'j'/'k' to navigate | 'Enter' to select | '?' for help | 'q' to quit").alignment(Alignment::Center);
    f.render_widget(status, chunks[2]);
}

/// Renders the full-screen help manual.
fn render_help_manual<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = "This is the main help page for Arch System Suite v3.1.0.\n\nIt would contain detailed sections on the Replicator, Cloner, and all Utilities, explaining each feature in depth.\n\nPress 'q' or 'Esc' to return to the main menu.";
    let paragraph = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help Manual")).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Renders the context-aware help pop-up window over the current view.
fn render_help_popup<B: Backend>(f: &mut Frame<B>, app: &App) {
    let help_text = match app.current_view {
        AppView::MainMenu => app.main_menu.items[app.main_menu.state.selected().unwrap_or(0)].help,
        AppView::HelpManual => "This is the main help page. Use 'q' or 'Esc' to return to the previous menu.",
    };

    let block = Block::default().title("Context Help").borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(60, 40, f.size());
    let wrapped_text = wrap(help_text, (area.width - 4) as usize);
    let paragraph = Paragraph::new(wrapped_text).block(block);

    f.render_widget(Clear, area); // This clears the background, making it a true pop-up.
    f.render_widget(paragraph, area);
}

// --- Terminal Setup & Helpers ---

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

/// Helper function to create a centered rectangle for pop-up windows.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)])
        .split(r);
    Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)])
        .split(popup_layout[1])[1]
}

const ASCII_ART: &str = r"
    █████╗  ██████╗  ██████╗██╗  ██╗     ███████╗██╗   ██╗██╗████████╗███████╗
   ██╔══██╗██╔════╝ ██╔════╝██║  ██║     ██╔════╝██║   ██║██║╚══██╔══╝██╔════╝
   ███████║██║  ███╗██║     ███████║     ███████╗██║   ██║██║   ██║   ███████╗
   ██╔══██║██║   ██║██║     ██╔══██║     ╚════██║██║   ██║██║   ██║   ╚════██║
   ██║  ██║╚██████╔╝╚██████╗██║  ██║     ███████║╚██████╔╝██║   ██║   ███████║
   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝     ╚══════╝ ╚═════╝ ╚═╝   ╚═╝   ╚══════╝
";
