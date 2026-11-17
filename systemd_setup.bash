#!/usr/bin/env bash
# Installs the taskwarrior-connector binary as a systemd user service.
set -euo pipefail

unit="taskwarrior_connector"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# The connector binary: installed by Homebrew or the release's shell installer (on PATH),
# else a local `cargo build --release` of this repo.
prog="$(command -v taskwarrior-connector || true)"
[ -n "$prog" ] || prog="${script_dir}/target/release/taskwarrior-connector"
if [ ! -x "$prog" ]; then
  echo "taskwarrior-connector not found: brew install alkaphreak/tap/taskwarrior-connector, or cargo build --release" >&2
  exit 1
fi

# The daemon stores each bookmark's URL in a `url` UDA — required for it to
# be written at all, and for MARSTECH-697's duplicate-URL detection to find
# existing tasks (a filter on an unconfigured UDA silently matches nothing).
# Idempotent: re-running with the same value is a no-op past the first time.
task rc.confirmation=off config uda.url.type string >/dev/null
task rc.confirmation=off config uda.url.label URL >/dev/null

__sysdunitfile="[Unit]
Description=TaskWarrior Connector

[Service]
Type=simple
StandardOutput=journal
ExecStart=\"${prog}\"

[Install]
WantedBy=default.target"

mkdir -p ~/.config/systemd/user
echo "$__sysdunitfile" > ~/.config/systemd/user/${unit}.service

# daemon-reload + restart so a re-run picks up a changed unit or an upgraded binary.
systemctl --user daemon-reload
systemctl --user enable ${unit}.service
systemctl --user restart ${unit}.service
systemctl --user --no-pager status ${unit}.service
