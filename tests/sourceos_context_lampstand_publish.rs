use assert_cmd::Command;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::thread;
use tempfile::tempdir;

fn cargo_bin() -> Command {
    Command::cargo_bin("sourceos-context").expect("sourceos-context binary should build")
}

fn make_home_with_repo() -> (tempfile::TempDir, std::path::PathBuf) {
    let home = tempdir().expect("temp home");
    let repo = home.path().join("dev").join("example");
    fs::create_dir_all(repo.join("src")).expect("repo dirs");
    fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = \"example\"\nversion = \"0.1.0\"\n",
    )
    .expect("Cargo.toml");
    fs::write(repo.join("README.md"), "# Example\n").expect("README");
    fs::write(repo.join("src").join("main.rs"), "fn main() {}\n").expect("main.rs");
    (home, repo)
}

#[test]
fn lampstand_publish_unavailable_fails_closed_when_publish_requested() {
    let (home, repo) = make_home_with_repo();
    let socket_dir = tempdir().expect("socket dir");
    let missing_socket = socket_dir.path().join("missing.sock");

    let output = cargo_bin()
        .env("HOME", home.path())
        .args([
            "lampstand-publish",
            repo.to_str().unwrap(),
            "--publish",
            "--format",
            "json",
            "--socket",
            missing_socket.to_str().unwrap(),
        ])
        .assert()
        .code(2)
        .get_output()
        .stdout
        .clone();

    let value: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(value["schema_version"], "sourceos.adapter_error.v1");
    assert_eq!(value["error_code"], "lampstand_unavailable");
    assert_eq!(value["safe_retry"], true);
}

#[test]
fn lampstand_publish_sends_records_to_unixjson_when_publish_requested() {
    let (home, repo) = make_home_with_repo();
    let socket_dir = tempdir().expect("socket dir");
    let socket = socket_dir.path().join("lampstand.sock");
    let listener = UnixListener::bind(&socket).expect("bind fake Lampstand socket");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fake publish request");
        let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
        let mut request = String::new();
        reader.read_line(&mut request).expect("read request");
        let value: Value = serde_json::from_str(&request).expect("json request");
        assert_eq!(value["method"], "PublishAdapterRecords");
        assert_eq!(value["params"]["dry_run"], false);
        let records = value["params"]["records"].as_array().expect("records array");
        assert!(records.len() >= 2, "expected repo context and structure records");
        assert_eq!(records[0]["object_kind"], "repo_context");
        assert_eq!(records[0]["classification"], "local_only");

        let response = json!({
            "ok": true,
            "result": {
                "accepted": records.len(),
                "published": records.len(),
                "record_ids": ["record-1", "record-2"]
            }
        });
        stream.write_all(response.to_string().as_bytes()).expect("write response");
        stream.write_all(b"\n").expect("write newline");
    });

    let output = cargo_bin()
        .env("HOME", home.path())
        .args([
            "lampstand-publish",
            repo.to_str().unwrap(),
            "--publish",
            "--format",
            "json",
            "--socket",
            socket.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    handle.join().expect("fake Lampstand server thread");

    let value: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(value["response_type"], "LampstandPublishReport");
    assert_eq!(value["data"]["dry_run"], false);
    assert!(value["data"]["accepted_count"].as_u64().unwrap() >= 2);
    assert!(value["data"]["published_count"].as_u64().unwrap() >= 2);
    assert_eq!(value["data"]["record_ids"].as_array().unwrap().len(), 2);
}
