> [!IMPORTANT]
> Unmaintained.
>
> An up-to-date fork maybe found in https://github.com/alkaphreak/Taskwarrior-Connector/

# Send to Taskwarrior — a bookmark manager / "read-later app"

> **Fork notice**: this is a hardened fork of the original (unmaintained since 2022)
> [shenlebantongying/Taskwarrior-Connector](https://github.com/shenlebantongying/Taskwarrior-Connector)
> (GPLv3). The original daemon had three real issues, fixed here:
>
> 1. **Command injection** — the original built a shell string via concatenation and ran it
>    through `os.system(...)`. A page whose `<title>` contained a `"` followed by shell
>    metacharacters could execute arbitrary commands. This fork uses `subprocess.run([...])`
>    with an argument list — no shell is ever invoked.
> 2. **Bound to all interfaces** — the original's `HTTPServer(('', port), ...)` accepted
>    requests from any device on the LAN. This fork binds to `127.0.0.1` only.
> 3. **GET with side effects** — the original triggered `task add` on a plain `GET`, which
>    any page's JS can fire cross-origin without a CORS preflight (CSRF). This fork only
>    accepts `POST` and requires the `Origin` header to start with `moz-extension://`.
>
> On top of the security fixes: duplicate-URL detection (skips re-saving a page you already
> bookmarked), a `TASK_TIMEOUT_SECONDS` guard so a stuck `task` can't hang the daemon, clear
> JSON errors instead of silent failures, bookmarks filed under a fixed `project:Links`, and a
> renamed extension ("Send to Taskwarrior") — distinct from the original's still-live
> [AMO listing](https://addons.mozilla.org/firefox/addon/taskwarrior/) to avoid confusion between
> the two.
>
> Because of fix #3, the extension in `firefox/` sends a `POST` — it is **not** compatible with
> the original AMO-listed "Taskwarrior-Connector" add-on. Install this fork's own build instead
> (see below).

Why? After some research, i just realize that every bookmark managers on this planet suck.

Firefox's default manager is too primitive for power users.

+ Cannot archive a viewed link
+ Don't record when you bookmarked something
+ Plain tree structure (Bookmarks, like documents,can not be classified perfectly, one thing can exist in multiple nodes of a tree. Tagging is superior to collect links)

Taskwarrior is "accidentally" the best bookmark manager on Earth

+ Superb tagging and querying system
+ Plain JSON data storage (super easy git backup)
+ By easily adding and filer tags, users can manage a magnitude more things.
+ See clearly when you added the book mark and
+ Easily add more metadata to a bookmark entry
+ Never delete a bookmark, just mark it `Done` and you can revisits old links in future
+ More features

More importantly, easily combine with other command tools.

# INSTALLATION

1. Install [Taskwarrior](https://taskwarrior.org/)
2. Load the Firefox extension — two options:
   - **Temporary** (quick test, lost on every Firefox restart): `about:debugging#/runtime/this-firefox`
     → "Load Temporary Add-on" → pick `firefox/manifest.json`.
   - **Permanent** (daily-driver): download the signed `.xpi` from the
     [latest release](https://github.com/alkaphreak/Taskwarrior-Connector/releases/latest)
     (built and signed automatically by `.github/workflows/build-xpi.yml` on every `v*` tag) and
     drag it into a Firefox window, or `about:addons` → gear icon → "Install Add-on From File…".
     Survives restarts, no dev-mode flags needed.
   - **From the Firefox Add-ons store**: pending review as **"Send to Taskwarrior"** — once
     approved, installing from [addons.mozilla.org](https://addons.mozilla.org/firefox/) will also
     get automatic updates on future releases, no manual reinstall needed. Link added here once
     live.
3. Run the setup script for your platform (see below) — it configures the `url` UDA
   Taskwarrior entries need (`task config uda.url.type/label`) and starts
   `taskwarrior_connector.py` in the background. Safe to re-run any time.

## For most Linux users-> Systemd setup

Just run `systemd_setup.bash`. It will configure the `url` UDA and let the connector script
run in background forever.

To uninstall, use the `systemd_cleanup.bash` 

## For macOS Users -> Launchd setup

Just run `launchd_setup.bash`. It will configure the `url` UDA and let the connector script
run in background forever.

To uninstall, use the `launchd_cleanup.bash` 

# Usage of Taskwarrior 

TaskWarrior is really powerful, and you should really read the [official documentation](https://taskwarrior.org/docs/) to grasp the ultimate management power.

Here is short list of commonly used commands if you intend to use taskwarrior as bookmark manager:

## List your bookmarks

Every bookmark saved by the extension is filed under a fixed `project:Links`:

```bash
task project:Links list
```

## Open a task's link
```bash
task _get {id}.url | xargs xdg-open # GNU/Linux
task _get {id}.url | xargs open # macOS
```

# Releasing (maintainers)

Two GitHub Actions workflows automate cutting a new version — see `.github/workflows/`:

- **`release.yml`** (`workflow_dispatch`) — bumps `firefox/manifest.json`'s version (`bump`
  input: patch/minor/major, or an exact `version` override), commits, tags `vX.Y.Z`, pushes, and
  dispatches `build-xpi.yml` for that tag:
  ```bash
  gh workflow run release.yml --repo alkaphreak/Taskwarrior-Connector -f bump=patch
  ```
- **`build-xpi.yml`** — lints (`web-ext lint`) and signs (`web-ext sign --channel=unlisted`) the
  extension via the AMO API (`WEB_EXT_API_KEY`/`WEB_EXT_API_SECRET` repo secrets), uploads the
  signed `.xpi` as a build artifact, and attaches it to the GitHub Release on real tag runs.

Manual re-runs of `build-xpi.yml` (`workflow_dispatch`, not a tag push) sign a throwaway
`<version>.<run_number>` build for testing — they don't touch the GitHub Release.

# Development

Please don't hesitate to report bugs and suggest new features.

# LICENSE

GPL3
