# AGENTS.md – Taskwarrior-Connector

Hardened fork of shenlebantongying/Taskwarrior-Connector: a Firefox extension (`firefox/`) that
POSTs the current tab to a local Python server (`taskwarrior_connector.py`), which runs
`task add`.

- **Version**: never bump `firefox/manifest.json` by hand. `release.yml` computes and commits
  it; `build-xpi.yml` suffixes it for non-tag runs (AMO signs a version only once).
- **Lint**: `npx web-ext@10 lint --source-dir=firefox`, the same version as CI.
- **Packaging**: everything under `firefox/` ships in the `.xpi`. Design sources (SVG icon
  masters in `assets/`) stay outside it.
- **Popup contract**: `popup.js` uses the IDs `title`, `url`, `tags`, `submit`, `status`;
  a renamed ID fails silently. Status styling is keyed on `#status[data-state]`, set by
  `setStatus(text, state)`.
- **Connector**: listens on `127.0.0.1:34810`, POST only. Tags are space-separated without
  `+` (the connector adds it); tasks go to `project:Links`.
- Issues are tracked in YouTrack (MARSTECH project), not GitHub Issues.
