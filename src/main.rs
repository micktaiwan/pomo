use crossterm::{
    cursor, execute,
    event::{self, Event, KeyCode, KeyModifiers},
    style::{Print, SetForegroundColor, ResetColor, Color},
    terminal::{self, ClearType},
};
use std::{env, fs, io::stdout, path::{Path, PathBuf}, process::Command, time::{Duration, SystemTime}};
use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate, NaiveDateTime, Timelike};

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

/// Width in columns the given time string would occupy at the given size,
/// matching render_big's layout (each glyph followed by two spaces).
fn rendered_width(time_str: &str, size: DisplaySize) -> usize {
    if matches!(size, DisplaySize::Text) {
        return time_str.chars().count();
    }
    time_str
        .chars()
        .filter_map(|ch| size.glyph(ch))
        .map(|g| g[0].chars().count() + 2)
        .sum()
}

/// Pick the largest size, starting from `desired` and stepping down, whose
/// rendered width fits within the terminal. Falls back to Text if nothing fits.
fn fit_size(time_str: &str, term_width: u16, desired: DisplaySize) -> DisplaySize {
    let ladder = [DisplaySize::Large, DisplaySize::Compact, DisplaySize::Text];
    let start = match desired {
        DisplaySize::Large => 0,
        DisplaySize::Compact => 1,
        DisplaySize::Text => 2,
    };
    for &s in &ladder[start..] {
        if rendered_width(time_str, s) <= term_width as usize {
            return s;
        }
    }
    DisplaySize::Text
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

/// OSC 0 sequence setting the terminal window/tab/pane title.
fn osc_title(s: &str) -> String {
    format!("\x1b]0;{s}\x07")
}

fn notify(msg: &str) {
    blocking_dialog("pomo", msg);
}

/// Ring the alert sound and show a modal dialog that stays frontmost until the
/// user clicks OK. Blocks the calling process until dismissed.
fn blocking_dialog(title: &str, msg: &str) {
    show_dialog(title, msg, &["OK"], "OK");
}

/// Ring the alert sound and show a modal dialog with the given buttons, staying
/// frontmost until dismissed. Blocks the calling process and returns the label
/// of the clicked button (empty string if osascript failed to run).
fn show_dialog(title: &str, msg: &str, buttons: &[&str], default: &str) -> String {
    // Play the alert sound (non-blocking) so it rings while the dialog is up.
    let _ = Command::new("afplay")
        .arg("/System/Library/Sounds/Glass.aiff")
        .spawn();
    // AppleScript string literals: escape backslashes and double quotes, then
    // splice newlines back in as `" & return & "` — a raw linefeed inside an
    // AppleScript string literal is a syntax error, so an unescaped multi-line
    // message would make osascript fail and the dialog silently never appear.
    let esc = |s: &str| {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace("\r\n", "\" & return & \"")
            .replace('\n', "\" & return & \"")
            .replace('\r', "\" & return & \"")
    };
    let button_list = buttons
        .iter()
        .map(|b| format!("\"{}\"", esc(b)))
        .collect::<Vec<_>>()
        .join(", ");
    let script = format!(
        "display dialog \"{}\" with title \"{}\" buttons {{{}}} default button \"{}\" with icon note",
        esc(msg),
        esc(title),
        button_list,
        esc(default),
    );
    let out = Command::new("osascript").args(["-e", &script]).output();
    // osascript prints `button returned:<label>` on stdout for the clicked button.
    out.map(|o| {
        String::from_utf8_lossy(&o.stdout)
            .split("button returned:")
            .nth(1)
            .and_then(|s| s.lines().next())
            .unwrap_or("")
            .trim()
            .to_string()
    })
    .unwrap_or_default()
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

// ===== Scheduled reminders (launchd-backed, survive reboot) =====

const LABEL_PREFIX: &str = "com.mick.pomo.";
/// Name suffix of the one-shot created when a reminder is snoozed.
const SNOOZE_SUFFIX: &str = "-snooze";
/// The suffix a snooze switches to when it is itself snoozed. Re-snoozing must
/// land on a *different* label than the job doing the snoozing: `load_agent`
/// boots the label out before bootstrapping it, and booting out our own label
/// kills this process mid-way, leaving a plist that launchd never armed.
const SNOOZE_SUFFIX_ALT: &str = "-snooze-2";

/// A recurring schedule for a reminder.
enum Schedule {
    /// Fire every N seconds (launchd StartInterval; launchd throttles to ~10s min).
    Interval(u64),
    /// Fire every day at HH:MM.
    Daily { hour: u32, minute: u32 },
    /// Fire on the given weekdays (launchd numbering: 0=Sun..6=Sat) at HH:MM.
    Weekly { days: Vec<u32>, hour: u32, minute: u32 },
    /// Fire once at a given date and time, then remove itself.
    Once(NaiveDateTime),
}

impl Schedule {
    /// The launchd scheduling key(s), indented to sit inside the top-level <dict>.
    fn xml(&self) -> String {
        match self {
            Schedule::Interval(secs) => {
                format!("    <key>StartInterval</key>\n    <integer>{secs}</integer>")
            }
            Schedule::Daily { hour, minute } => format!(
                "    <key>StartCalendarInterval</key>\n    <dict>\n        <key>Hour</key><integer>{hour}</integer>\n        <key>Minute</key><integer>{minute}</integer>\n    </dict>"
            ),
            Schedule::Weekly { days, hour, minute } => {
                let mut items = String::new();
                for d in days {
                    items.push_str(&format!(
                        "        <dict><key>Weekday</key><integer>{d}</integer><key>Hour</key><integer>{hour}</integer><key>Minute</key><integer>{minute}</integer></dict>\n"
                    ));
                }
                format!("    <key>StartCalendarInterval</key>\n    <array>\n{items}    </array>")
            }
            // Month + Day pins the single date; the job deletes itself once
            // fired, so it never comes back a year later.
            Schedule::Once(at) => format!(
                "    <key>StartCalendarInterval</key>\n    <dict>\n        <key>Month</key><integer>{}</integer>\n        <key>Day</key><integer>{}</integer>\n        <key>Hour</key><integer>{}</integer>\n        <key>Minute</key><integer>{}</integer>\n    </dict>",
                at.month(),
                at.day(),
                at.hour(),
                at.minute()
            ),
        }
    }

    /// Human-readable one-liner for confirmations and `remind list`.
    fn human(&self) -> String {
        match self {
            Schedule::Interval(secs) => format!("every {}", format_duration_human(*secs)),
            Schedule::Daily { hour, minute } => format!("daily at {hour:02}:{minute:02}"),
            Schedule::Weekly { days, hour, minute } => {
                let names: Vec<&str> = days.iter().map(|d| weekday_name(*d)).collect();
                format!("weekly on {} at {hour:02}:{minute:02}", names.join(","))
            }
            Schedule::Once(at) => format!("once at {}", at.format("%Y-%m-%d %H:%M")),
        }
    }
}

fn weekday_name(d: u32) -> &'static str {
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
        .get(d as usize)
        .copied()
        .unwrap_or("?")
}

fn parse_weekday(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "sun" | "sunday" | "dim" | "dimanche" => Some(0),
        "mon" | "monday" | "lun" | "lundi" => Some(1),
        "tue" | "tuesday" | "mar" | "mardi" => Some(2),
        "wed" | "wednesday" | "mer" | "mercredi" => Some(3),
        "thu" | "thursday" | "jeu" | "jeudi" => Some(4),
        "fri" | "friday" | "ven" | "vendredi" => Some(5),
        "sat" | "saturday" | "sam" | "samedi" => Some(6),
        _ => None,
    }
}

fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let (h, m) = s.split_once(':')?;
    let h: u32 = h.parse().ok()?;
    let m: u32 = m.parse().ok()?;
    (h < 24 && m < 60).then_some((h, m))
}

/// Parse an --until value: a bare date (that day at 00:00) or a datetime.
/// The reminder self-removes once now >= this instant.
fn parse_until(s: &str) -> Option<NaiveDateTime> {
    let norm = s.replace('T', " ");
    if let Ok(dt) = NaiveDateTime::parse_from_str(&norm, "%Y-%m-%d %H:%M") {
        return Some(dt);
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok().and_then(|d| d.and_hms_opt(0, 0, 0))
}

/// Parse an --at value for a one-shot reminder. Accepts an explicit datetime
/// (`2026-08-12 09:00`, `T` separator allowed), a time today or tomorrow
/// (`09:00`, taken as tomorrow when it has already passed), `tomorrow 09:00`,
/// or a delay from now (`30m`, `2h`).
fn parse_at(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    let norm = s.replace('T', " ");
    if let Ok(dt) = NaiveDateTime::parse_from_str(&norm, "%Y-%m-%d %H:%M") {
        return Some(dt);
    }
    let now = Local::now().naive_local();
    let lower = norm.to_lowercase();
    if let Some(rest) = lower
        .strip_prefix("tomorrow ")
        .or_else(|| lower.strip_prefix("demain "))
    {
        let (hour, minute) = parse_hhmm(rest.trim())?;
        return (now.date() + ChronoDuration::days(1)).and_hms_opt(hour, minute, 0);
    }
    if let Some((hour, minute)) = parse_hhmm(s) {
        let today = now.date().and_hms_opt(hour, minute, 0)?;
        return Some(if today > now {
            today
        } else {
            today + ChronoDuration::days(1)
        });
    }
    let secs = parse_duration(s)?;
    Some(now + ChronoDuration::seconds(secs as i64))
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    let out: String = out.trim_matches('-').chars().take(40).collect();
    let out = out.trim_matches('-').to_string();
    if out.is_empty() { "reminder".to_string() } else { out }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn fail(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1);
}

fn home() -> PathBuf {
    PathBuf::from(env::var("HOME").expect("HOME not set"))
}
fn agents_dir() -> PathBuf {
    home().join("Library/LaunchAgents")
}
fn meta_dir() -> PathBuf {
    home().join(".pomo/reminders")
}
fn plist_path(label: &str) -> PathBuf {
    agents_dir().join(format!("{label}.plist"))
}
fn meta_path(name: &str) -> PathBuf {
    meta_dir().join(format!("{name}.meta"))
}

fn current_uid() -> String {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn load_agent(plist: &Path, label: &str) {
    let uid = current_uid();
    // Remove any prior instance so re-scheduling the same name is idempotent.
    let _ = Command::new("launchctl")
        .args(["bootout".to_string(), format!("gui/{uid}/{label}")])
        .output();
    let out = Command::new("launchctl")
        .args([
            "bootstrap".to_string(),
            format!("gui/{uid}"),
            plist.to_string_lossy().to_string(),
        ])
        .output();
    if let Ok(o) = out
        && !o.status.success()
    {
        let err = String::from_utf8_lossy(&o.stderr);
        if !err.trim().is_empty() {
            eprintln!("launchctl: {}", err.trim());
        }
    }
}

fn unload_agent(label: &str) {
    let uid = current_uid();
    let _ = Command::new("launchctl")
        .args(["bootout".to_string(), format!("gui/{uid}/{label}")])
        .output();
}

fn remind_create(msg: String, sched: Schedule, until: Option<String>, name: String) {
    let label = format!("{LABEL_PREFIX}{name}");
    let exe = env::current_exe()
        .expect("current_exe")
        .to_string_lossy()
        .to_string();
    let mut prog = vec![
        exe,
        "__fire".to_string(),
        "--label".to_string(),
        label.clone(),
        "--msg".to_string(),
        msg.clone(),
    ];
    if let Some(u) = &until {
        prog.push("--until".to_string());
        prog.push(u.clone());
    }
    if matches!(sched, Schedule::Once(_)) {
        // Tells the fired job to tear itself down: a one-shot never repeats.
        prog.push("--once".to_string());
    }
    let prog_xml: String = prog
        .iter()
        .map(|a| format!("        <string>{}</string>\n", xml_escape(a)))
        .collect();
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
{prog_xml}    </array>
{sched_xml}
    <key>RunAtLoad</key>
    <false/>
    <key>StandardErrorPath</key>
    <string>/tmp/{label}.err</string>
</dict>
</plist>
"#,
        sched_xml = sched.xml()
    );
    fs::create_dir_all(agents_dir()).ok();
    fs::create_dir_all(meta_dir()).ok();
    let pp = plist_path(&label);
    fs::write(&pp, plist).unwrap_or_else(|e| fail(&format!("Failed to write plist: {e}")));
    let meta = format!(
        "name={name}\nlabel={label}\nschedule={}\nmsg={}\nuntil={}\n",
        sched.human(),
        msg.replace('\n', " "),
        until.clone().unwrap_or_default(),
    );
    fs::write(meta_path(&name), meta).ok();
    load_agent(&pp, &label);
    println!("Reminder '{name}' scheduled: {}.", sched.human());
    if let Some(u) = &until {
        println!("  Active until {u} (removes itself afterwards).");
    }
    println!("  Message: {msg}");
    println!("  Remove with: pomo remind rm {name}");
}

fn remind_list() {
    let mut found = false;
    if let Ok(entries) = fs::read_dir(meta_dir()) {
        let mut metas: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "meta").unwrap_or(false))
            .collect();
        metas.sort();
        for p in metas {
            if let Ok(content) = fs::read_to_string(&p) {
                let get = |k: &str| -> String {
                    content
                        .lines()
                        .find_map(|l| l.strip_prefix(&format!("{k}=")))
                        .unwrap_or("")
                        .to_string()
                };
                found = true;
                let name = get("name");
                let sched = get("schedule");
                let msg = get("msg");
                let until = get("until");
                let until_suffix = if until.is_empty() {
                    String::new()
                } else {
                    format!("  ·  until {until}")
                };
                println!("• {name}");
                println!("    {sched}{until_suffix}");
                println!("    \"{msg}\"");
            }
        }
    }
    if !found {
        println!("No reminders scheduled.");
    }
}

