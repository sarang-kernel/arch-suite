// ===================================================================
// Arch System Suite - v3.1.2 - Main Application Source
// ===================================================================
// This is a pure-Rust application for managing Arch Linux systems.
// It uses the `ratatui` library to create a professional and intuitive
// terminal user interface. This version is fully self-contained and corrects
// all previous compiler errors related to lifetimes and borrowing.

// --- Crate Imports ---
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
use std::future::Future;
use std::io::{self, stdout, Stdout, Write};
use std::pin::Pin;
use std::process::Stdio;
use textwrap::wrap;
use tokio::process::Command;

// --- Application Model & State Management ---

#[derive(Clone, Copy, PartialEq, Debug)]
enum AppView {
    MainMenu,
    HelpManual,
    ActionPopup,
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> Self {
        let mut list = Self { state: ListState::default(), items };
        list.state.select(Some(0));
        list
    }
    fn next(&mut self) { let i = self.state.selected().map_or(0, |i| if i >= self.items.len() - 1 { 0 } else { i + 1 }); self.state.select(Some(i)); }
    fn previous(&mut self) { let i = self.state.selected().map_or(0, |i| if i == 0 { self.items.len() - 1 } else { i - 1 }); self.state.select(Some(i)); }
}

// FIX: The action function no longer takes a mutable reference to App.
// Instead, it's a self-contained unit of work that returns a String result.
// This is the key to solving the borrow checker errors.
type AppAction = Pin<Box<dyn Future<Output = Result<String>>>>;
type ActionFn = fn() -> AppAction;

struct MenuItem<'a> {
    icon: &'a str,
    text: &'a str,
    help: &'a str,
    action: ActionFn,
}

struct App<'a> {
    current_view: AppView,
    show_help_popup: bool,
    should_quit: bool,
    main_menu: StatefulList<MenuItem<'a>>,
    action_popup_text: String,
}

impl<'a> App<'a> {
    fn new() -> Self {
        App {
            current_view: AppView::MainMenu,
            show_help_popup: false,
            should_quit: false,
            main_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[R]", text: "Replicator (Recommended)", help: "Captures the 'recipe' for your system (packages, configs) into a single file. Use this to perform a clean, fresh installation on new hardware that perfectly matches your setup.", action: action_create_snapshot },
                MenuItem { icon: "[C]", text: "Cloner (Advanced)", help: "Creates a direct, 1:1 bootable ISO image of your current system's disk. This is best for creating a full backup or moving your exact OS to identical hardware.", action: || Box::pin(async { Ok("Cloner not yet implemented.".to_string()) }) },
                MenuItem { icon: "[U]", text: "Utilities & Manual Tools", help: "A collection of essential tools for system maintenance, including a hardware inspector, USB flasher, and manual installation steps.", action: || Box::pin(async { Ok("Utilities not yet implemented.".to_string()) }) },
                MenuItem { icon: "[H]", text: "Main Help", help: "Displays the main, scrollable help manual for the entire application.", action: || Box::pin(async { Err(anyhow::anyhow!("help")) }) }, // Special case
                MenuItem { icon: "[Q]", text: "Quit", help: "Exits the Arch System Suite application.", action: || Box::pin(async { Err(anyhow::anyhow!("quit")) }) }, // Special case
            ]),
            action_popup_text: String::new(),
        }
    }
}

// --- Main Application Entry & Loop ---

#[tokio::main]
async fn main() -> Result<()> {
    if !check_and_install_dependencies().await? {
        println!("Cannot proceed without dependencies. Aborting.");
        return Ok(());
    }
    let mut terminal = init_terminal()?;
    let mut app = App::new();
    run_app(&mut terminal, &mut app).await?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App<'_>) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_input(key.code, app).await?;
                }
            }
        }
    }
    Ok(())
}

async fn handle_input(key_code: KeyCode, app: &mut App<'_>) -> Result<()> {
    if key_code == KeyCode::Char('?') {
        app.show_help_popup = !app.show_help_popup;
        return Ok(());
    }
    if app.show_help_popup {
        app.show_help_popup = false;
        return Ok(());
    }
    match app.current_view {
        AppView::MainMenu => handle_main_menu_input(key_code, app).await?,
        AppView::HelpManual | AppView::ActionPopup => {
            app.current_view = AppView::MainMenu;
        }
    }
    Ok(())
}

