# Taskwarrior as a bookmark manager / "read-later app"

> **Fork notice**: this is a hardened fork of the original (unmaintained since 2022)
> [shenlebantongying/Taskwarrior-Connector](https://github.com/shenlebantongying/Taskwarrior-Connector).
> The original daemon had three real issues, fixed here:
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
> Because of fix #3, the extension in `firefox/` now sends a `POST` — it is **not** compatible
> with the original AMO-listed "Taskwarrior" add-on. Load `firefox/` as a temporary add-on
> (see below) instead of installing from addons.mozilla.org.

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
2. Load the Firefox extension: `about:debugging#/runtime/this-firefox` → "Load Temporary
   Add-on" → pick `firefox/manifest.json`. (Temporary add-ons are removed on Firefox restart —
   reload after each restart, or package as a signed `.xpi` for daily-driver use.)
3. Let the `taskwarrior_connector.py` to run in background. (Use terminal or use a service manager -> See below)
4. Add extra attribute `url` for taskwarrior entries 

```bash
yes | task config uda.url.type string
yes | task config uda.url.label URL
```

## For most Linux users-> Systemd setup

Just run `systemd_setup.bash`. It will let the connector script run in background forever.

To uninstall, use the `systemd_cleanup.bash` 

## For macOS Users -> Launchd setup

Just run `launchd_setup.bash`. It will let the connector script run in background forever.

To uninstall, use the `launchd_cleanup.bash` 

# Usage of Taskwarrior 

TaskWarrior is really powerful, and you should really read the [official documentation](https://taskwarrior.org/docs/) to grasp the ultimate management power.

Here is short list of commonly used commands if you intend to use taskwarrior as bookmark manager:

## Open a task's link
```bash
task _get {id}.url | xargs xdg-open # GNU/Linux
task _get {id}.url | xargs open # macOS
```

# Development

Please don't hesitate to report bugs and suggest new features.

# LICENSE

GPL3