fn remind_rm(name: &str) {
    let label = format!("{LABEL_PREFIX}{name}");
    unload_agent(&label);
    let pp = plist_path(&label);
    let existed = pp.exists();
    fs::remove_file(&pp).ok();
    fs::remove_file(meta_path(name)).ok();
    if existed {
        println!("Removed reminder '{name}'.");
    } else {
        eprintln!("No reminder named '{name}'.");
        std::process::exit(1);
    }
}

fn print_usage() {
    println!("pomo — terminal timer, stopwatch and scheduled reminders.");
    println!();
    println!("Timer / stopwatch:");
    println!("  pomo                            # stopwatch, counts up");
    println!("  pomo 25m                        # countdown (d/h/m/s, e.g. 90s, 1h30m)");
    println!("  pomo 14:30                      # countdown to a target time (HH:MM)");
    println!("  pomo 25m -t standup             # title above the timer (must be last)");
    println!("  pomo -s 1|2|3                   # display size: 1 text, 2 compact, 3 large (default)");
    println!();
    println!("Reminders (launchd):");
    println!("  pomo remind \"message\" --at 09:00             # once: HH:MM, YYYY-MM-DD HH:MM, or a delay (2h)");
    println!("  pomo remind \"message\" --every 30m            # interval: s/m/h/d");
    println!("  pomo remind \"message\" --daily 09:00          # every day at HH:MM");
    println!("  pomo remind \"message\" --weekly mon,wed 09:00 # given weekdays at HH:MM");
    println!("  pomo remind ... --until 2026-07-15           # auto-removes afterwards");
    println!("  pomo remind ... --name <slug>                # explicit name (default: from message)");
    println!("  pomo list                                    # list scheduled reminders");
    println!("  pomo rm <name>                               # remove one");
}

