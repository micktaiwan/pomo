use crossterm::{
    cursor, execute,
    event::{self, Event, KeyCode, KeyModifiers},
    style::{Print, SetForegroundColor, ResetColor, Color},
    terminal::{self, ClearType},
};
use std::{env, io::stdout, process::Command, time::{Duration, SystemTime}};
use chrono::Local;

#[derive(Clone, Copy)]
enum DisplaySize {
    Text,
    Compact,
    Large,
}

impl DisplaySize {
    fn height(self) -> usize {
        match self {
            DisplaySize::Text => 1,
            DisplaySize::Compact => 5,
            DisplaySize::Large => 7,
        }
    }

    fn glyph(self, ch: char) -> Option<&'static [&'static str]> {
        match self {
            DisplaySize::Text => None,
            DisplaySize::Compact => match ch {
                '0'..='9' => Some(DIGITS_SM[ch as usize - '0' as usize]),
                ':' => Some(&COLON_SM),
                'd' => Some(&GLYPH_D_SM),
                ' ' => Some(&GLYPH_SPACE_SM),
                _ => None,
            },
            DisplaySize::Large => match ch {
                '0'..='9' => Some(DIGITS[ch as usize - '0' as usize]),
                ':' => Some(&COLON),
                'd' => Some(&GLYPH_D),
                ' ' => Some(&GLYPH_SPACE),
                _ => None,
            },
        }
    }
}

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
const GLYPH_D: [&str; 7] = ["  ██  ", "  ██  ", "  ██  ", "  ██  ", "  ██  ", "  ██  ", "███   "];
const GLYPH_SPACE: [&str; 7] = ["    ", "    ", "    ", "    ", "    ", "    ", "    "];

const DIGITS_SM: [&[&str]; 10] = [
    &["███", "█ █", "█ █", "█ █", "███"],
    &[" █ ", "██ ", " █ ", " █ ", "███"],
    &["███", "  █", "███", "█  ", "███"],
    &["███", "  █", "███", "  █", "███"],
    &["█ █", "█ █", "███", "  █", "  █"],
    &["███", "█  ", "███", "  █", "███"],
    &["███", "█  ", "███", "█ █", "███"],
    &["███", "  █", " █ ", "█  ", "█  "],
    &["███", "█ █", "███", "█ █", "███"],
    &["███", "█ █", "███", "  █", "███"],
];
const COLON_SM: [&str; 5] = ["   ", " █ ", "   ", " █ ", "   "];
const GLYPH_D_SM: [&str; 5] = [" █ ", " █ ", " █ ", " █ ", "█  "];
const GLYPH_SPACE_SM: [&str; 5] = ["   ", "   ", "   ", "   ", "   "];

fn center_pad(available: usize, content: usize) -> String {
    if available > content {
        " ".repeat((available - content) / 2)
    } else {
        String::new()
    }
}

fn decompose_secs(secs: u64) -> (u64, u64, u64, u64) {
    (secs / 86400, (secs % 86400) / 3600, (secs % 3600) / 60, secs % 60)
}

