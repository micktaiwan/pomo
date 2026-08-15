# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

CLI pomodoro/timer/stopwatch for the terminal (macOS) written in Rust. Displays large ASCII digits and sends a native macOS notification + sound when a timer ends.

## Commands

```bash
cargo build              # dev build
cargo build --release    # release build
cargo run                # stopwatch mode (counts up)
cargo run -- 25m         # timer mode (countdown from duration)
cargo run -- 14:30       # timer mode (countdown to target time)
cargo run -- 25m -t foo  # timer with title
cargo test               # run tests
cargo clippy             # lint
```

## Modes

- **No argument** (`pomo`): stopwatch, counts up from 00:00, no notification
- **With duration** (`pomo 25m`): countdown timer, notification + sound at the end
- **With target time** (`pomo 14:30`): countdown to a specific time (HH:MM), notification + sound at the end. If the time has already passed today, targets tomorrow.

## Input Formats

- **Duration**: `25m`, `90s`, `1h30m`, `2d` (days). Units: `d` (days), `h` (hours), `m` (minutes), `s` (seconds).
- **Target time**: `HH:MM` (24h format, e.g. `14:30`, `9:00`).

## Options

- `-s 1|2|3` — display size: 1 = text, 2 = compact, 3 = large (default)
- `-t`, `--title TEXT...` — title above the timer. Must be last option (consumes all remaining args).

## Reminders

`pomo remind` writes a launchd LaunchAgent per reminder. `--at WHEN` is the one-shot
form (`HH:MM`, `tomorrow HH:MM`, `YYYY-MM-DD HH:MM`, or a delay like `2h`): the plist
pins Month + Day + Hour + Minute and the fired job removes its own plist and metadata,
so it never repeats. `--every` / `--daily` / `--weekly` are the recurring forms.

The fired dialog of a recurring reminder has three buttons: `OK` dismisses, `Disable`
tears the reminder down, `Snooze 10m` schedules a one-shot copy named `<name>-snooze`
ten minutes later while leaving the original schedule untouched. A one-shot shows only
`Snooze 10m` and `OK`: it tears itself down whichever button is clicked, so `Disable`
would duplicate `OK`.

## Architecture

Single `src/main.rs`. Key components:
- `parse_duration` — parses duration strings (`25m`, `1h30m`)
- `parse_target_time` — parses target time (`HH:MM`) and computes remaining seconds
- `render_big` — ASCII digit rendering (7-line high glyphs), centered horizontally
- Main loop uses `Instant`-based timing (no drift), polls crossterm events at 100ms
- Terminal title mirrors the countdown via OSC 0 (`osc_title`), saved/restored with XTWINOPS push/pop
- Alternate screen + raw mode with `RawModeGuard` (Drop-based cleanup)
- macOS notification via `osascript`

## The `pomo` skill is this tool's public surface

`~/.claude/skills/pomo/SKILL.md` — the real file is
`~/projects/perso/dotfiles/claude/skills/pomo/SKILL.md`, the symlink is only how Claude Code sees
it — is what tells a session anywhere on this Mac how to use pomo: the commands, the paths, the
ports, what it must not do. Nothing keeps it in sync automatically.

**A change here that alters what the skill promises must land in the skill in the same breath**:
a command or a flag, a socket or data path, a port, a default value, a config file name, a rule
about what a session may touch. Leaving it stale is worse than having no skill at all, because a
session will act on what it says. `/skill-check pomo` compares the two and reports what drifted.