fn print_remind_usage() {
    eprintln!("Usage:");
    eprintln!("  pomo remind \"message\" --at 09:00             # once: HH:MM, YYYY-MM-DD HH:MM, or a delay (2h)");
    eprintln!("  pomo remind \"message\" --every 30m            # interval: s/m/h/d");
    eprintln!("  pomo remind \"message\" --daily 09:00          # every day at HH:MM");
    eprintln!("  pomo remind \"message\" --weekly mon,wed 09:00 # given weekdays at HH:MM");
    eprintln!("  pomo remind ... --until 2026-07-15           # auto-removes afterwards");
    eprintln!("  pomo remind ... --name <slug>                # explicit name (default: from message)");
    eprintln!("  pomo remind list");
    eprintln!("  pomo remind rm <name>");
}

fn remind_parse_create(args: &[String]) {
    let mut msg: Option<String> = None;
    let mut every: Option<String> = None;
    let mut daily: Option<String> = None;
    let mut weekly: Option<(String, String)> = None;
    let mut at: Option<String> = None;
    let mut until: Option<String> = None;
    let mut name: Option<String> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--every" => {
                every = Some(it.next().cloned().unwrap_or_else(|| {
                    fail("--every needs a duration (e.g. 30s, 15m, 2h, 1d)")
                }))
            }
            "--daily" => {
                daily = Some(
                    it.next()
                        .cloned()
                        .unwrap_or_else(|| fail("--daily needs a time (HH:MM)")),
                )
            }
            "--weekly" => {
                let d = it.next().cloned().unwrap_or_else(|| {
                    fail("--weekly needs days then a time, e.g. --weekly mon,wed,fri 09:00")
                });
                let t = it.next().cloned().unwrap_or_else(|| {
                    fail("--weekly needs a time after the days, e.g. --weekly mon,wed,fri 09:00")
                });
                weekly = Some((d, t));
            }
            "--at" => {
                at = Some(it.next().cloned().unwrap_or_else(|| {
                    fail("--at needs a time (09:00), a datetime (2026-08-12 09:00), or a delay (2h)")
                }))
            }
            "--until" => {
                until = Some(it.next().cloned().unwrap_or_else(|| {
                    fail("--until needs a date (YYYY-MM-DD) or datetime (YYYY-MM-DD HH:MM)")
                }))
            }
            "--name" => {
                name = Some(
                    it.next()
                        .cloned()
                        .unwrap_or_else(|| fail("--name needs a value")),
                )
            }
            s if s.starts_with("--") => fail(&format!("Unknown option: {s}")),
            _ => {
                if msg.is_none() {
                    msg = Some(a.clone());
                } else {
                    fail("Multiple message words detected — wrap the message in quotes: pomo remind \"your message\" --daily 09:00");
                }
            }
        }
    }
    let msg = msg.unwrap_or_else(|| {
        print_remind_usage();
        std::process::exit(1);
    });
    let count = every.is_some() as u8
        + daily.is_some() as u8
        + weekly.is_some() as u8
        + at.is_some() as u8;
    if count == 0 {
        fail("Choose a schedule: --at WHEN, --every DUR, --daily HH:MM, or --weekly DAYS HH:MM");
    }
    if count > 1 {
        fail("Choose only one of --at / --every / --daily / --weekly");
    }
    let sched = if let Some(a) = at {
        let when = parse_at(&a).unwrap_or_else(|| {
            fail(&format!(
                "Invalid --at value: {a} (expected HH:MM, YYYY-MM-DD HH:MM, tomorrow HH:MM, or a delay like 2h)"
            ))
        });
        if when <= Local::now().naive_local() {
            fail(&format!("--at {a} is in the past"));
        }
        Schedule::Once(when)
    } else if let Some(e) = every {
        let secs = parse_duration(&e).unwrap_or_else(|| fail(&format!("Invalid duration: {e}")));
        Schedule::Interval(secs)
    } else if let Some(d) = daily {
        let (hour, minute) =
            parse_hhmm(&d).unwrap_or_else(|| fail(&format!("Invalid time: {d} (expected HH:MM)")));
        Schedule::Daily { hour, minute }
    } else {
        let (ds, t) = weekly.unwrap();
        let (hour, minute) =
            parse_hhmm(&t).unwrap_or_else(|| fail(&format!("Invalid time: {t} (expected HH:MM)")));
        let days: Vec<u32> = ds
            .split(',')
            .map(|d| {
                parse_weekday(d.trim()).unwrap_or_else(|| {
                    fail(&format!("Invalid day: {d} (use mon,tue,wed,thu,fri,sat,sun)"))
                })
            })
            .collect();
        Schedule::Weekly { days, hour, minute }
    };
    if let Some(u) = &until
        && parse_until(u).is_none()
    {
        fail(&format!(
            "Invalid --until date: {u} (expected YYYY-MM-DD or YYYY-MM-DD HH:MM)"
        ));
    }
    // Always slugify: an explicit --name goes into the plist Label and
    // StandardErrorPath (which are not XML-escaped), so it must be reduced to
    // the same safe alphanumeric/dash slug as a name derived from the message.
    let name = slugify(&name.unwrap_or_else(|| msg.clone()));
    remind_create(msg, sched, until, name);
}

