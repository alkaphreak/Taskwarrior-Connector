#!/usr/bin/env python3
"""
Local HTTP daemon that lets the "TaskWarrior Connector" Firefox extension
save the current tab as a Taskwarrior task (task add "<title>" url:"<url>" +tag...).

Hardened vs. the original 2022 version. Fixes:
  1. Command injection: the original built a shell string via concatenation
     and ran it through os.system(). A crafted page <title> (e.g. containing
     a `"` followed by shell metacharacters) could execute arbitrary shell
     commands. This version calls `task` via subprocess with an argument
     list — no shell is ever invoked.
  2. Network exposure: the original bound to '' (all interfaces), so any
     device on the LAN could trigger a task-add request. This version binds
     to 127.0.0.1 only.
  3. Method: GET-with-side-effects is CSRF-able from any page you have open
     (a simple cross-origin GET needs no preflight). This version only
     accepts POST, and requires an Origin header of moz-extension:// —
     rejecting plain-page-triggered requests.
"""

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
import subprocess
import sys
import urllib.parse

HOST = "127.0.0.1"
DEFAULT_PORT = 34810
TASK_TIMEOUT_SECONDS = 10
TASK_PROJECT = "Links"


class TaskError(Exception):
    """Carries the (status, body) an unrecoverable `task` invocation should respond with."""
    def __init__(self, status: int, body: dict):
        self.status = status
        self.body = body


def _run_task(*args: str) -> subprocess.CompletedProcess:
    """Runs `task <args>`, raising TaskError on anything that should become an HTTP error."""
    try:
        result = subprocess.run(["task", *args], capture_output=True, text=True, timeout=TASK_TIMEOUT_SECONDS)
    except FileNotFoundError:
        raise TaskError(500, {"error": "'task' binary not found — is Taskwarrior installed and on PATH?"})
    except subprocess.TimeoutExpired:
        raise TaskError(504, {"error": f"'task' timed out after {TASK_TIMEOUT_SECONDS}s — is it waiting on a sync lock or prompt?"})
    except OSError as e:
        raise TaskError(500, {"error": f"failed to run 'task': {e}"})
    if result.returncode != 0:
        raise TaskError(500, {"error": result.stderr.strip() or "'task' command failed"})
    return result


class Handler(BaseHTTPRequestHandler):
    # BaseHTTPRequestHandler defaults to HTTP/1.0 with no explicit Connection
    # header — the response's end is signaled only by the socket closing.
    # curl tolerates that; Firefox's fetch() has been observed to report a
    # bare "NetworkError" on it instead, especially on rapid successive
    # requests. HTTP/1.1 + an explicit "Connection: close" removes the
    # ambiguity for stricter clients without needing real keep-alive support.
    protocol_version = "HTTP/1.1"

    def _respond(self, status: int, body: dict):
        payload = json.dumps(body).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(payload)

    def do_GET(self):
        # No side-effecting GET — avoids trivial cross-origin CSRF from any
        # open tab. The extension must use POST.
        self._respond(404, {"error": "use POST"})

    def do_POST(self):
        origin = self.headers.get("Origin", "")
        if not origin.startswith("moz-extension://"):
            self._respond(403, {"error": "forbidden origin"})
            return

        length = int(self.headers.get("Content-Length", 0))
        raw = self.rfile.read(length) if length else b""
        try:
            args = urllib.parse.parse_qs(raw.decode("utf-8"))
            title = args["title"][0]
            url = args["url"][0]
            tags = args.get("tag", [""])[0].split()
        except (KeyError, IndexError, UnicodeDecodeError):
            self._respond(400, {"error": "missing title/url"})
            return

        try:
            # Skip tasks already deleted, but count completed ones too — this
            # is a bookmark manager, "Done" means "read", not "gone".
            existing = _run_task(f"url:{url}", "status.not:deleted", "count")
            if existing.stdout.strip() not in ("", "0"):
                self._respond(200, {"ok": True, "duplicate": True, "output": "URL already saved"})
                return

            result = _run_task(*[f"+{t}" for t in tags], "add", title, f"url:{url}", f"project:{TASK_PROJECT}")
        except TaskError as e:
            self._respond(e.status, e.body)
            return
        self._respond(200, {"ok": True, "duplicate": False, "output": result.stdout.strip()})


def run(port: int = DEFAULT_PORT):
    # Homebrew's `task` isn't on the default launchd PATH on macOS.
    os.environ["PATH"] += os.pathsep + "/opt/homebrew/bin" + os.pathsep + "/usr/local/bin"
    httpd = HTTPServer((HOST, port), Handler)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    httpd.server_close()


if __name__ == "__main__":
    run(int(sys.argv[1]) if len(sys.argv) == 2 else DEFAULT_PORT)
