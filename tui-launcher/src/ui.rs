// ===================================================================
// UI Rendering Module
// ===================================================================
// This module is responsible for drawing all widgets to the screen.
// It is stateless and only reads from the `App` struct.

use crate::app::{App, AppView};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use textwrap::wrap;

/// The main UI drawing function. It delegates to other functions based on the current view.
pub fn ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default().constraints([Constraint::Percentage(100)]).split(f.size());

    // Always render the main view in the background
    match app.current_view {
        AppView::MainMenu | AppView::ActionPopup => render_main_menu(f, app, main_layout[0]),
        AppView::HelpManual => render_help_manual(f, main_layout[0]),
    }

    // Render popups over the main view if they are active
    if app.show_help_popup {
        render_help_popup(f, app);
    }
    if app.current_view == AppView::ActionPopup {
        render_action_popup(f, app);
    }
}

/// Renders the Main Menu screen.
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

    let status = Paragraph::new("v3.2.0 | Press 'j'/'k' to navigate | 'Enter' to select | '?' for help | 'q' to quit").alignment(Alignment::Center);
    f.render_widget(status, chunks[2]);
}

/// Renders the full-screen help manual.
fn render_help_manual(f: &mut Frame, area: Rect) {
    let help_text = "This is the main help page for Arch System Suite v3.2.0.\n\nIt would contain detailed sections on the Replicator, Cloner, and all Utilities, explaining each feature in depth.\n\nPress 'q' or 'Esc' to return to the main menu.";
    let paragraph = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help Manual")).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Renders the context-aware help pop-up window over the current view.
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

/// Renders a generic popup to show the result of an action.
fn render_action_popup(f: &mut Frame, app: &App<'_>) {
    let block = Block::default().title("Action Result").borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(80, 50, f.size());
    let wrapped_text: Vec<Line> = wrap(&app.popup_text, (area.width - 4) as usize).iter().map(|s| Line::from(s.to_string())).collect();
    let paragraph = Paragraph::new(wrapped_text).block(block);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
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