fn remind_cmd(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("-h") | Some("--help") | Some("help") => print_remind_usage(),
        None => {
            print_remind_usage();
            std::process::exit(1);
        }
        Some("list") => remind_list(),
        Some("rm") | Some("remove") => match args.get(1) {
            Some(n) => remind_rm(n),
            None => fail("Usage: pomo remind rm <name>"),
        },
        _ => remind_parse_create(args),
    }
}

/// How long the Snooze button pushes a reminder back.
const SNOOZE_SECS: u64 = 10 * 60;
/// Snooze button label. Spells out the delay so the dialog needs no explaining.
const SNOOZE_BUTTON: &str = "Snooze 10m";

/// The name a snooze of `name` must take: the base name plus a snooze suffix
/// that alternates, so snoozing a snooze never targets its own label.
fn snooze_name(name: &str) -> String {
    if let Some(base) = name.strip_suffix(SNOOZE_SUFFIX_ALT) {
        format!("{base}{SNOOZE_SUFFIX}")
    } else if let Some(base) = name.strip_suffix(SNOOZE_SUFFIX) {
        format!("{base}{SNOOZE_SUFFIX_ALT}")
    } else {
        format!("{name}{SNOOZE_SUFFIX}")
    }
}

/// Schedule a one-shot copy of a fired reminder SNOOZE_SECS from now, under its
/// own `<name>-snooze` label. The original is untouched: a recurring reminder
/// keeps its schedule, a one-shot still tears itself down after firing.
fn snooze_reminder(label: &str, msg: &str) {
    let name = label.strip_prefix(LABEL_PREFIX).unwrap_or(label);
    let at = Local::now().naive_local() + ChronoDuration::seconds(SNOOZE_SECS as i64);
    remind_create(msg.to_string(), Schedule::Once(at), None, snooze_name(name));
}

