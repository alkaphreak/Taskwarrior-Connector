#!/usr/bin/env bash
# Unloads and removes the taskwarrior_connector LaunchAgent.
set -euo pipefail

label="org.taskwarrior.connector"
plist="$HOME/Library/LaunchAgents/${label}.plist"

launchctl unload "$plist" 2>/dev/null || true
rm -f "$plist"
echo "Removed ${label}"
