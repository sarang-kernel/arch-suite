// ===================================================================
// UI Rendering Module
// ===================================================================
use crate::app::{App, AppView, MenuItem, Popup, StatefulList};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use textwrap::wrap;

pub fn ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default().constraints([Constraint::Percentage(100)]).split(f.size());

    // The main view is always rendered. Popups are drawn on top.
    match app.current_view {
        AppView::MainMenu => render_menu(f, &mut app.main_menu, "Main Menu", main_layout[0], true),
        AppView::Replicator => render_menu(f, &mut app.replicator_menu, "Replicator Menu", main_layout[0], false),
        AppView::Cloner => render_menu(f, &mut app.cloner_menu, "Cloner Menu", main_layout[0], false),
        AppView::Utilities => render_menu(f, &mut app.utilities_menu, "Utilities Menu", main_layout[0], false),
        AppView::ManualInstaller => render_menu(f, &mut app.manual_install_menu, "Manual Installer", main_layout[0], false),
        AppView::HelpManual => render_help_manual(f, main_layout[0]),
    }

    match app.active_popup {
        Popup::Help => render_help_popup(f, app),
        Popup::Action => render_action_popup(f, app),
        Popup::Confirm => render_confirm_popup(f, app),
        Popup::Input => render_input_popup(f, app),
        Popup::Select => render_select_popup(f, app),
        Popup::None => {}
    }
}

fn render_menu(f: &mut Frame, list: &mut StatefulList<MenuItem>, title: &str, area: Rect, show_art: bool) {
    let chunks = if show_art {
        Layout::default().direction(Direction::Vertical).margin(2)
            .constraints([Constraint::Length(8), Constraint::Min(0), Constraint::Length(1)]).split(area)
    } else {
        Layout::default().direction(Direction::Vertical).margin(2)
            .constraints([Constraint::Min(0), Constraint::Length(1)]).split(area)
    };
    if show_art {
        let art = Paragraph::new(ASCII_ART).style(Style::default().fg(Color::Rgb(110, 125, 224))).alignment(Alignment::Center);
        f.render_widget(art, chunks[0]);
    }
    let list_chunk = if show_art { chunks[1] } else { chunks[0] };
    let status_chunk = if show_art { chunks[2] } else { chunks[1] };
    let items: Vec<ListItem> = list.items.iter().map(|i| ListItem::new(format!("{} {}", i.icon, i.text)).style(Style::default().fg(Color::White))).collect();
    let list_widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::Rgb(60, 60, 90)).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list_widget, list_chunk, &mut list.state);
    let status_text = if show_art { "v4.0.0 | 'j'/'k' to navigate | 'Enter' to select | '?' for help | 'q' to quit" } else { "'j'/'k' to navigate | 'Enter' to select | '?' for help | 'Esc' to go back" };
    let status = Paragraph::new(status_text).alignment(Alignment::Center);
    f.render_widget(status, status_chunk);
}

fn render_help_manual(f: &mut Frame, area: Rect) {
    let help_text = "This is the main help page for Arch System Suite v4.0.0.\n\nIt contains detailed sections on the Replicator, Cloner, and all Utilities, explaining each feature in depth.\n\nPress 'q' or 'Esc' to return to the main menu.";
    let paragraph = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help Manual")).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_help_popup(f: &mut Frame, app: &App<'_>) {
    let help_text = match app.current_view {
        AppView::MainMenu => app.main_menu.selected_item().map_or("", |i| i.help),
        AppView::Replicator => app.replicator_menu.selected_item().map_or("", |i| i.help),
        AppView::Cloner => app.cloner_menu.selected_item().map_or("", |i| i.help),
        AppView::Utilities => app.utilities_menu.selected_item().map_or("", |i| i.help),
        AppView::ManualInstaller => app.manual_install_menu.selected_item().map_or("", |i| i.help),
        AppView::HelpManual => "This is the main help page. Use 'q' or 'Esc' to return to the previous menu.",
        // FIX: ActionPopup is a popup, not a view with its own help. We show help for the view behind it.
        AppView::ActionPopup => "", // Should not happen as help is disabled during action popups.
    };
    render_popup(f, "Context Help", help_text, 60, 40);
}

fn render_action_popup(f: &mut Frame, app: &App<'_>) { render_popup(f, &app.popup_title, &app.popup_text, 80, 50); }
fn render_confirm_popup(f: &mut Frame, app: &App<'_>) { let text = format!("{}\n\n[Y] Yes / [N] No", app.popup_text); render_popup(f, &app.popup_title, &text, 60, 25); }

fn render_input_popup(f: &mut Frame, app: &App<'_>) {
    let block = Block::default().title(app.popup_title.as_str()).borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(60, 20, f.size());
    let input = Paragraph::new(app.popup_input.value()).block(Block::default());
    f.set_cursor(area.x + app.popup_input.visual_cursor() as u16 + 1, area.y + 1);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let input_area = Layout::default().margin(1).constraints([Constraint::Min(0)]).split(area)[0];
    f.render_widget(input, input_area);
}

fn render_select_popup(f: &mut Frame, app: &mut App<'_>) {
    let block = Block::default().title(app.popup_title.as_str()).borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(80, 70, f.size());
    let items: Vec<ListItem> = app.popup_list.items.iter().map(|i| ListItem::new(i.clone())).collect();
    let list = List::new(items).highlight_style(Style::default().bg(Color::Rgb(60, 60, 90)).add_modifier(Modifier::BOLD)).highlight_symbol(">> ");
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let list_area = Layout::default().margin(1).constraints([Constraint::Min(0)]).split(area)[0];
    f.render_stateful_widget(list, list_area, &mut app.popup_list.state);
}

fn render_popup(f: &mut Frame, title: &str, text: &str, width_percent: u16, height_percent: u16) {
    let block = Block::default().title(title).borders(Borders::ALL).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    let area = centered_rect(width_percent, height_percent, f.size());
    let wrapped_text: Vec<Line> = wrap(text, (area.width - 4) as usize).iter().map(|s| Line::from(s.to_string())).collect();
    let paragraph = Paragraph::new(wrapped_text).block(block);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)]).split(r);
    Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)]).split(popup_layout[1])[1]
}

const ASCII_ART: &str = r"
    █████╗  ██████╗  ██████╗██╗  ██╗     ███████╗██╗   ██╗██╗████████╗███████╗
   ██╔══██╗██╔════╝ ██╔════╝██║  ██║     ██╔════╝██║   ██║██║╚══██╔══╝██╔════╝
   ███████║██║  ███╗██║     ███████║     ███████╗██║   ██║██║   ██║   ███████╗
   ██╔══██║██║   ██║██║     ██╔══██║     ╚════██║██║   ██║██║   ██║   ╚════██║
   ██║  ██║╚██████╔╝╚██████╗██║  ██║     ███████║╚██████╔╝██║   ██║   ███████║
   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝     ╚══════╝ ╚═════╝ ╚═╝   ╚═╝   ╚══════╝
";