fn parse_duration(input: &str) -> Option<u64> {
    let input = input.trim().to_lowercase();
    let mut total: u64 = 0;
    let mut current = String::new();

    for c in input.chars() {
        match c {
            'd' => {
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
    let (d, h, m, s) = decompose_secs(secs);
    let mut parts = String::new();
    if d > 0 { parts.push_str(&format!("{d}d")); }
    if h > 0 { parts.push_str(&format!("{h}h")); }
    if m > 0 { parts.push_str(&format!("{m}m")); }
    if s > 0 { parts.push_str(&format!("{s}s")); }
    parts
}

fn format_time(secs: u64) -> String {
    let (d, h, m, s) = decompose_secs(secs);
    if d > 0 {
        format!("{d}d {h}:{m:02}:{s:02}")
    } else if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

fn render_big(time_str: &str, term_width: u16, size: DisplaySize) -> String {
    if matches!(size, DisplaySize::Text) {
        let pad = center_pad(term_width as usize, time_str.len());
        return format!("{pad}{time_str}");
    }

    let height = size.height();
    let mut lines = vec![String::new(); height];
    for ch in time_str.chars() {
        let Some(glyph) = size.glyph(ch) else { continue };
        for (i, row) in glyph.iter().enumerate() {
            lines[i].push_str(row);
            lines[i].push_str("  ");
        }
    }
    let content_width = lines[0].chars().count();
    let pad = center_pad(term_width as usize, content_width);
    lines.iter().map(|l| format!("{pad}{l}")).collect::<Vec<_>>().join("\r\n")
}

enum Mode {
    Timer { secs: u64, label: String },
    Stopwatch,
}

fn notify(msg: &str) {
    // terminal-notifier doesn't open Script Editor on click (unlike osascript)
    let ok = Command::new("terminal-notifier")
        .args(["-title", "pomo", "-message", msg, "-sound", "Glass"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if ok {
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

    // Parse options
    let mut title: Option<String> = None;
    let mut size = DisplaySize::Large;
    let mut remaining_args: Vec<String> = Vec::new();
    let mut args_iter = args.iter().skip(1).peekable();
    while let Some(arg) = args_iter.next() {
        if arg == "-s" {
            if let Some(val) = args_iter.next() {
                match val.as_str() {
                    "1" => size = DisplaySize::Text,
                    "2" => size = DisplaySize::Compact,
                    "3" => size = DisplaySize::Large,
                    _ => {
                        eprintln!("Invalid -s option: {} (valid values: 1, 2, 3)", val);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("-s requires a value (1, 2, 3)");
                std::process::exit(1);
            }
        } else if arg == "--title" || arg == "-t" {
            let title_words: Vec<String> = args_iter.by_ref().cloned().collect();
            if title_words.is_empty() {
                eprintln!("--title requires a value");
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
            let label = format!("→ {} ({})", remaining_args[0], format_duration_human(secs));
            Mode::Timer { secs, label }
        } else if let Some(secs) = parse_duration(&remaining_args[0]) {
            let label = format!("({})", format_duration_human(secs));
            Mode::Timer { secs, label }
        } else {
            eprintln!("Invalid duration: {}", remaining_args[0]);
            eprintln!("Usage: pomo [-s 1|2|3] [duration|time] [--title TEXT]  (e.g. pomo 25m -t standup)");
            std::process::exit(1);
        }
    } else {
        eprintln!("Usage: pomo [-s 1|2|3] [duration|time] [--title TEXT]  (e.g. pomo 25m -t standup)");
        std::process::exit(1);
    };

    let mut stdout = stdout();
    terminal::enable_raw_mode().expect("raw mode");
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).ok();
    let guard = RawModeGuard;

    let start = SystemTime::now();
    let start_time = Local::now();
    let mut adjust_secs: i64 = 0; // +/- adjustment in seconds
    let mut pause_start: Option<std::time::Instant> = None;
    let mut total_paused = Duration::ZERO;
    let (total_secs, timer_label) = match &mode {
        Mode::Timer { secs, label } => (*secs, label.clone()),
        Mode::Stopwatch => (0, String::new()),
    };

    loop {
        let raw_elapsed = start.elapsed().unwrap_or_default();
        let current_paused = pause_start.map_or(total_paused, |ps| total_paused + ps.elapsed());
        let elapsed_secs = raw_elapsed.saturating_sub(current_paused).as_secs();
        let (display_secs, info_line) = match mode {
            Mode::Stopwatch => (
                (elapsed_secs as i64 + adjust_secs).max(0) as u64,
                start_time.format("Started at %H:%M").to_string(),
            ),
            Mode::Timer { .. } => {
                let remaining = (total_secs as i64 + adjust_secs) - elapsed_secs as i64;
                let end_time = start_time + chrono::Duration::seconds((total_secs as i64 + adjust_secs).max(0)) + chrono::Duration::from_std(current_paused).unwrap_or_default();
                (
                    remaining.max(0) as u64,
                    format!("Started at {} — End at {} {timer_label}", start_time.format("%H:%M"), end_time.format("%H:%M")),
                )
            }
        };

        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let time_str = format_time(display_secs);
        let big = render_big(&time_str, cols, size);
        let digit_lines = size.height() as u16;
        let title_lines: u16 = if title.is_some() { 2 } else { 0 };
        let total_lines = title_lines + digit_lines + 2; // digits + blank + label
        let top = if rows > total_lines { (rows - total_lines) / 2 } else { 0 };

        let title_display = if let Some(ref t) = title {
            let pad = center_pad(cols as usize, t.len());
            format!("{pad}{t}\r\n\r\n")
        } else {
            String::new()
        };

        let paused = pause_start.is_some();
        let pause_text = if paused { " ⏸ PAUSED" } else { "" };
        let label_pad = center_pad(cols as usize, info_line.len() + pause_text.len());

        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, top),
            Print(&title_display),
            Print(&big),
            Print(format!("\r\n\r\n{label_pad}{info_line}")),
        )
        .ok();
        if paused {
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print(pause_text),
                ResetColor,
            )
            .ok();
        }

        if matches!(mode, Mode::Timer { .. }) && display_secs == 0 {
            break;
        }

        if event::poll(Duration::from_millis(100)).unwrap_or(false)
            && let Ok(Event::Key(key)) = event::read()
        {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Char(' ') | KeyCode::Char('p') => {
                    if let Some(ps) = pause_start.take() {
                        total_paused += ps.elapsed();
                    } else {
                        pause_start = Some(std::time::Instant::now());
                    }
                }
                KeyCode::Char('+') | KeyCode::Char('=') => adjust_secs += 60,
                KeyCode::Char('-') => {
                    let min_adjust = match mode {
                        Mode::Stopwatch => -(elapsed_secs as i64),
                        Mode::Timer { .. } => -(total_secs as i64),
                    };
                    adjust_secs = (adjust_secs - 60).max(min_adjust);
                }
                _ => {}
            }
        }
    }

    if matches!(mode, Mode::Timer { .. }) {
        notify("Time's up!");
    }

    drop(guard);
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
        assert_eq!(format_time(86400), "1d 0:00:00");
        assert_eq!(format_time(2 * 86400 + 2 * 3600 + 13 * 60 + 5), "2d 2:13:05");
    }
}
