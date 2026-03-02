use crossterm::{
    cursor, execute,
    event::{self, Event, KeyCode, KeyModifiers},
    style::Print,
    terminal::{self, ClearType},
};
use std::{env, io::stdout, process::Command, time::{Duration, SystemTime}};
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
    Timer { secs: u64, label: String },
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

fn parse_target_time(input: &str) -> Option<u64> {
    let parts: Vec<&str> = input.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let hour: u32 = parts[0].parse().ok()?;
    let min: u32 = parts[1].parse().ok()?;
    if hour >= 24 || min >= 60 {
        return None;
    }
    let now = Local::now();
    let today = now.date_naive();
    let target_naive = today.and_hms_opt(hour, min, 0)?;
    let target = target_naive.and_local_timezone(now.timezone()).single()?;
    let target = if target <= now {
        // Target time already passed today, schedule for tomorrow
        target + chrono::Duration::days(1)
    } else {
        target
    };
    let diff = (target - now).num_seconds();
    if diff <= 0 { None } else { Some(diff as u64) }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse --title option: consumes all words until a valid duration/time is found
    let mut title: Option<String> = None;
    let mut remaining_args: Vec<String> = Vec::new();
    let mut args_iter = args.iter().skip(1).peekable();
    while let Some(arg) = args_iter.next() {
        if arg == "--title" {
            let mut title_words: Vec<String> = Vec::new();
            while let Some(next) = args_iter.peek() {
                if parse_duration(next).is_some() || parse_target_time(next).is_some() {
                    break;
                }
                title_words.push(args_iter.next().unwrap().clone());
            }
            if title_words.is_empty() {
                eprintln!("--title nécessite une valeur");
                std::process::exit(1);
            }
            title = Some(title_words.join(" "));
        } else {
            remaining_args.push(arg.clone());
        }
    }

    let mode = if remaining_args.is_empty() {
        Mode::Stopwatch
    } else if remaining_args.len() == 1 {
        if let Some(secs) = parse_target_time(&remaining_args[0]) {
            let label = format!("→ {}", remaining_args[0]);
            Mode::Timer { secs, label }
        } else if let Some(secs) = parse_duration(&remaining_args[0]) {
            let label = format!("({})", format_duration_human(secs));
            Mode::Timer { secs, label }
        } else {
            eprintln!("Durée invalide : {}", remaining_args[0]);
            eprintln!("Usage: pomo [--title TEXT] [durée|heure]  (ex: pomo, pomo 25m, pomo --title Focus 25m)");
            std::process::exit(1);
        }
    } else {
        eprintln!("Usage: pomo [--title TEXT] [durée|heure]  (ex: pomo, pomo 25m, pomo --title Focus 25m)");
        std::process::exit(1);
    };

    let mut stdout = stdout();
    terminal::enable_raw_mode().expect("raw mode");
    execute!(stdout, terminal::EnterAlternateScreen).ok();
    let _guard = RawModeGuard;

    let start = SystemTime::now();
    let start_time = Local::now();
    let (total_secs, started_at) = match &mode {
        Mode::Timer { secs, label } => {
            (*secs, format!("Started at {} {label}", start_time.format("%H:%M")))
        }
        Mode::Stopwatch => (0, start_time.format("Started at %H:%M").to_string()),
    };

    loop {
        let elapsed_secs = start.elapsed().unwrap_or_default().as_secs();
        let display_secs = match mode {
            Mode::Stopwatch => elapsed_secs,
            Mode::Timer { .. } => total_secs.saturating_sub(elapsed_secs),
        };

        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let time_str = format_time(display_secs);
        let big = render_big(&time_str, cols);
        let title_lines: u16 = if title.is_some() { 2 } else { 0 }; // title + blank line
        let total_lines = title_lines + 9; // (title?) + 7 digits + blank + label
        let top = if rows > total_lines { (rows - total_lines) / 2 } else { 0 };

        let title_display = if let Some(ref t) = title {
            let pad = if (cols as usize) > t.len() {
                " ".repeat((cols as usize - t.len()) / 2)
            } else {
                String::new()
            };
            format!("{pad}{t}\r\n\r\n")
        } else {
            String::new()
        };

        let label_pad = if (cols as usize) > started_at.len() {
            " ".repeat((cols as usize - started_at.len()) / 2)
        } else {
            String::new()
        };

        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, top),
            Print(&title_display),
            Print(&big),
            Print(format!("\r\n\r\n{label_pad}{started_at}")),
            cursor::Hide,
        )
        .ok();

        if matches!(mode, Mode::Timer { .. }) && display_secs == 0 {
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

    if matches!(mode, Mode::Timer { .. }) {
        notify("Temps écoulé !");
    }

    drop(_guard);
    let end_time = Local::now();
    let elapsed = format_duration_human(start.elapsed().unwrap_or_default().as_secs());
    println!();
    println!("  Started:  {}", start_time.format("%Y-%m-%d %H:%M:%S"));
    println!("  Duration: {}", elapsed);
    println!("  Ended:    {}", end_time.format("%Y-%m-%d %H:%M:%S"));
    println!();
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
    fn test_parse_target_time() {
        // Valid times return Some (exact value depends on current time, just check Some/None)
        assert!(parse_target_time("23:59").is_some());
        assert!(parse_target_time("00:00").is_some());
        assert!(parse_target_time("9:05").is_some());

        // Invalid formats
        assert_eq!(parse_target_time("25:00"), None);
        assert_eq!(parse_target_time("12:60"), None);
        assert_eq!(parse_target_time("abc"), None);
        assert_eq!(parse_target_time("12"), None);
        assert_eq!(parse_target_time("12:00:00"), None);
        assert_eq!(parse_target_time(""), None);
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
