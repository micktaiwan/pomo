# pomo

CLI timer / pomodoro / stopwatch for the terminal, with large ASCII digits, a blocking end-of-timer dialog, and launchd-backed scheduled reminders that survive reboot.

## Usage

```bash
pomo                          # stopwatch (counts up)
pomo 25m                      # classic 25 min pomodoro
pomo 5m                       # 5 min break
pomo 90s                      # 90 seconds
pomo 1h30m                    # 1 hour 30
pomo 1h30                     # same (trailing number takes the next smaller unit)
pomo 2d                       # 2 days
pomo 5min                     # spelled-out units too (min/mins/minutes, sec, hour, day)
pomo 14:30                    # countdown to 14:30 (tomorrow if already passed)
pomo 25m -t daily standup     # timer with title
pomo -s 2 25m                 # compact display size
```

### Options

- `-s 1|2|3` — display size: 1 = text, 2 = compact, 3 = large (default)
- `-r`, `--repeat` — the end-of-timer dialog gets a second button, `Snooze <duration>`, which restarts the same countdown. Rounds never chain on their own: one click, one round. Needs a duration (not a target time, not the stopwatch).
- `-t`, `--title TEXT...` — title displayed above the timer. Must be last option (all remaining args are the title).

### Terminal title

While running, pomo mirrors the countdown into the terminal title (window, tab,
or pane title depending on the terminal) via the standard OSC escape sequence:
`<title> — MM:SS` with `-t`, `pomo MM:SS` without. The previous title is
restored on exit.

### Controls

- `q` / `Esc` / `Ctrl+C` — quit
- `Space` / `p` — pause / resume
- `+` / `-` — adjust time by ±1 minute

## End-of-timer dialog

When a timer ends, pomo plays the Glass alert sound and shows a **blocking modal
dialog** (via `osascript`) that stays frontmost until you click OK — impossible to
miss, unlike a notification that stacks in Notification Center. No external
dependency required.

## Reminders

Schedule one-shot or recurring reminders that survive reboot. Each reminder is a
launchd LaunchAgent that fires the same blocking dialog on a schedule.

```bash
pomo remind "message" --at 09:00                     # once, at the next 09:00
pomo remind "message" --at "2026-08-12 09:00"        # once, at an explicit datetime
pomo remind "message" --at 2h                        # once, after a delay
pomo remind "message" --every 30m                    # interval: s / m / h / d
pomo remind "message" --daily 09:00                  # every day at HH:MM
pomo remind "message" --weekly mon,wed,fri 09:30     # given weekdays at HH:MM
pomo remind "message" --daily 09:00 --until 2026-07-15   # auto-removes afterwards
pomo remind "message" --daily 09:00 --name my-slug   # explicit name (default: from message)
pomo remind list                                     # list active reminders
pomo remind rm <name>                                # remove one
```

`pomo list` and `pomo rm <name>` are root-level shortcuts for the two above.

Exactly one schedule is required (`--at` / `--every` / `--daily` / `--weekly`).

Notes:

- **`--at WHEN`** fires once and then removes itself (LaunchAgent + metadata),
  whichever dialog button is clicked. `WHEN` is `HH:MM` (today, or tomorrow if that
  time has passed), `tomorrow HH:MM`, an explicit `YYYY-MM-DD HH:MM`, or a delay
  from now (`30m`, `2h`). A time in the past is rejected. Under the hood the plist
  pins Month + Day + Hour + Minute, and the job deletes itself when it fires, so it
  never comes back a year later.
- **`--every` floor**: launchd throttles relaunches to ~10s minimum, so very short
  intervals are smoothed to about 10 seconds.
- **`--until DATE`** stops the reminder once now reaches that instant. A bare date
  (`2026-07-15`) means midnight that day, so the last fire is the day before; pass
  `--until 2026-07-16` (or a datetime) to include the 15th. When the deadline
  passes, the reminder deletes its own LaunchAgent and metadata.
- LaunchAgents live in `~/Library/LaunchAgents/com.mick.pomo.<name>.plist`;
  metadata for `list` lives in `~/.pomo/reminders/<name>.meta`.

## Build

```bash
cargo build --release
```