async fn handle_main_menu_input(key_code: KeyCode, app: &mut App<'_>) -> Result<()> {
    match key_code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('k') | KeyCode::Up => app.main_menu.previous(),
        KeyCode::Char('j') | KeyCode::Down => app.main_menu.next(),
        KeyCode::Enter => {
            if let Some(selected) = app.main_menu.state.selected() {
                let action = app.main_menu.items[selected].action;
                
                // FIX: The action is called, and its result is awaited.
                // The borrow of `app` for the action is contained within the `action()` call.
                match action().await {
                    Ok(message) => {
                        // AFTER the await, we can now safely mutate `app`.
                        app.action_popup_text = message;
                        app.current_view = AppView::ActionPopup;
                    }
                    Err(e) => {
                        // Handle special "quit" and "help" signals.
                        let msg = e.to_string();
                        if msg == "quit" {
                            app.should_quit = true;
                        } else if msg == "help" {
                            app.current_view = AppView::HelpManual;
                        } else {
                            app.action_popup_text = format!("Error: {}", e);
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

// --- Core Logic Functions ---

fn action_create_snapshot() -> AppAction {
    Box::pin(async {
        let home_dir = std::env::var("HOME").context("Failed to get HOME directory")?;
        let work_dir = format!("{}/arch-suite-work", home_dir);
        let snapshot_dir = format!("{}/snapshot_tmp", work_dir);
        let snapshot_file = format!("{}/snapshot-{}.tar.gz", work_dir, chrono::Local::now().format("%Y%m%d"));

        std::fs::create_dir_all(&snapshot_dir)?;

        let command_script = format!(
            "echo 'Gathering package lists...'; \
             pacman -Qqe > {0}/packages.x86_64.txt && \
             pacman -Qqm > {0}/packages.foreign.txt && \
             echo 'Archiving /etc...'; \
             sudo tar -czf {0}/etc.tar.gz /etc && \
             echo 'Archiving dotfiles...'; \
             tar -czf {0}/home.tar.gz -C {1} --exclude='.cache' . && \
             echo 'Creating final snapshot...'; \
             tar -czf {2} -C {0} . && \
             sudo rm -rf {0}",
            snapshot_dir, home_dir, snapshot_file
        );

        let output = Command::new("sh").arg("-c").arg(command_script).output().await?;

        if output.status.success() {
            Ok(format!("✅ Snapshot created successfully:\n{}", snapshot_file))
        } else {
            Err(anyhow::anyhow!("Failed to create snapshot:\n{}", String::from_utf8_lossy(&output.stderr)))
        }
    })
}

// --- Dependency Management ---

async fn check_and_install_dependencies() -> Result<bool> {
    let deps = ["gum", "arch-install-scripts", "pacman-contrib", "gptfdisk", "dosfstools", "e2fsprogs", "archiso", "rsync", "pciutils"];
    let mut missing_deps = Vec::new();
    println!("Checking dependencies...");
    for dep in &deps {
        let status = Command::new("pacman").arg("-Q").arg(dep).stdout(Stdio::null()).stderr(Stdio::null()).status().await?;
        if !status.success() { missing_deps.push(*dep); }
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
        if status.success() { println!("✅ Dependencies installed successfully."); Ok(true) } 
        else { println!("❌ Failed to install dependencies. Please try installing them manually."); Ok(false) }
    } else { Ok(false) }
}

// --- UI Rendering ---

fn ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default().constraints([Constraint::Percentage(100)]).split(f.size());
    match app.current_view {
        AppView::MainMenu | AppView::ActionPopup => render_main_menu(f, app, main_layout[0]),
        AppView::HelpManual => render_help_manual(f, main_layout[0]),
    }
    if app.show_help_popup { render_help_popup(f, app); }
    if app.current_view == AppView::ActionPopup { render_action_popup(f, app); }
}

fn render_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
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
    let status = Paragraph::new("v3.1.2 | Press 'j'/'k' to navigate | 'Enter' to select | '?' for help | 'q' to quit").alignment(Alignment::Center);
    f.render_widget(status, chunks[2]);
}

fn render_help_manual(f: &mut Frame, area: Rect) {
    let help_text = "This is the main help page for Arch System Suite v3.1.2.\n\nIt would contain detailed sections on the Replicator, Cloner, and all Utilities, explaining each feature in depth.\n\nPress 'q' or 'Esc' to return to the main menu.";
    let paragraph = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help Manual")).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_help_popup(f: &mut Frame, app: &App<'_>) {
    let help_text = match app.current_view {
        AppView::MainMenu | AppView::ActionPopup => app.main_menu.items[app.main_menu.state.selected().unwrap_or(0)].help,
        AppView::HelpManual => "This is the main help page. Use 'q' or 'Esc' to return to the previous menu.",
    };
    let block = Block::default().title("Context Help").borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(60, 40, f.size());
    let wrapped_text: Vec<Line> = wrap(help_text, (area.width - 4) as usize).iter().map(|s| Line::from(s.to_string())).collect();
    let paragraph = Paragraph::new(wrapped_text).block(block);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_action_popup(f: &mut Frame, app: &App<'_>) {
    let block = Block::default().title("Action Result").borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(80, 50, f.size());
    let wrapped_text: Vec<Line> = wrap(&app.action_popup_text, (area.width - 4) as usize).iter().map(|s| Line::from(s.to_string())).collect();
    let paragraph = Paragraph::new(wrapped_text).block(block);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

// --- Terminal Setup & Helpers ---
fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> { stdout().execute(EnterAlternateScreen)?; enable_raw_mode()?; let backend = CrosstermBackend::new(stdout()); let terminal = Terminal::new(backend)?; Ok(terminal) }
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> { disable_raw_mode()?; terminal.backend_mut().execute(LeaveAlternateScreen)?; terminal.show_cursor()?; Ok(()) }
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect { let popup_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)]).split(r); Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)]).split(popup_layout[1])[1] }

const ASCII_ART: &str = r"
    █████╗  ██████╗  ██████╗██╗  ██╗     ███████╗██╗   ██╗██╗████████╗███████╗
   ██╔══██╗██╔════╝ ██╔════╝██║  ██║     ██╔════╝██║   ██║██║╚══██╔══╝██╔════╝
   ███████║██║  ███╗██║     ███████║     ███████╗██║   ██║██║   ██║   ███████╗
   ██╔══██║██║   ██║██║     ██╔══██║     ╚════██║██║   ██║██║   ██║   ╚════██║
   ██║  ██║╚██████╔╝╚██████╗██║  ██║     ███████║╚██████╔╝██║   ██║   ███████║
   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝     ╚══════╝ ╚═════╝ ╚═╝   ╚═╝   ╚══════╝
";
