//! Local HTTP daemon behind the "Send to Taskwarrior Hardened" Firefox extension: saves the
//! POSTed tab as a Taskwarrior task (`task +tag... add "<title>" url:<url> project:Links`).
//!
//! Hardened vs. the original 2022 Python version of shenlebantongying/Taskwarrior-Connector:
//!   1. No shell: `task` gets an argument list, so a crafted page title cannot inject commands.
//!   2. Binds 127.0.0.1 only, never the LAN.
//!   3. POST only, and only from an `Origin: moz-extension://...`: a side-effecting GET, or a
//!      POST from an ordinary web page, would let any open tab add tasks (CSRF).

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, thread};

use serde_json::{Value, json};
use tiny_http::{Header, Method, Request, Response, Server};
use wait_timeout::ChildExt;

const HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 34810;
const TASK_TIMEOUT: Duration = Duration::from_secs(10);
const SYNC_TIMEOUT: Duration = Duration::from_secs(15);
const TASK_PROJECT: &str = "Links";
const MAX_BODY_BYTES: u64 = 1 << 20;

/// An unrecoverable `task` invocation, carrying the HTTP status it should answer with.
#[derive(Debug, PartialEq)]
struct TaskError {
    status: u16,
    error: String,
}

impl TaskError {
    fn new(status: u16, error: impl Into<String>) -> Self {
        Self {
            status,
            error: error.into(),
        }
    }
}

/// Runs `<program> <args>` and returns its stdout, mapping every failure to an HTTP error.
fn run_task(
    program: &str,
    path: &str,
    args: &[String],
    timeout: Duration,
) -> Result<String, TaskError> {
    let mut child = Command::new(program)
        .args(args)
        .env("PATH", path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => TaskError::new(
                500,
                "'task' binary not found — is Taskwarrior installed and on PATH?",
            ),
            _ => TaskError::new(500, format!("failed to run 'task': {e}")),
        })?;
    // ponytail: pipes are read after exit; fine for `task add`/`count` output (a few lines),
    // a reader thread per pipe if a command ever prints more than the 64 KB pipe buffer.
    let status = match child.wait_timeout(timeout) {
        Ok(Some(status)) => status,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(TaskError::new(
                504,
                format!(
                    "'task' timed out after {}s — is it waiting on a sync lock or prompt?",
                    timeout.as_secs()
                ),
            ));
        }
        Err(e) => return Err(TaskError::new(500, format!("failed to run 'task': {e}"))),
    };
    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_string(&mut stderr);
    }
    if !status.success() {
        let stderr = stderr.trim();
        return Err(TaskError::new(
            500,
            if stderr.is_empty() {
                "'task' command failed"
            } else {
                stderr
            },
        ));
    }
    Ok(stdout)
}

/// What a request turns into: the HTTP answer, and whether a task was added (triggers a sync).
struct Outcome {
    status: u16,
    body: Value,
    saved: bool,
}

fn outcome(status: u16, body: Value) -> Outcome {
    Outcome {
        status,
        body,
        saved: false,
    }
}

