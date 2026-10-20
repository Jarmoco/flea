#!/usr/bin/env bash
. "$(dirname "$0")/headless-commons.sh"
# The Theme page's own colour row: the panel opens the row's editor, a hex typed
# into it lands in the state, and the palette is laid over again.
set -u
. "$(dirname "$0")/../tools/flea-sandbox-guard"
cd "$(dirname "$0")/.." || exit 1

if ! command -v qs >/dev/null; then
    echo "theme-editor.sh: qs is not installed, cannot drive the colour editor"
    exit 1
fi

sandbox_root_ok
test_root=$(mktemp -d "$SANDBOX_ROOT/flea-themeeditor.XXXXXX") || exit 1
: > "$test_root/$SANDBOX_MARKER" || exit 1
cleanup() { sandbox_remove "$test_root"; }
trap cleanup EXIT

mkdir -p "$test_root/config" "$test_root/home" "$test_root/state" "$test_root/runtime" || exit 1
chmod 700 "$test_root/runtime" || exit 1
# The probe imports ui/ as Flea, and ui/'s qs.Commons resolves against this root, as it does from ui/boot.
ln -s "$PWD/ui" "$test_root/config/flea" || exit 1
headless_commons "$test_root" || exit 1
ln -s "$(readlink -f ui/boot/Ui)" "$test_root/config/Ui" || exit 1
cp tests/theme-editor.qml "$test_root/config/shell.qml" || exit 1
cp tests/qml-probe.js "$test_root/config/qml-probe.js" || exit 1

output=$(env -u DISPLAY -u WAYLAND_DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE \
    HOME="$test_root/home" XDG_STATE_HOME="$test_root/state" XDG_RUNTIME_DIR="$test_root/runtime" \
    FLEA_BIN="${FLEA_BIN:-$PWD/target/debug/flea}" \
    QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME=generic QT_FORCE_STDERR_LOGGING=1 \
    timeout 60 qs -p "$test_root/config" 2>&1)

# Sample input, one probe line: "  INFO qml: THEMEEDITOR PASS 6 checks"
pass_count=$(printf '%s\n' "$output" | grep -c 'THEMEEDITOR PASS')
fail_count=$(printf '%s\n' "$output" | grep -c 'THEMEEDITOR FAIL')
noisy=$(printf '%s\n' "$output" | grep -av 'does not support setting window masks' | grep -aiE 'WARN|ERROR|Binding loop|failed to load|Unable to assign|Cannot assign|TypeError|ReferenceError|is not a function|is not defined')
if [ "$pass_count" -ne 1 ] || [ "$fail_count" -ne 0 ] || [ -n "$noisy" ]; then
    printf 'FAIL the colour editor did not open, did not land its colour, or the probe logged a QML error\n'
    printf '%s\n' "$output" | grep -a 'THEMEEDITOR'
    printf '%s\n' "$output" | grep -aiE 'ERROR|error'
    printf '%s\n' "$output"
    exit 1
fi
# The colour the panel wrote reached the state file through the one writer, which is
# the binary the window asks.
state_file="$test_root/state/flea/ui.json"
if [ ! -f "$state_file" ] || ! grep -q '"colors"' "$state_file"; then
    printf 'FAIL the colour the panel wrote did not reach the state file\n'
    exit 1
fi
printf '%s\n' "$output" | grep -o 'THEMEEDITOR PASS.*'
