# pomo

CLI timer / pomodoro / stopwatch for the terminal, with large ASCII digits and macOS notifications.

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

## Notifications

When a timer ends, a macOS notification with sound is sent.

For best results, install [terminal-notifier](https://github.com/julienXX/terminal-notifier):

```bash
brew install terminal-notifier
```

Without it, pomo falls back to `osascript` (clicking the notification will open Script Editor).

## Build

```bash
cargo build --release
```
