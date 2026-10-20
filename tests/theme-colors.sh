#!/usr/bin/env bash
. "$(dirname "$0")/headless-commons.sh"
# The Theme page's own colours: the operator's own over the theme's, one hex for
# every role the app draws with, and the palette laid over again when they change.
set -u
. "$(dirname "$0")/../tools/flea-sandbox-guard"
cd "$(dirname "$0")/.." || exit 1

if ! command -v qs >/dev/null; then
    echo "theme-colors.sh: qs is not installed, cannot drive the theme colours"
    exit 1
fi

sandbox_root_ok
test_root=$(mktemp -d "$SANDBOX_ROOT/flea-themecolors.XXXXXX") || exit 1
: > "$test_root/$SANDBOX_MARKER" || exit 1
cleanup() { sandbox_remove "$test_root"; }
trap cleanup EXIT

mkdir -p "$test_root/config" "$test_root/home" "$test_root/state" "$test_root/runtime" || exit 1
chmod 700 "$test_root/runtime" || exit 1
# The state file the window reads before its first paint: the operator's own
# colours, which is the whole path the probe reads back.
mkdir -p "$test_root/state/flea" || exit 1
cat > "$test_root/state/flea/ui.json" <<'JSON'
{"theme": {"name": "", "colors": {"background": "#0a0b0c", "accent": "#3a7bd5"}}}
JSON
# The probe imports ui/ as Flea, and ui/'s qs.Commons resolves against this root, as it does from ui/boot.
ln -s "$PWD/ui" "$test_root/config/flea" || exit 1
headless_commons "$test_root" || exit 1
ln -s "$(readlink -f ui/boot/Ui)" "$test_root/config/Ui" || exit 1
cp tests/theme-colors.qml "$test_root/config/shell.qml" || exit 1

output=$(env -u DISPLAY -u WAYLAND_DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE \
    HOME="$test_root/home" XDG_STATE_HOME="$test_root/state" XDG_RUNTIME_DIR="$test_root/runtime" \
    QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME=generic QT_FORCE_STDERR_LOGGING=1 \
    timeout 30 qs -p "$test_root/config" 2>&1)

# Sample input, one probe line: "  INFO qml: THEMECOLORS PASS own=9 kept=1 derived=2 reset=3"
pass_count=$(printf '%s\n' "$output" | grep -c 'THEMECOLORS PASS')
fail_count=$(printf '%s\n' "$output" | grep -c 'THEMECOLORS FAIL')
noisy=$(printf '%s\n' "$output" | grep -av 'does not support setting window masks' | grep -aiE 'WARN|ERROR|Binding loop|failed to load|Unable to assign|Cannot assign|TypeError|ReferenceError|is not a function|is not defined')
if [ "$pass_count" -ne 1 ] || [ "$fail_count" -ne 0 ] || [ -n "$noisy" ]; then
    printf 'FAIL the operator own colours did not reach the palette, or the probe logged a QML error\n'
    printf '%s\n' "$output" | grep -a 'THEMECOLORS'
    printf '%s\n' "$output" | grep -aiE 'ERROR|error'
    printf '%s\n' "$output"
    exit 1
fi
printf '%s\n' "$output" | grep -o 'THEMECOLORS PASS.*'
