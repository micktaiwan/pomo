# pomo

CLI timer / pomodoro / stopwatch for the terminal, with large ASCII digits, a blocking end-of-timer dialog, and launchd-backed scheduled reminders that survive reboot.

## Usage

```bash
pomo                          # stopwatch (counts up)
pomo 25m                      # classic 25 min pomodoro
pomo 5m                       # 5 min break
pomo 90s                      # 90 seconds
pomo 1h30m                    # 1 hour 30
pomo 2d                       # 2 days
pomo 14:30                    # countdown to 14:30 (tomorrow if already passed)
pomo 25m -t daily standup     # timer with title
pomo -s 2 25m                 # compact display size
```

### Options

- `-s 1|2|3` — display size: 1 = text, 2 = compact, 3 = large (default)
- `-t`, `--title TEXT...` — title displayed above the timer. Must be last option (all remaining args are the title).

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

Schedule recurring reminders that survive reboot. Each reminder is a launchd
LaunchAgent that fires the same blocking dialog on a schedule.

```bash
pomo remind "message" --every 30m                    # interval: s / m / h / d
pomo remind "message" --daily 09:00                  # every day at HH:MM
pomo remind "message" --weekly mon,wed,fri 09:30     # given weekdays at HH:MM
pomo remind "message" --daily 09:00 --until 2026-07-15   # auto-removes afterwards
pomo remind "message" --daily 09:00 --name my-slug   # explicit name (default: from message)
pomo remind list                                     # list active reminders
pomo remind rm <name>                                # remove one
```

`pomo list` and `pomo rm <name>` are root-level shortcuts for the two above.

Exactly one schedule is required (`--every` / `--daily` / `--weekly`).

Notes:

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
