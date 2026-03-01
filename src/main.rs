use crossterm::{
    cursor, execute,
    event::{self, Event, KeyCode, KeyModifiers},
    style::Print,
    terminal::{self, ClearType},
};
use std::{env, io::stdout, process::Command, time::{Duration, Instant}};
use chrono::Local;

const DIGITS: [&[&str]; 10] = [
    &[" ████ ", "██  ██", "██  ██", "██  ██", "██  ██", "██  ██", " ████ "],
    &["  ██  ", " ███  ", "  ██  ", "  ██  ", "  ██  ", "  ██  ", "██████"],
    &[" ████ ", "██  ██", "    ██", "  ██  ", " ██   ", "██    ", "██████"],
    &[" ████ ", "██  ██", "    ██", "  ███ ", "    ██", "██  ██", " ████ "],
    &["██  ██", "██  ██", "██  ██", "██████", "    ██", "    ██", "    ██"],
    &["██████", "██    ", "██    ", "█████ ", "    ██", "    ██", "█████ "],
    &[" ████ ", "██    ", "██    ", "█████ ", "██  ██", "██  ██", " ████ "],
    &["██████", "    ██", "   ██ ", "  ██  ", " ██   ", "██    ", "██    "],
    &[" ████ ", "██  ██", "██  ██", " ████ ", "██  ██", "██  ██", " ████ "],
    &[" ████ ", "██  ██", "██  ██", " █████", "    ██", "    ██", " ████ "],
];

const COLON: [&str; 7] = ["    ", " ██ ", " ██ ", "    ", " ██ ", " ██ ", "    "];
const GLYPH_J: [&str; 7] = ["  ██  ", "  ██  ", "  ██  ", "  ██  ", "  ██  ", "  ██  ", "███   "];
const GLYPH_SPACE: [&str; 7] = ["    ", "    ", "    ", "    ", "    ", "    ", "    "];

fn parse_duration(input: &str) -> Option<u64> {
    let input = input.trim().to_lowercase();
    let mut total: u64 = 0;
    let mut current = String::new();

    for c in input.chars() {
        match c {
            'j' => {
                total += current.parse::<u64>().ok()? * 86400;
                current.clear();
            }
            'h' => {
                total += current.parse::<u64>().ok()? * 3600;
                current.clear();
            }
            'm' => {
                total += current.parse::<u64>().ok()? * 60;
                current.clear();
            }
            's' => {
                total += current.parse::<u64>().ok()?;
                current.clear();
            }
            '0'..='9' => current.push(c),
            _ => return None,
        }
    }

    if !current.is_empty() {
        return None;
    }

    if total == 0 { None } else { Some(total) }
}

fn format_duration_human(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let mut parts = String::new();
    if d > 0 { parts.push_str(&format!("{d}j")); }
    if h > 0 { parts.push_str(&format!("{h}h")); }
    if m > 0 { parts.push_str(&format!("{m}m")); }
    if s > 0 { parts.push_str(&format!("{s}s")); }
    parts
}

fn format_time(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if d > 0 {
        format!("{d}j {h}:{m:02}:{s:02}")
    } else if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

fn render_big(time_str: &str, term_width: u16) -> String {
    let mut lines = vec![String::new(); 7];
    for ch in time_str.chars() {
        let glyph: &[&str] = match ch {
            '0'..='9' => DIGITS[ch as usize - '0' as usize],
            ':' => &COLON,
            'j' => &GLYPH_J,
            ' ' => &GLYPH_SPACE,
            _ => continue,
        };
        for (i, row) in glyph.iter().enumerate() {
            lines[i].push_str(row);
            lines[i].push_str("  ");
        }
    }
    let content_width = unicode_display_width(&lines[0]);
    let pad = if (term_width as usize) > content_width {
        " ".repeat((term_width as usize - content_width) / 2)
    } else {
        String::new()
    };
    lines.iter().map(|l| format!("{pad}{l}")).collect::<Vec<_>>().join("\r\n")
}

fn unicode_display_width(s: &str) -> usize {
    // █ (U+2588) is a single-width character in monospace terminals
    s.chars().count()
}

enum Mode {
    Timer(u64),
    Stopwatch,
}

fn notify(msg: &str) {
    // terminal-notifier doesn't open Script Editor on click (unlike osascript)
    let tn = Command::new("terminal-notifier")
        .args(["-title", "pomo", "-message", msg, "-sound", "Glass"])
        .output();
    if tn.is_ok() {
        return;
    }
    // Fallback: osascript (clicking the notification will open Script Editor)
    let _ = Command::new("osascript")
        .args(["-e", &format!("display notification \"{msg}\" with title \"pomo\" sound name \"Glass\"")])
        .output();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = if args.len() == 1 {
        Mode::Stopwatch
    } else if args.len() == 2 {
        match parse_duration(&args[1]) {
            Some(s) => Mode::Timer(s),
            None => {
                eprintln!("Durée invalide : {}", args[1]);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Usage: pomo [durée]  (ex: pomo, pomo 25m, pomo 90s)");
        std::process::exit(1);
    };

    let mut stdout = stdout();
    terminal::enable_raw_mode().expect("raw mode");
    execute!(stdout, terminal::EnterAlternateScreen).ok();
    let _guard = RawModeGuard;

    let start = Instant::now();
    let started_at = match mode {
        Mode::Timer(total) => {
            let label = format_duration_human(total);
            Local::now().format("Started at %H:%M").to_string() + &format!(" ({label})")
        }
        Mode::Stopwatch => Local::now().format("Started at %H:%M").to_string(),
    };

    loop {
        let elapsed_secs = start.elapsed().as_secs();
        let display_secs = match mode {
            Mode::Stopwatch => elapsed_secs,
            Mode::Timer(total) => {
                total.saturating_sub(elapsed_secs)
            }
        };

        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let time_str = format_time(display_secs);
        let big = render_big(&time_str, cols);
        let total_lines = 9; // 1 label + 1 blank + 7 digits
        let top = if rows > total_lines { (rows - total_lines) / 2 } else { 0 };
        let label_pad = if (cols as usize) > started_at.len() {
            " ".repeat((cols as usize - started_at.len()) / 2)
        } else {
            String::new()
        };

        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, top),
            Print(&big),
            Print(format!("\r\n\r\n{label_pad}{started_at}")),
            cursor::Hide,
        )
        .ok();

        if matches!(mode, Mode::Timer(_)) && display_secs == 0 {
            break;
        }

        if event::poll(Duration::from_millis(100)).unwrap_or(false)
            && let Ok(Event::Key(key)) = event::read()
            && (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
                || key.code == KeyCode::Esc
                || key.code == KeyCode::Char('q'))
        {
            break;
        }
    }

    if matches!(mode, Mode::Timer(_)) {
        notify("Temps écoulé !");
    }

    drop(_guard);
    println!("\n\n");
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), terminal::LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("25m"), Some(1500));
        assert_eq!(parse_duration("90s"), Some(90));
        assert_eq!(parse_duration("1h30m"), Some(5400));
        assert_eq!(parse_duration("1h"), Some(3600));
        assert_eq!(parse_duration("abc"), None);
        assert_eq!(parse_duration("0m"), None);
        assert_eq!(parse_duration("25"), None);
    }

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(90), "01:30");
        assert_eq!(format_time(3661), "1:01:01");
        assert_eq!(format_time(0), "00:00");
        assert_eq!(format_time(86400), "1j 0:00:00");
        assert_eq!(format_time(2 * 86400 + 2 * 3600 + 13 * 60 + 5), "2j 2:13:05");
    }
}
