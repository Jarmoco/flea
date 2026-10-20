#!/usr/bin/env bash
. "$(dirname "$0")/headless-commons.sh"
# The Theme page's own plumbing: the picker reads the themes the launcher named
# in the environment, and choosing one sets the palette reader's theme
# directory at once.
set -u
. "$(dirname "$0")/../tools/flea-sandbox-guard"
cd "$(dirname "$0")/.." || exit 1

if ! command -v qs >/dev/null; then
    echo "theme-picker.sh: qs is not installed, cannot drive the theme picker"
    exit 1
fi

sandbox_root_ok
test_root=$(mktemp -d "$SANDBOX_ROOT/flea-themepicker.XXXXXX") || exit 1
: > "$test_root/$SANDBOX_MARKER" || exit 1
cleanup() { sandbox_remove "$test_root"; }
trap cleanup EXIT

mkdir -p "$test_root/config" "$test_root/home" "$test_root/state" "$test_root/runtime" || exit 1
chmod 700 "$test_root/runtime" || exit 1
# The probe imports ui/ as Flea, and ui/'s qs.Commons resolves against this root, as it does from ui/boot.
ln -s "$PWD/ui" "$test_root/config/flea" || exit 1
headless_commons "$test_root" || exit 1
ln -s "$(readlink -f ui/boot/Ui)" "$test_root/config/Ui" || exit 1
cp tests/theme-picker.qml "$test_root/config/shell.qml" || exit 1

# The launcher names the whole list in the environment, because the window
# cannot list a directory; this is that list.
themes='[{"name":"","dir":"","file":""},{"name":"rose-pine","dir":"/probe/themes/rose-pine","file":"/probe/themes/rose-pine/name"}]'

output=$(env -u DISPLAY -u WAYLAND_DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE \
    HOME="$test_root/home" XDG_STATE_HOME="$test_root/state" XDG_RUNTIME_DIR="$test_root/runtime" \
    FLEA_THEMES="$themes" \
    QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME=generic QT_FORCE_STDERR_LOGGING=1 \
    timeout 30 qs -p "$test_root/config" 2>&1)

# Sample input, one probe line: "  INFO qml: THEMEPICKER PASS listed=2 applied=rose-pine own=none refused=gone"
pass_count=$(printf '%s\n' "$output" | grep -c 'THEMEPICKER PASS')
fail_count=$(printf '%s\n' "$output" | grep -c 'THEMEPICKER FAIL')
noisy=$(printf '%s\n' "$output" | grep -av 'does not support setting window masks' | grep -aiE 'WARN|ERROR|Binding loop|failed to load|Unable to assign|Cannot assign|TypeError|ReferenceError|is not a function|is not defined')
if [ "$pass_count" -ne 1 ] || [ "$fail_count" -ne 0 ] || [ -n "$noisy" ]; then
    printf 'FAIL the theme picker did not read the list, did not apply the theme, or the probe logged a QML error\n'
    printf '%s\n' "$output" | grep -a 'THEMEPICKER'
    printf '%s\n' "$output" | grep -aiE 'ERROR|error'
    printf '%s\n' "$output"
    exit 1
fi
printf '%s\n' "$output" | grep -o 'THEMEPICKER PASS.*'
