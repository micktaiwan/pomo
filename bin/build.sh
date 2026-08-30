#!/usr/bin/env bash
# Build pomo and refresh the installed binary + every launchd reminder.
#
# /usr/local/bin/pomo is a REAL COPY of target/release/pomo, not a symlink, and the
# reminder agents are re-registered on every install. Both rules exist for the same
# reason, learned on 2026-08-28 when the 18:00 reminder silently did not fire.
#
# macOS records a launch constraint for a LaunchAgent when its plist is bootstrapped:
# Background Task Management pins the executable's code-signing identity at that
# moment. `cargo build --release` rewrites target/release/pomo in place, which changes
# its CDHash, so the next time launchd fired the job AMFI killed it before main() ran:
#
#   AMFI: '.../target/release/pomo' has no CMS blob?
#   AMFI: Launch Constraint Violation (enforcing) ... (Constraint not matched)
#   launchd: removing service since it exited with consistent failure
#
# The job died with OS_REASON_CODESIGNING, launchd disabled it, and nothing was written
# to StandardErrorPath — the failure is completely silent from the user's side. With the
# symlink, every rebuild broke all eight reminders at once, medication included.
#
# So: sign explicitly (cargo's linker-signed binaries carry no CMS blob, which is what
# AMFI objected to), install a real copy, then bootout/bootstrap each agent so BTM
# re-pins the identity of the binary that is actually there now.
set -euo pipefail

cd "$(dirname "$0")/.."

DEST=/usr/local/bin/pomo
UID_N=$(id -u)

# Not ./target: CARGO_TARGET_DIR is set to ~/.cargo/target in .zshrc, so the build
# lands outside the repo. Ask cargo rather than guess — it honours both the env var
# and .cargo/config.toml, and returns the same answer from any shell.
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
BIN="$TARGET_DIR/release/pomo"

cargo clippy --release -- -D warnings
cargo test --release
cargo build --release

# Explicit ad-hoc signature. Replaces cargo's linker-signed one, which has no CMS blob.
codesign -f -s - "$BIN"
codesign -v "$BIN"

# Install a real copy. ditto preserves the embedded signature and the xattrs that
# codesign validation reads; the rm first is what drops any leftover symlink.
rm -f "$DEST"
ditto "$BIN" "$DEST"

# Re-pin every reminder against the binary we just installed. Without this the agents
# still carry the launch constraint recorded for the previous build and die on their
# next fire, exactly as described above.
shopt -s nullglob
agents=("$HOME/Library/LaunchAgents/com.mick.pomo."*.plist)
for plist in "${agents[@]}"; do
    label=$(basename "$plist" .plist)
    launchctl bootout "gui/$UID_N/$label" 2>/dev/null || true
    launchctl bootstrap "gui/$UID_N" "$plist"
    echo "re-registered $label"
done

echo
codesign -v "$DEST"
echo "installed $DEST, signature verified"
echo "${#agents[@]} reminder(s) re-registered"
