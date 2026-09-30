#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const CONNECT_TIMEOUT: Duration = Duration::from_millis(300);
const FIRE_AND_FORGET_BUDGET: Duration = Duration::from_secs(2);
const DECISION_BUDGET: Duration = Duration::from_secs(110);

const DROPPED_FIELDS: &[&str] = &["tool_response", "transcript_path"];
const MAX_FIELD_LEN: usize = 2_000;

/// Linux IPC socket used by Coucou.
///
/// The socket is per-user so another local user cannot accidentally receive
/// Claude Code events from this account.
fn socket_path() -> std::path::PathBuf {
    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(format!(
                "/tmp/coucou-{}",
                std::env::var("USER").unwrap_or_else(|_| "user".into())
            ))
        });

    runtime_dir.join("coucou.sock")
}

fn connect() -> Option<UnixStream> {
    let path = socket_path();
    let deadline = Instant::now() + CONNECT_TIMEOUT;

    loop {
        match UnixStream::connect(&path) {
            Ok(stream) => {
                let _ = stream.set_read_timeout(Some(DECISION_BUDGET));
                let _ = stream.set_write_timeout(Some(CONNECT_TIMEOUT));
                return Some(stream);
            }
            Err(_) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(15));
            }
            Err(_) => return None,
        }
    }
}

fn main() {
    let Some((payload, event)) = read_event() else {
        std::process::exit(0);
    };

    let waits_for_answer = event == "PermissionRequest";
    let budget = if waits_for_answer {
        DECISION_BUDGET
    } else {
        FIRE_AND_FORGET_BUDGET
    };

    let (tx, rx) = mpsc::channel::<Option<String>>();

    std::thread::spawn(move || {
        let _ = tx.send(talk(&payload, waits_for_answer));
    });

    if let Ok(Some(decision)) = rx.recv_timeout(budget) {
        if let Some(json) = decision_json(&decision) {
            let mut out = std::io::stdout();
            let _ = writeln!(out, "{json}");
            let _ = out.flush();
        }
    }

    std::process::exit(0);
}

fn decision_json(decision: &str) -> Option<String> {
    let behavior = match decision.trim() {
        "allow" | "always" => r#"{"behavior":"allow"}"#.to_string(),
        "deny" => r#"{"behavior":"deny","message":"Denied from Coucou"}"#.to_string(),
        _ => return None,
    };

    Some(format!(
        r#"{{"hookSpecificOutput":{{"hookEventName":"PermissionRequest","decision":{behavior}}}}}"#
    ))
}

fn read_event() -> Option<(String, String)> {
    let mut raw = Vec::new();

    if std::io::stdin().read_to_end(&mut raw).is_err() || raw.is_empty() {
        return None;
    }

    if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        raw.drain(..3);
    }

    let mut payload = serde_json::from_slice::<serde_json::Value>(&raw).ok()?;
    let map = payload.as_object_mut()?;

    let arg_event = std::env::args().nth(1).unwrap_or_default();

    let event = map
        .get("hook_event_name")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .unwrap_or(arg_event);

    map.insert(
        "hook_event_name".into(),
        serde_json::Value::String(event.clone()),
    );

    for field in DROPPED_FIELDS {
        map.remove(*field);
    }

    let cwd_missing = map
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(str::is_empty)
        .unwrap_or(true);

    if cwd_missing {
        if let Ok(cwd) = std::env::current_dir() {
            map.insert(
                "cwd".into(),
                serde_json::Value::String(cwd.to_string_lossy().to_string()),
            );
        }
    }

    for (key, var) in [
        ("term_program", "TERM_PROGRAM"),
        ("wt_session", "WT_SESSION"),
        ("term_session_id", "TERM_SESSION_ID"),
        ("vscode_pid", "VSCODE_PID"),
        ("session_pid", "CLAUDE_CODE_SSE_PORT"),
    ] {
        if !map.contains_key(key) {
            let value = std::env::var(var).unwrap_or_default();
            map.insert(key.into(), serde_json::Value::String(value));
        }
    }

    truncate_strings(&mut payload);

    let mut line = payload.to_string();
    line.push('\n');

    Some((line, event))
}

fn truncate_strings(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            if s.len() > MAX_FIELD_LEN {
                let mut end = MAX_FIELD_LEN;

                while end > 0 && !s.is_char_boundary(end) {
                    end -= 1;
                }

                s.truncate(end);
                s.push('…');
            }
        }

        serde_json::Value::Array(items) => {
            items.iter_mut().for_each(truncate_strings)
        }

        serde_json::Value::Object(map) => {
            map.values_mut().for_each(truncate_strings)
        }

        _ => {}
    }
}

fn talk(payload: &str, waits_for_answer: bool) -> Option<String> {
    let mut socket = connect()?;

    if socket.write_all(payload.as_bytes()).is_err() {
        return None;
    }

    let _ = socket.flush();

    if !waits_for_answer {
        return None;
    }

    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];

    loop {
        match socket.read(&mut chunk) {
            Ok(0) => break,

            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);

                if buf.contains(&b'\n') {
                    break;
                }
            }

            Err(_) => break,
        }
    }

    let answer = String::from_utf8_lossy(&buf).trim().to_string();

    (!answer.is_empty()).then_some(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_json_matches_the_documented_shape() {
        assert_eq!(
            decision_json("allow").unwrap(),
            r#"{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}"#
        );

        assert_eq!(
            decision_json("deny").unwrap(),
            r#"{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"deny","message":"Denied from Coucou"}}}"#
        );

        assert!(decision_json("always")
            .unwrap()
            .contains(r#""behavior":"allow""#));
    }

    #[test]
    fn anything_unrecognised_prints_nothing() {
        assert!(decision_json("").is_none());
        assert!(decision_json("maybe").is_none());
        assert!(decision_json(r#"{"permissionDecision":"allow"}"#).is_none());
    }

    #[test]
    fn long_strings_are_cut_on_a_char_boundary() {
        let mut v =
            serde_json::json!({ "tool_input": { "content": "é".repeat(4000) } });

        truncate_strings(&mut v);

        let s = v["tool_input"]["content"].as_str().unwrap();

        assert!(s.len() <= MAX_FIELD_LEN + 4);
        assert!(s.ends_with('…'));
    }
}
