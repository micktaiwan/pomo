# pomo

CLI timer / pomodoro / stopwatch for the terminal, with large ASCII digits and macOS notifications.

## Usage

```bash
pomo            # stopwatch (counts up)
pomo 25m        # classic 25 min pomodoro
pomo 5m         # 5 min break
pomo 90s        # 90 seconds
pomo 1h30m      # 1 hour 30
pomo 2j         # 2 days
```

Quit: `Ctrl+C`, `q` or `Esc`.

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
