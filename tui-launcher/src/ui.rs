// ===================================================================
// UI Rendering Module
// ===================================================================
// This module is responsible for drawing all widgets to the screen.
// It is stateless and only reads from the `App` struct.

use crate::app::{App, AppView, InputMode, MenuItem, StatefulList};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use textwrap::wrap;

/// The main UI drawing function. It delegates to other functions based on the current view.
pub fn ui(f: &mut Frame, app: &mut App) {
    // The main layout is a single full-screen area.
    let main_layout = Layout::default().constraints([Constraint::Percentage(100)]).split(f.size());

    // Render the primary view based on the application's current state.
    // Note: ActionPopup is rendered as an overlay, so the view behind it is still drawn.
    match app.current_view {
        AppView::MainMenu | AppView::ActionPopup => render_main_menu(f, app, main_layout[0]),
        AppView::HelpManual => render_help_manual(f, main_layout[0]),
        AppView::Replicator => render_submenu(f, &mut app.replicator_menu, "Replicator Menu", main_layout[0]),
        AppView::Cloner => render_submenu(f, &mut app.cloner_menu, "Cloner Menu", main_layout[0]),
        AppView::Utilities => render_submenu(f, &mut app.utilities_menu, "Utilities Menu", main_layout[0]),
    }

    // Render popups over the main view if they are active.
    if app.show_help_popup {
        render_help_popup(f, app);
    }
    if app.current_view == AppView::ActionPopup {
        render_action_popup(f, app);
    }
    if app.input_mode == InputMode::Editing {
        render_input_popup(f, app);
    }
}

/// Renders the Main Menu screen, including the ASCII art and status line.
fn render_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default().direction(Direction::Vertical).margin(2)
        .constraints([Constraint::Length(8), Constraint::Min(0), Constraint::Length(1)]).split(area);
    
    let art = Paragraph::new(ASCII_ART).style(Style::default().fg(Color::Rgb(110, 125, 224))).alignment(Alignment::Center);
    f.render_widget(art, chunks[0]);

    render_menu_list(f, &mut app.main_menu, "Main Menu", chunks[1]);

    let status = Paragraph::new("v4.0.0 | Press 'j'/'k' to navigate | 'Enter' to select | '?' for help | 'q' to quit").alignment(Alignment::Center);
    f.render_widget(status, chunks[2]);
}

/// A generic function to render any of our sub-menus.
fn render_submenu(f: &mut Frame, list: &mut StatefulList<MenuItem>, title: &str, area: Rect) {
    let chunks = Layout::default().direction(Direction::Vertical).margin(2)
        .constraints([Constraint::Min(0), Constraint::Length(1)]).split(area);
    
    render_menu_list(f, list, title, chunks[0]);

    let status = Paragraph::new("Press 'j'/'k' to navigate | 'Enter' to select | '?' for help | 'Esc' to go back").alignment(Alignment::Center);
    f.render_widget(status, chunks[1]);
}

/// A generic helper function to render a `StatefulList` of `MenuItems`.
fn render_menu_list(f: &mut Frame, list: &mut StatefulList<MenuItem>, title: &str, area: Rect) {
    let items: Vec<ListItem> = list.items.iter()
        .map(|i| ListItem::new(format!("{} {}", i.icon, i.text)).style(Style::default().fg(Color::White)))
        .collect();

    let list_widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::Rgb(60, 60, 90)).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list_widget, area, &mut list.state);
}

/// Renders the full-screen help manual.
fn render_help_manual(f: &mut Frame, area: Rect) {
    let help_text = "This is the main help page for Arch System Suite v4.0.0.\n\nIt would contain detailed sections on the Replicator, Cloner, and all Utilities, explaining each feature in depth.\n\nPress 'q' or 'Esc' to return to the main menu.";
    let paragraph = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help Manual")).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Renders the context-aware help pop-up window over the current view.
fn render_help_popup(f: &mut Frame, app: &App<'_>) {
    let help_text = match app.current_view {
        AppView::MainMenu | AppView::ActionPopup => app.main_menu.items[app.main_menu.state.selected().unwrap_or(0)].help,
        AppView::Replicator => app.replicator_menu.items[app.replicator_menu.state.selected().unwrap_or(0)].help,
        AppView::Cloner => app.cloner_menu.items[app.cloner_menu.state.selected().unwrap_or(0)].help,
        AppView::Utilities => app.utilities_menu.items[app.utilities_menu.state.selected().unwrap_or(0)].help,
        AppView::HelpManual => "This is the main help page. Use 'q' or 'Esc' to return to the previous menu.",
    };
    let block = Block::default().title("Context Help").borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(60, 40, f.size());
    let wrapped_text: Vec<Line> = wrap(help_text, (area.width - 4) as usize).iter().map(|s| Line::from(s.to_string())).collect();
    let paragraph = Paragraph::new(wrapped_text).block(block);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

/// Renders a generic popup to show the result of an action.
fn render_action_popup(f: &mut Frame, app: &App<'_>) {
    let block = Block::default().title("Action Result").borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(80, 50, f.size());
    let wrapped_text: Vec<Line> = wrap(&app.popup_text, (area.width - 4) as usize).iter().map(|s| Line::from(s.to_string())).collect();
    let paragraph = Paragraph::new(wrapped_text).block(block);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

/// Renders a popup with a text input field.
fn render_input_popup(f: &mut Frame, app: &App<'_>) {
    let block = Block::default().title(app.popup_title.as_str()).borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(60, 20, f.size());
    
    let input = Paragraph::new(app.input.value())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default());
    
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    
    // Create a smaller area inside the popup for the input widget
    let input_area = Layout::default().margin(1).constraints([Constraint::Min(0)]).split(area)[0];
    f.render_widget(input, input_area);

    // Set the terminal cursor position to be inside the input field
    f.set_cursor(
        input_area.x + app.input.visual_cursor() as u16,
        input_area.y,
    );
}

/// Helper function to create a centered rectangle for pop-up windows.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)]).split(r);
    Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)]).split(popup_layout[1])[1]
}

const ASCII_ART: &str = r"
    █████╗  ██████╗  ██████╗██╗  ██╗     ███████╗██╗   ██╗██╗████████╗███████╗
   ██╔══██╗██╔════╝ ██╔════╝██║  ██║     ██╔════╝██║   ██║██║╚══██╔══╝██╔════╝
   ███████║██║  ███╗██║     ███████║     ███████╗██║   ██║██║   ██║   ███████╗
   ██╔══██║██║   ██║██║     ██╔══██║     ╚════██║██║   ██║██║   ██║   ╚════██║
   ██║  ██║╚██████╔╝╚██████╗██║  ██║     ███████║╚██████╔╝██║   ██║   ███████║
   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝     ╚══════╝ ╚═════╝ ╚═╝   ╚═╝   ╚══════╝
";
