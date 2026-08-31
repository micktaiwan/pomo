# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

CLI pomodoro/timer/stopwatch for the terminal (macOS) written in Rust. Displays large ASCII digits and sends a native macOS notification + sound when a timer ends.

## Commands

```bash
./bin/build.sh           # lint + test + release build + INSTALL (the only way to ship)
cargo build              # dev build
cargo build --release    # release build, NOT installed — see below
cargo run                # stopwatch mode (counts up)
cargo run -- 25m         # timer mode (countdown from duration)
cargo run -- 14:30       # timer mode (countdown to target time)
cargo run -- 25m -t foo  # timer with title
cargo run -- --help      # usage (timer + reminders + alert)
cargo run -- alert "boom"  # blocking dialog right now
cargo test               # run tests
cargo clippy             # lint
```

## Installing: always `./bin/build.sh`, never `cargo build` alone

`/usr/local/bin/pomo` is a **real copy**, and `bin/build.sh` re-registers every
`com.mick.pomo.*` LaunchAgent after installing it. Both exist because of the same trap.

macOS pins a launch constraint on a LaunchAgent's executable when the plist is
bootstrapped: Background Task Management records its code-signing identity then. A
rebuild changes the binary's CDHash, so on the next fire AMFI kills the job before
`main()` runs — `OS_REASON_CODESIGNING`, `Launch Constraint Violation` — and launchd
disables the service. Nothing lands in `StandardErrorPath`: the reminder just never
appears. On 2026-08-28 the 18:00 reminder died this way, and while `/usr/local/bin/pomo`
was a symlink into the cargo target directory, a single `cargo build --release` armed
that failure for all eight reminders at once, medication included.

`bin/build.sh` therefore signs the binary explicitly (cargo's linker-signed output has
no CMS blob, which is what AMFI objects to), installs a real copy with `ditto`, and
bootouts/bootstraps each agent so BTM re-pins the identity of the binary now in place.
It resolves the build output through `cargo metadata` rather than `./target`, because
`CARGO_TARGET_DIR` points at `~/.cargo/target`.

A plain `cargo build --release` is fine for iterating. It is never how a change ships.

## Modes

- **No argument** (`pomo`): stopwatch, counts up from 00:00, no notification
- **With duration** (`pomo 25m`): countdown timer, notification + sound at the end
- **With target time** (`pomo 14:30`): countdown to a specific time (HH:MM), notification + sound at the end. If the time has already passed today, targets tomorrow.

## Input Formats

- **Duration**: `25m`, `90s`, `1h30m`, `2d` (days). Units: `d` (days), `h` (hours), `m` (minutes), `s` (seconds).
  A trailing number with no unit takes the next smaller one, so `1h30` == `1h30m` and `1m30` == `1m30s`. A bare number with no unit at all (`25`) is still rejected.
  Spelled-out units work too — `5min`, `5mins`, `5minutes`, `90sec`, `2hours`, `2days` — because that is what a hand types under pressure. `UNIT_WORDS` rewrites them to the single letter before parsing, longest form first: rewriting `second` before `seconds` would leave a stray `s` and reject a valid duration.
- **Target time**: `HH:MM` (24h format, e.g. `14:30`, `9:00`).

## Options

- `-s 1|2|3` — display size: 1 = text, 2 = compact, 3 = large (default)
- `-r`, `--repeat` — repeating timer, see below.
- `-t`, `--title TEXT...` — title above the timer. Must be last option (consumes all remaining args).
- `-h`, `--help` (or `pomo help`) — full usage, timer and reminders. `pomo remind --help` prints the reminder usage alone.

## `-r`: the repeating timer

`pomo 20m -r -t levo pharma` runs the same countdown, then ends on a dialog with two
buttons instead of one: `Snooze 20m` (default) restarts an identical round, `OK` ends the
session. The delay in the label is the timer's own length, not the reminders' fixed ten
minutes. Rounds are never chained automatically — the loop only advances on a click,
which is the whole point: the timer waits for you, it does not run behind your back.

It is *not* an alias for `pomo remind --every 20m`. This one is a foreground process
drawing the countdown and it dies with its terminal; a reminder is a launchd job that
fires with no terminal at all. Both exist on purpose.

`-r` requires a duration: a stopwatch never ends, and a target time (`14:30`) is a point
in the day rather than a length to replay. Both are rejected at parse time.

The rewrite that made this possible also split "the countdown reached zero" from "the
user pressed q": quitting a timer early used to ring the dialog anyway, because the loop
had a single exit. It now leaves silently.


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

Snoozing a snooze alternates the suffix (`-snooze` → `-snooze-2` → `-snooze`). It has to:
`load_agent` boots a label out before bootstrapping it, so re-scheduling under our own
label kills the running job between the bootout and the bootstrap — the plist and metadata
are already on disk, so the reminder shows up in `pomo list` but launchd never arms it.

## `pomo alert`

`pomo alert "message"` rings and shows the same blocking dialog straight away, with no
countdown and no launchd job behind it. It exists because the dialog used to be reachable
only through a timer or a reminder, so an alert that had to be seen *now* had to wait at
least until the next minute (a one-shot plist pins `Minute`, so `--at 30s` cannot fire
sooner). Single `OK` button; the command returns when it is clicked.

## Dialog icons

The three dialogs each carry their own icon, built from `tools/make-icons.py` into
`assets/icons/`: `once.icns` (blue clock) for a one-shot reminder and for a timer running
out — both are one-shot events — `repeat.icns` (green loop) for a recurring reminder, and
`alert.icns` (red triangle) for `pomo alert`.

`icons_dir()` takes the first directory that exists, in order: `POMO_ICONS_DIR`, an
`icons/` folder next to the installed binary, then `assets/icons` in this repo — that last
path is baked in at build time via `CARGO_MANIFEST_DIR`, so it is absolute and resolves
the same from a launchd job as from a shell. A missing directory or a missing `.icns` is
not an error: `icon_clause` falls back to `with icon note`, exactly what every dialog
looked like before.

Regenerate with `python3 tools/make-icons.py` (Pillow + `iconutil`). The script draws each
icon at 1024px and lets `iconutil` build the `.icns`; the intermediate `.iconset`
directories are removed on the way out.

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