/// Invoked by launchd. Show the reminder, or tear it down if --until has passed.
fn fire_cmd(args: &[String]) {
    let mut label = String::new();
    let mut msg = String::new();
    let mut until: Option<String> = None;
    let mut once = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--label" => label = it.next().cloned().unwrap_or_default(),
            "--msg" => msg = it.next().cloned().unwrap_or_default(),
            "--until" => until = it.next().cloned(),
            "--once" => once = true,
            _ => {}
        }
    }
    if let Some(u) = &until
        && let Some(dt) = parse_until(u)
        && Local::now().naive_local() >= dt
    {
        // Deadline passed: delete our own files FIRST, because the bootout below
        // terminates this very process (we are the launchd job) and would
        // otherwise kill us before the files are removed.
        fs::remove_file(plist_path(&label)).ok();
        if let Some(name) = label.strip_prefix(LABEL_PREFIX) {
            fs::remove_file(meta_path(name)).ok();
        }
        unload_agent(&label);
        return;
    }
    // OK just dismisses; Snooze re-fires the same message shortly after;
    // Disable tears the reminder down so it never fires again (same teardown as
    // `pomo remind rm`). A one-shot gets no Disable button: it tears itself down
    // whichever button was clicked, so Disable would be a duplicate of OK.
    let buttons: &[&str] = if once {
        &[SNOOZE_BUTTON, "OK"]
    } else {
        &["Disable", SNOOZE_BUTTON, "OK"]
    };
    let choice = show_dialog("Reminder", &msg, buttons, "OK");
    if choice == SNOOZE_BUTTON {
        // Before any teardown below: unload_agent kills this very process.
        snooze_reminder(&label, &msg);
    }
    if choice == "Disable" || once {
        // Remove our own files FIRST: unload_agent below boots out this very
        // launchd job and would otherwise kill us before the files are removed.
        fs::remove_file(plist_path(&label)).ok();
        if let Some(name) = label.strip_prefix(LABEL_PREFIX) {
            fs::remove_file(meta_path(name)).ok();
        }
        unload_agent(&label);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("-h") | Some("--help") | Some("help") => {
            print_usage();
            return;
        }
        Some("remind") => {
            remind_cmd(&args[2..]);
            return;
        }
        Some("__fire") => {
            fire_cmd(&args[2..]);
            return;
        }
        // Root-level shortcuts for reminders (list/rm are not valid durations).
        Some("list") => {
            remind_list();
            return;
        }
        Some("rm") | Some("remove") => {
            match args.get(2) {
                Some(n) => remind_rm(n),
                None => fail("Usage: pomo rm <name>"),
            }
            return;
        }
        _ => {}
    }

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
    // Save the current title (XTWINOPS push); the guard pops it back on exit.
    execute!(stdout, Print("\x1b[22;0t")).ok();
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
    let mut last_title = String::new();

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

        // Mirror the timer into the terminal title (Kova pane title, tab title…).
        let pane_title = match &title {
            Some(t) => format!("{t} — {time_str}"),
            None => format!("pomo {time_str}"),
        };
        if pane_title != last_title {
            execute!(stdout, Print(osc_title(&pane_title))).ok();
            last_title = pane_title;
        }

        let eff_size = fit_size(&time_str, cols, size);
        let big = render_big(&time_str, cols, eff_size);
        let digit_lines = eff_size.height() as u16;
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

    // Capture end time and duration before the blocking dialog: it stays open
    // until the user clicks OK, which would otherwise inflate both.
    let end_time = Local::now();
    let elapsed = format_duration_human(start.elapsed().unwrap_or_default().as_secs());

    if matches!(mode, Mode::Timer { .. }) {
        notify("Time's up!");
    }

    drop(guard);
    println!();
    println!("  Started:  {}", start_time.format("%Y-%m-%d %H:%M:%S"));
    println!("  Duration: {}", elapsed);
    println!("  Ended:    {}", end_time.format("%Y-%m-%d %H:%M:%S"));
    println!();
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Clear the title (fallback for terminals without XTWINOPS), then pop
        // the saved one back where supported.
        let _ = execute!(stdout(), Print(osc_title("")), Print("\x1b[23;0t"));
        let _ = execute!(stdout(), terminal::LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snooze_name_alternates() {
        // A snooze must never target the label of the job creating it: booting
        // out our own label kills us before the new job is bootstrapped.
        assert_eq!(snooze_name("pilulier"), "pilulier-snooze");
        assert_eq!(snooze_name("pilulier-snooze"), "pilulier-snooze-2");
        assert_eq!(snooze_name("pilulier-snooze-2"), "pilulier-snooze");
    }

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
    fn test_parse_at() {
        let now = Local::now().naive_local();

        // Explicit datetime, both separators.
        assert_eq!(
            parse_at("2026-08-12 09:00"),
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap().and_hms_opt(9, 0, 0)
        );
        assert_eq!(parse_at("2026-08-12T09:00"), parse_at("2026-08-12 09:00"));

        // A delay from now lands in the future.
        let in_two_hours = parse_at("2h").unwrap();
        assert!(in_two_hours > now);
        assert!((in_two_hours - now).num_minutes() >= 119);

        // Bare HH:MM always resolves to the next occurrence.
        assert!(parse_at("09:00").unwrap() > now);
        assert!(parse_at("tomorrow 09:00").unwrap() > now);

        // Rejected.
        assert_eq!(parse_at("25:00"), None);
        assert_eq!(parse_at("abc"), None);
        assert_eq!(parse_at(""), None);
    }

    #[test]
    fn test_once_schedule_xml_pins_the_date() {
        let at = NaiveDate::from_ymd_opt(2026, 8, 12)
            .unwrap()
            .and_hms_opt(9, 5, 0)
            .unwrap();
        let xml = Schedule::Once(at).xml();
        assert!(xml.contains("<key>Month</key><integer>8</integer>"));
        assert!(xml.contains("<key>Day</key><integer>12</integer>"));
        assert!(xml.contains("<key>Hour</key><integer>9</integer>"));
        assert!(xml.contains("<key>Minute</key><integer>5</integer>"));
        assert_eq!(Schedule::Once(at).human(), "once at 2026-08-12 09:05");
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
