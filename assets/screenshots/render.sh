#!/usr/bin/env bash
# Renders the AMO listing screenshots (1280x800 PNG) of the real popup with headless Chrome.
# A stub stands in for the WebExtension API and the connector, so no daemon is needed.
# Usage: assets/screenshots/render.sh   (from the repo root, macOS Google Chrome)
set -euo pipefail

CHROME=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
OUT=$(cd "$(dirname "$0")" && pwd)
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
cp -R firefox/. "$WORK/"

# Stub loaded before projects.js: fake tab, project list, and the connector's answer per state.
cat > "$WORK/stub.js" <<'EOF'
const state = new URLSearchParams(location.search).get("state") || "empty";
window.browser = {
  tabs: { query: async () => [{ title: "Rust Atomics and Locks: Chapter 1, Basics of Rust Concurrency", url: "https://marabos.nl/atomics/basics.html" }] },
  storage: { sync: { get: async () => ({ projects: ["Links", "Unread", "Lectures"] }) } },
};
const answers = {
  saved: { ok: true, project: "Unread", annotated: true },
  duplicate: { ok: true, duplicate: true },
};
window.fetch = async () => ({ json: async () => answers[state] });
const fill = () => {
  document.getElementById("title").value = "Rust Atomics and Locks: Chapter 1, Basics of Rust Concurrency";
  document.getElementById("url").value = "https://marabos.nl/atomics/basics.html";
  document.getElementById("tags").value = "rust concurrency";
  document.getElementById("project").value = "Unread";
  document.getElementById("description").value = "Chapter on threads, Arc and Mutex, before the atomics part";
};
document.addEventListener("DOMContentLoaded", () => setTimeout(() => {
  if (!answers[state]) return;
  fill();
  document.getElementById("submit").click();
  // A success clears the form, as in the real popup: put the values back for the screenshot.
  setTimeout(() => { fill(); document.activeElement.blur(); }, 50);
}, 50));
EOF
sed -i '' 's|<script src="projects.js">|<script src="stub.js"></script><script src="projects.js">|' "$WORK/popup.html"

# Frame: the popup at 1:1, centred on the MarsTech-Skin background.
cat > "$WORK/frame.html" <<'EOF'
<!DOCTYPE html><html><head><style>
html, body { margin: 0; width: 1280px; height: 800px; background: #05070a; overflow: hidden; }
iframe { position: absolute; left: 440px; top: 55px; width: 400px; height: 690px; border: 1px solid #1c2430; background: #0a0e13; }
</style></head><body><iframe id="p"></iframe>
<script>document.getElementById("p").src = "popup.html" + location.search;</script></body></html>
EOF

# The number sets the order on AMO (amo-listing.yml uploads them in name order).
n=0
for s in empty saved duplicate; do
  n=$((n + 1))
  "$CHROME" --headless=new --disable-gpu --hide-scrollbars --allow-file-access-from-files \
    --window-size=1280,800 --virtual-time-budget=3000 \
    --screenshot="$OUT/$n-$s.png" "file://$WORK/frame.html?state=$s" 2>/dev/null
done
ls -l "$OUT"/*.png
