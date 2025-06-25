use std::io::{stdout, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use crossterm::{
    cursor, event::{self, Event, KeyCode}, execute, queue, style::{self, Color, Print, Stylize}, terminal
};

// --- Constants ---
const APP_VERSION: &str = "1.5.0";
const SCRIPT_NAME: &str = "engine.sh";

// --- Structs ---
struct MenuItem<'a> { icon: &'a str, text: &'a str, key: char, arg: &'a str }
struct Status { is_live_env: bool, has_internet: bool, has_sudo: bool }

// --- Main Application Logic ---
fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    run_app(&mut stdout)?;
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn run_app(stdout: &mut impl Write) -> std::io::Result<()> {
    let status = check_status();
    loop {
        draw_ui(stdout, &status, None)?;
        if let Event::Key(key_event) = event::read()? {
            let selected_key = match key_event.code { KeyCode::Char(c) => Some(c), KeyCode::Esc => Some('q'), _ => None };
            if let Some(key) = selected_key {
                let menu_items = get_menu_items();
                if let Some(item) = menu_items.iter().find(|i| i.key == key) {
                    if item.key == 'q' { break; }
                    draw_ui(stdout, &status, Some(item.key))?;
                    thread::sleep(Duration::from_millis(100));
                    call_script(item.arg)?;
                }
            }
        }
    }
    Ok(())
}

// --- UI Drawing ---
// (draw_ui, draw_art, draw_menu, and draw_status_line are unchanged from the previous version)
fn draw_ui(stdout: &mut impl Write, status: &Status, selected_key: Option<char>) -> std::io::Result<()> {
    let menu_items = get_menu_items();
    let (width, height) = terminal::size()?;
    let art_height = ASCII_ART.lines().count() as u16;
    let menu_height = menu_items.len() as u16;
    let total_height = art_height + menu_height + 4;
    let start_y = (height.saturating_sub(total_height)) / 2;
    let start_x = (width.saturating_sub(70)) / 2;
    queue!(stdout, style::SetBackgroundColor(Color::Rgb { r: 28, g: 28, b: 46 }), terminal::Clear(terminal::ClearType::All))?;
    draw_art(stdout, start_x, start_y)?;
    draw_menu(stdout, &menu_items, start_x, start_y + art_height + 2, selected_key)?;
    draw_status_line(stdout, status, height.saturating_sub(1))?;
    stdout.flush()?;
    Ok(())
}
fn draw_art(stdout: &mut impl Write, x: u16, y: u16) -> std::io::Result<()> { for (i, line) in ASCII_ART.lines().enumerate() { queue!(stdout, cursor::MoveTo(x, y + i as u16), style::SetForegroundColor(Color::Rgb { r: 110, g: 125, b: 224 }), Print(line))?; } Ok(()) }
fn draw_menu(stdout: &mut impl Write, items: &[MenuItem], x: u16, y: u16, selected_key: Option<char>) -> std::io::Result<()> { for (i, item) in items.iter().enumerate() { let item_y = y + i as u16; let is_selected = selected_key.map_or(false, |k| k == item.key); let text_color = if is_selected { Color::Rgb { r: 242, g: 153, b: 89 } } else { Color::Rgb { r: 138, g: 204, b: 209 } }; let key_color = if is_selected { Color::Rgb { r: 138, g: 204, b: 209 } } else { Color::Rgb { r: 242, g: 153, b: 89 } }; queue!(stdout, cursor::MoveTo(x + 4, item_y), style::SetForegroundColor(text_color), Print(format!("{} {}", item.icon, item.text)), cursor::MoveTo(x + 40, item_y), style::SetForegroundColor(key_color), Print(item.key))?; } Ok(()) }
fn draw_status_line(stdout: &mut impl Write, status: &Status, y: u16) -> std::io::Result<()> { let env_text = if status.is_live_env { "🔵 Live ISO" } else { "🟢 Installed System" }; let net_text = if status.has_internet { "✓ Net" } else { "✗ Net" }; let sudo_text = if status.has_sudo { "✓ Sudo" } else { "✗ Sudo" }; let status_string = format!(" {}  |  {}  |  {}  |  {} ", APP_VERSION.bold(), env_text, net_text, sudo_text); queue!(stdout, cursor::MoveTo(1, y), style::SetBackgroundColor(Color::Rgb { r: 60, g: 60, b: 90 }), style::SetForegroundColor(Color::White), Print(status_string))?; Ok(()) }

// --- System Interaction & Helpers ---

// NEW: This function intelligently finds the engine script path.
fn get_engine_script_path() -> String {
    // option_env! is a compile-time macro that returns an Option.
    // It doesn't fail if the env var is not set.
    option_env!("ENGINE_SCRIPT_PATH")
        .map(|s| s.to_string()) // If set (production build via PKGBUILD), use it.
        .unwrap_or_else(|| {
            // If not set (local development), construct a relative path.
            // This assumes the executable is in `.../tui-launcher/target/release/`
            // and the script is in `.../`
            let mut path = std::env::current_exe().expect("Failed to get executable path");
            path.pop(); // to target/release or target/debug
            path.pop(); // to target
            path.pop(); // to project root (tui-launcher)
            path.push(SCRIPT_NAME);
            path.to_str().expect("Failed to construct path").to_string()
        })
}

fn call_script(arg: &str) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    // Use the new helper function to get the correct path
    let script_path = get_engine_script_path();
    Command::new(script_path).arg(arg).status()?;

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    Ok(())
}

fn check_status() -> Status {
    Status {
        is_live_env: std::path::Path::new("/etc/archiso_version").exists(),
        has_internet: Command::new("ping").args(["-c", "1", "archlinux.org"]).stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success()),
        has_sudo: Command::new("sudo").arg("-n").arg("true").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success()),
    }
}

fn get_menu_items() -> Vec<MenuItem<'static>> {
    vec![
        MenuItem { icon: "🚀", text: "Replicator (Recommended)", key: 'r', arg: "--replicator" },
        MenuItem { icon: "💿", text: "Cloner (Advanced)", key: 'c', arg: "--cloner" },
        MenuItem { icon: "🧰", text: "Utilities & Manual Tools", key: 'u', arg: "--utilities" },
        MenuItem { icon: "❓", text: "Help", key: 'h', arg: "--help" },
        MenuItem { icon: "❌", text: "Quit", key: 'q', arg: "--quit" },
    ]
}

const ASCII_ART: &str = r#"
    █████╗  ██████╗  ██████╗██╗  ██╗     ███████╗██╗   ██╗██╗████████╗███████╗
   ██╔══██╗██╔════╝ ██╔════╝██║  ██║     ██╔════╝██║   ██║██║╚══██╔══╝██╔════╝
   ███████║██║  ███╗██║     ███████║     ███████╗██║   ██║██║   ██║   ███████╗
   ██╔══██║██║   ██║██║     ██╔══██║     ╚════██║██║   ██║██║   ██║   ╚════██║
   ██║  ██║╚██████╔╝╚██████╗██║  ██║     ███████║╚██████╔╝██║   ██║   ███████║
   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝     ╚══════╝ ╚═════╝ ╚═╝   ╚═╝   ╚══════╝
"#;
