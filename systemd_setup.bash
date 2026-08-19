#!/usr/bin/env bash

prog="taskwarrior_connector"

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
ExecStart=$(dirname "$(realpath "$0")")/${prog}.py

[Install]
WantedBy=default.target"

echo "$__sysdunitfile" > ~/.config/systemd/user/${prog}.service

systemctl --user enable ${prog}.service
systemctl --user start ${prog}.service
systemctl --user status ${prog}.service
