#!/usr/bin/env bash
# Installs taskwarrior_connector.py as a macOS LaunchAgent so it runs in the
# background and restarts on crash/login.
set -euo pipefail

label="org.taskwarrior.connector"
plist="$HOME/Library/LaunchAgents/${label}.plist"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
prog="${script_dir}/taskwarrior_connector.py"

# The daemon stores each bookmark's URL in a `url` UDA — required for it to
# be written at all, and for MARSTECH-697's duplicate-URL detection to find
# existing tasks (a filter on an unconfigured UDA silently matches nothing).
# Idempotent: re-running with the same value is a no-op past the first time.
task rc.confirmation=off config uda.url.type string >/dev/null
task rc.confirmation=off config uda.url.label URL >/dev/null

mkdir -p "$HOME/Library/LaunchAgents"

cat > "$plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>${label}</string>
    <key>ProgramArguments</key>
    <array>
      <string>/usr/bin/python3</string>
      <string>${prog}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/taskwarrior-connector.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/taskwarrior-connector.log</string>
  </dict>
</plist>
PLIST

launchctl unload "$plist" 2>/dev/null || true
launchctl load "$plist"
echo "Loaded ${label} — logs at /tmp/taskwarrior-connector.log"
echo "Check it's running: launchctl list | grep ${label}"
