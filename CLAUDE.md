# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

CLI pomodoro/timer/stopwatch for the terminal (macOS) written in Rust. Displays large ASCII digits and sends a native macOS notification + sound when a timer ends.

## Commands

```bash
cargo build              # dev build
cargo build --release    # release build
cargo run                # stopwatch mode (counts up)
cargo run -- 25m         # timer mode (countdown)
cargo test               # run tests
cargo clippy             # lint
```

## Modes

- **No argument** (`pomo`): stopwatch, counts up from 00:00, no notification
- **With duration** (`pomo 25m`): countdown timer, notification + sound at the end

## Duration Format

Accepts `25m`, `90s`, `1h30m`. Units: `h` (hours), `m` (minutes), `s` (seconds).

## Architecture

Single `src/main.rs`. Key components:
- `parse_duration` — parses duration strings
- `render_big` — ASCII digit rendering (7-line high glyphs), centered horizontally
- Main loop uses `Instant`-based timing (no drift), polls crossterm events at 100ms
- Alternate screen + raw mode with `RawModeGuard` (Drop-based cleanup)
- macOS notification via `osascript`
