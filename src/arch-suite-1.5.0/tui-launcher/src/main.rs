use std::io::{self, stdout, Stdout};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use textwrap::wrap;

// --- Application State & Model ---

// Represents the current view the user is looking at.
#[derive(Clone, Copy, PartialEq, Debug)]
enum AppView {
    MainMenu,
    HelpManual,
    // Add other views here as they are built
    // e.g., Replicator, Cloner, Utilities
}

// Holds the state for a selectable list (like our menus).
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut list = StatefulList { state: ListState::default(), items };
        list.state.select(Some(0)); // Select first item by default
        list
    }
    fn next(&mut self) { let i = match self.state.selected() { Some(i) => if i >= self.items.len() - 1 { 0 } else { i + 1 }, None => 0 }; self.state.select(Some(i)); }
    fn previous(&mut self) { let i = match self.state.selected() { Some(i) => if i == 0 { self.items.len() - 1 } else { i - 1 }, None => 0 }; self.state.select(Some(i)); }
}

// The main application struct that holds all state.
struct App<'a> {
    current_view: AppView,
    show_help_popup: bool,
    should_quit: bool,
    main_menu: StatefulList<MenuItem<'a>>,
}

// Represents a single item in our menus.
struct MenuItem<'a> {
    icon: &'a str,
    text: &'a str,
    help: &'a str,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            current_view: AppView::MainMenu,
            show_help_popup: false,
            should_quit: false,
            main_menu: StatefulList::with_items(vec![
                MenuItem { icon: "🚀", text: "Replicator (Recommended)", help: "The Replicator captures the 'recipe' for your system (packages, configs) into a single file. Use this to perform a clean, fresh installation on new hardware that perfectly matches your setup." },
                MenuItem { icon: "💿", text: "Cloner (Advanced)", help: "The Cloner creates a direct, 1:1 bootable ISO image of your current system's disk. This is best for creating a full backup or moving your exact OS to identical hardware." },
                MenuItem { icon: "🧰", text: "Utilities & Manual Tools", help: "A collection of essential tools for system maintenance. Includes a hardware inspector, a USB flasher, manual installation steps, and quick backup utilities." },
                MenuItem { icon: "❓", text: "Main Help", help: "Displays the main, scrollable help manual for the entire application." },
                MenuItem { icon: "❌", text: "Quit", help: "Exits the Arch System Suite application." },
            ]),
        }
    }
}

// --- Main Application Loop ---

fn main() -> io::Result<()> {
    let mut terminal = init_terminal()?;
    let mut app = App::new();
    run_app(&mut terminal, &mut app)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Global keybindings
                if key.code == KeyCode::Char('?') {
                    app.show_help_popup = !app.show_help_popup;
                    continue;
                }
                if app.show_help_popup {
                    app.show_help_popup = false;
                    continue;
                }

                // View-specific keybindings
                match app.current_view {
                    AppView::MainMenu => handle_main_menu_input(key.code, app),
                    AppView::HelpManual => {
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                            app.current_view = AppView::MainMenu;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_main_menu_input(key_code: KeyCode, app: &mut App) {
    match key_code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('k') | KeyCode::Up => app.main_menu.previous(),
        KeyCode::Char('j') | KeyCode::Down => app.main_menu.next(),
        KeyCode::Enter => {
            if let Some(selected) = app.main_menu.state.selected() {
                match selected {
                    0 => { /* TODO: Change view to Replicator. app.current_view = AppView::Replicator; */ }
                    1 => { /* TODO: Change view to Cloner. app.current_view = AppView::Cloner; */ }
                    2 => { /* TODO: Change view to Utilities. app.current_view = AppView::Utilities; */ }
                    3 => app.current_view = AppView::HelpManual,
                    4 => app.should_quit = true,
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

// --- UI Rendering ---

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

fn render_main_menu<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(8), // For ASCII Art
            Constraint::Min(0),    // For Menu
            Constraint::Length(1), // For Status
        ])
        .split(area);

    let art = Paragraph::new(ASCII_ART).alignment(Alignment::Center);
    f.render_widget(art, chunks[0]);

    let items: Vec<ListItem> = app.main_menu.items.iter()
        .map(|i| ListItem::new(format!("{} {}", i.icon, i.text)).style(Style::default().fg(Color::White)))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .highlight_style(Style::default().bg(Color::Rgb(60, 60, 90)).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[1], &mut app.main_menu.state);

    let status = Paragraph::new("v2.1.0 | Press '?' for context help").alignment(Alignment::Center);
    f.render_widget(status, chunks[2]);
}

fn render_help_manual<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = "This is the main, scrollable help page for the entire Arch System Suite.\n\nIt would contain detailed sections on the Replicator, Cloner, and all Utilities.\n\nPress 'q' to return to the main menu.";
    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help Manual"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_help_popup<B: Backend>(f: &mut Frame<B>, app: &App) {
    let help_text = match app.current_view {
        AppView::MainMenu => app.main_menu.items[app.main_menu.state.selected().unwrap_or(0)].help,
        AppView::HelpManual => "This is the main help page. Use 'q' or 'Esc' to return to the previous menu.",
    };

    let block = Block::default().title("Context Help").borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(60, 40, f.size());
    let wrapped_text = wrap(help_text, (area.width - 4) as usize);
    let paragraph = Paragraph::new(wrapped_text).block(block);

    f.render_widget(Clear, area); //this clears the background
    f.render_widget(paragraph, area);
}

// --- Terminal Setup & Helpers ---

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
