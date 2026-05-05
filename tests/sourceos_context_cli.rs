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

fn assert_policy_denied(output: Vec<u8>) {
    // Every denied path must fail closed with the structured SourceOS adapter error envelope.
    let value: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(value["schema_version"], "sourceos.adapter_error.v1");
    assert_eq!(value["error_code"], "policy_denied");
    assert_eq!(value["policy_decision"]["decision"], "deny");
}

#[test]
fn snapshot_allows_repo_under_home_dev() {
    let (home, repo) = make_home_with_repo();

    let output = cargo_bin()
        .env("HOME", home.path())
        .args(["snapshot", repo.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(value["schema_version"], "sourceos.adapter_response.v1");
    assert_eq!(value["response_type"], "RepoContextSnapshot");
    assert_eq!(value["policy_profile"], "sourceos.repo_context.read_only");
    assert_eq!(
        value["data"]["schema_version"],
        "sourceos.repo_context_snapshot.v1"
    );
    assert_eq!(value["data"]["repo_identity"]["name"], "example");
}

#[test]
fn snapshot_denies_repo_outside_home_dev() {
    let home = tempdir().expect("temp home");
    let outside = tempdir().expect("outside root");
    fs::write(outside.path().join("README.md"), "# Outside\n").expect("outside README");

    let output = cargo_bin()
        .env("HOME", home.path())
        .args(["snapshot", outside.path().to_str().unwrap(), "--format", "json"])
        .assert()
        .code(2)
        .get_output()
        .stdout
        .clone();

    assert_policy_denied(output);
}

#[test]
fn snapshot_denies_unbounded_home_root() {
    let (home, _) = make_home_with_repo();

    let output = cargo_bin()
        .env("HOME", home.path())
        .args(["snapshot", home.path().to_str().unwrap(), "--format", "json"])
        .assert()
        .code(2)
        .get_output()
        .stdout
        .clone();

    assert_policy_denied(output);
}

#[test]
fn snapshot_denies_symlink_root_even_if_target_is_under_home_dev() {
    let (home, repo) = make_home_with_repo();
    let link = home.path().join("dev").join("repo-link");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&repo, &link).expect("symlink");

    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&repo, &link).expect("symlink");

    let output = cargo_bin()
        .env("HOME", home.path())
        .args(["snapshot", link.to_str().unwrap(), "--format", "json"])
        .assert()
        .code(2)
        .get_output()
        .stdout
        .clone();

    assert_policy_denied(output);
}

#[test]
fn lampstand_publish_is_dry_run_only_and_returns_records() {
    let (home, repo) = make_home_with_repo();

    let output = cargo_bin()
        .env("HOME", home.path())
        .args([
            "lampstand-publish",
            repo.to_str().unwrap(),
            "--dry-run",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(value["response_type"], "LampstandPublishReport");
    assert_eq!(value["data"]["dry_run"], true);
    assert_eq!(value["data"]["published_count"], 0);
    assert!(value["data"]["records"].as_array().expect("records array").len() >= 2);
}

#[test]
fn lampstand_roots_unavailable_fails_closed() {
    let socket_dir = tempdir().expect("socket dir");
    let missing_socket = socket_dir.path().join("missing.sock");

    let output = cargo_bin()
        .args([
            "lampstand-roots",
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
fn lampstand_roots_consumes_unixjson_root_hints() {
    let socket_dir = tempdir().expect("socket dir");
    let socket = socket_dir.path().join("lampstand.sock");
    let listener = UnixListener::bind(&socket).expect("bind fake lampstand socket");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fake root hints request");
        let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
        let mut request = String::new();
        reader.read_line(&mut request).expect("read request");
        let value: Value = serde_json::from_str(&request).expect("json request");
        assert_eq!(value["method"], "RootHints");

        let response = json!({
            "ok": true,
            "result": {
                "adapter_mode": "rpc",
                "roots": [
                    {
                        "source_root_id": "lampstand-root::sha256:abc",
                        "path": "/tmp/example-root",
                        "root_kind": "local_root",
                        "freshness": null,
                        "classification": "local_only",
                        "handling_tags": ["local-only", "lampstand-root"]
                    }
                ]
            }
        });
        stream.write_all(response.to_string().as_bytes()).expect("write response");
        stream.write_all(b"\n").expect("write newline");
    });

    let output = cargo_bin()
        .args([
            "lampstand-roots",
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

    handle.join().expect("fake lampstand server thread");

    let value: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(value["response_type"], "LampstandRootSet");
    assert_eq!(value["data"]["adapter_mode"], "rpc");
    let roots = value["data"]["roots"].as_array().expect("roots array");
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0]["source_root_id"], "lampstand-root::sha256:abc");
    assert_eq!(roots[0]["path_ref"], "/tmp/example-root");
    assert_eq!(roots[0]["classification"], "local_only");
}

#[test]
fn security_redacts_matched_text() {
    let (home, repo) = make_home_with_repo();
    fs::write(
        repo.join("settings.json"),
        r#"{"hooks":{"PreToolUse":["npx claude-flow@alpha swarm init"]}}"#,
    )
    .expect("settings.json");

    let output = cargo_bin()
        .env("HOME", home.path())
        .args(["security", repo.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(value["response_type"], "SecuritySignalSet");
    let signals = value["data"]["signals"].as_array().expect("signals array");
    assert!(!signals.is_empty(), "expected at least one security signal");
    let redacted = signals[0]["matched_text_redacted"].as_str().unwrap_or_default();
    assert!(redacted.starts_with("[redacted:"), "matched text must be redacted");
}