/// Request logic, independent of the HTTP library so it can be tested with a fake `task`.
fn handle(
    method: &Method,
    origin: Option<&str>,
    body: &[u8],
    task: &dyn Fn(&[String]) -> Result<String, TaskError>,
) -> Outcome {
    match method {
        Method::Post => {}
        Method::Get => return outcome(404, json!({"error": "use POST"})),
        _ => return outcome(501, json!({"error": "unsupported method"})),
    }
    if !origin.is_some_and(|o| o.starts_with("moz-extension://")) {
        return outcome(403, json!({"error": "forbidden origin"}));
    }

    let Ok(body) = std::str::from_utf8(body) else {
        return outcome(400, json!({"error": "missing title/url"}));
    };
    // Like Python's parse_qs: first value per key, blank values dropped.
    let field = |name: &str| {
        form_urlencoded::parse(body.as_bytes())
            .find(|(k, v)| k == name && !v.is_empty())
            .map(|(_, v)| v.into_owned())
    };
    let (Some(title), Some(url)) = (field("title"), field("url")) else {
        return outcome(400, json!({"error": "missing title/url"}));
    };
    let tags = field("tag").unwrap_or_default();

    let result = (|| {
        // Count completed tasks too: this is a bookmark manager, "Done" means "read", not "gone".
        let existing = task(&[
            format!("url:{url}"),
            "status.not:deleted".into(),
            "count".into(),
        ])?;
        if !matches!(existing.trim(), "" | "0") {
            return Ok(None);
        }
        let mut args: Vec<String> = tags.split_whitespace().map(|t| format!("+{t}")).collect();
        args.extend([
            "add".into(),
            title,
            format!("url:{url}"),
            format!("project:{TASK_PROJECT}"),
        ]);
        task(&args).map(Some)
    })();

    match result {
        Ok(None) => outcome(
            200,
            json!({"ok": true, "duplicate": true, "output": "URL already saved"}),
        ),
        Ok(Some(out)) => Outcome {
            status: 200,
            body: json!({"ok": true, "duplicate": false, "output": out.trim()}),
            saved: true,
        },
        Err(e) => outcome(e.status, json!({"error": e.error})),
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("static header")
}

fn respond(mut request: Request, path: &str, task_bin: &str) {
    let origin = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Origin"))
        .map(|h| h.value.as_str().to_owned());
    let mut body = Vec::new();
    let read_ok = request
        .as_reader()
        .take(MAX_BODY_BYTES + 1)
        .read_to_end(&mut body)
        .is_ok();

    let run = |args: &[String]| run_task(task_bin, path, args, TASK_TIMEOUT);
    let out = if !read_ok {
        outcome(400, json!({"error": "unreadable body"}))
    } else if body.len() as u64 > MAX_BODY_BYTES {
        outcome(413, json!({"error": "request body too large"}))
    } else {
        handle(request.method(), origin.as_deref(), &body, &run)
    };

    if out.saved {
        // Best effort: the task is on disk either way, sync only pushes it to other devices sooner.
        let (path, task_bin) = (path.to_owned(), task_bin.to_owned());
        thread::spawn(move || {
            let _ = run_task(&task_bin, &path, &["sync".into()], SYNC_TIMEOUT);
        });
    }

    // HTTP/1.1 with a Content-Length: Firefox's fetch() reported a bare NetworkError on responses
    // whose end was only signaled by the socket closing (MARSTECH-711). tiny_http manages the
    // Connection header itself (keep-alive), so the length is what marks the end.
    let response = Response::from_string(out.body.to_string())
        .with_status_code(out.status)
        .with_header(header("Content-Type", "application/json"));
    let _ = request.respond(response);
}

/// Serves requests one at a time, like the Python HTTPServer it replaces.
fn serve(server: &Server, path: &str, task_bin: &str) {
    for request in server.incoming_requests() {
        respond(request, path, task_bin);
    }
}

/// Homebrew's `task` is not on launchd's default PATH on macOS.
fn task_path() -> String {
    let base = env::var("PATH").unwrap_or_default();
    format!("{base}:/opt/homebrew/bin:/usr/local/bin")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = match args.as_slice() {
        [_, port] => port.parse().unwrap_or_else(|_| {
            eprintln!("usage: taskwarrior-connector [port]   (default {DEFAULT_PORT})");
            std::process::exit(2);
        }),
        _ => DEFAULT_PORT,
    };
    let server = Server::http((HOST, port)).unwrap_or_else(|e| {
        eprintln!("cannot listen on {HOST}:{port}: {e}");
        std::process::exit(1);
    });
    serve(&server, &task_path(), "task");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io::Write;
    use std::net::TcpStream;

    const EXT: Option<&str> = Some("moz-extension://abc");

    /// A fake `task`: answers `count` with `count`, anything else as a successful add.
    fn fake(count: &'static str) -> impl Fn(&[String]) -> Result<String, TaskError> {
        move |args: &[String]| {
            Ok(if args.last().is_some_and(|a| a == "count") {
                count.into()
            } else {
                "Created task 7.".into()
            })
        }
    }

    #[test]
    fn rejects_get_other_methods_and_foreign_origins() {
        let task = fake("0");
        assert_eq!(handle(&Method::Get, EXT, b"", &task).status, 404);
        assert_eq!(handle(&Method::Put, EXT, b"", &task).status, 501);
        assert_eq!(
            handle(&Method::Post, None, b"title=a&url=b", &task).status,
            403
        );
        assert_eq!(
            handle(
                &Method::Post,
                Some("https://evil.example"),
                b"title=a&url=b",
                &task
            )
            .status,
            403
        );
    }

    #[test]
    fn requires_title_and_url() {
        let task = fake("0");
        assert_eq!(handle(&Method::Post, EXT, b"title=a", &task).status, 400);
        assert_eq!(
            handle(&Method::Post, EXT, b"title=&url=b", &task).status,
            400
        );
        assert_eq!(handle(&Method::Post, EXT, b"\xff", &task).status, 400);
    }

    #[test]
    fn adds_task_with_tags_project_and_url() {
        let seen = RefCell::new(Vec::new());
        let task = |args: &[String]| {
            seen.borrow_mut().push(args.to_vec());
            Ok(if args.last().is_some_and(|a| a == "count") {
                "0\n".into()
            } else {
                "Created task 7.\n".into()
            })
        };
        let out = handle(
            &Method::Post,
            EXT,
            b"title=Hi+%22there%22%3B+rm&url=https%3A%2F%2Fx.io%2F%3Fa%3D1&tag=read++rust",
            &task,
        );
        assert_eq!(out.status, 200);
        assert!(out.saved);
        assert_eq!(
            out.body,
            json!({"ok": true, "duplicate": false, "output": "Created task 7."})
        );
        let seen = seen.borrow();
        assert_eq!(
            seen[0],
            ["url:https://x.io/?a=1", "status.not:deleted", "count"]
        );
        assert_eq!(
            seen[1],
            [
                "+read",
                "+rust",
                "add",
                "Hi \"there\"; rm",
                "url:https://x.io/?a=1",
                "project:Links"
            ]
        );
    }

    #[test]
    fn reports_duplicates_without_adding() {
        let task = fake("2\n");
        let out = handle(&Method::Post, EXT, b"title=a&url=b", &task);
        assert_eq!(out.status, 200);
        assert!(!out.saved);
        assert_eq!(out.body["duplicate"], true);
    }

    #[test]
    fn propagates_task_errors() {
        let task = |_: &[String]| Err(TaskError::new(504, "slow"));
        let out = handle(&Method::Post, EXT, b"title=a&url=b", &task);
        assert_eq!((out.status, out.body), (504, json!({"error": "slow"})));
    }

    fn script(name: &str, body: &str) -> String {
        let dir = env::temp_dir().join(format!("twc-test-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("task");
        std::fs::write(&file, format!("#!/bin/sh\n{body}\n")).unwrap();
        Command::new("chmod").arg("+x").arg(&file).status().unwrap();
        file.to_string_lossy().into_owned()
    }

    #[test]
    fn run_task_maps_process_failures() {
        let path = task_path();
        let none: &[String] = &[];
        assert_eq!(
            run_task("/nonexistent/task", &path, none, TASK_TIMEOUT)
                .unwrap_err()
                .status,
            500
        );
        let failing = script("fail", "echo 'bad filter' >&2; exit 1");
        assert_eq!(
            run_task(&failing, &path, none, TASK_TIMEOUT),
            Err(TaskError::new(500, "bad filter"))
        );
        let slow = script("slow", "sleep 5");
        assert_eq!(
            run_task(&slow, &path, none, Duration::from_millis(200))
                .unwrap_err()
                .status,
            504
        );
        let ok = script("ok", "echo \"$@\"");
        assert_eq!(
            run_task(&ok, &path, &["add".into(), "a b".into()], TASK_TIMEOUT).unwrap(),
            "add a b\n"
        );
    }

    #[test]
    fn http_response_carries_json_and_content_length() {
        let server = Server::http("127.0.0.1:0").unwrap();
        let port = server.server_addr().to_ip().unwrap().port();
        let task_bin = script(
            "http",
            "if [ \"$3\" = count ]; then echo 0; else echo 'Created task 1.'; fi",
        );
        let path = task_path();
        thread::spawn(move || {
            let request = server.recv().unwrap();
            respond(request, &path, &task_bin);
        });
        let body = "title=a&url=b";
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            stream,
            "POST / HTTP/1.1\r\nHost: x\r\nConnection: close\r\nOrigin: moz-extension://abc\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let lower = response.to_ascii_lowercase();
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(lower.contains("content-length: 56"), "{response}");
        assert!(
            lower.contains("content-type: application/json"),
            "{response}"
        );
        assert!(
            response.ends_with(r#"{"duplicate":false,"ok":true,"output":"Created task 1."}"#),
            "{response}"
        );
    }
}
