use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use st::security_scan::{RiskLevel, SecurityFinding, SecurityScanner};
use st::{FileNode, Scanner, ScannerConfig, TreeStats};

const POLICY_PROFILE: &str = "sourceos.repo_context.read_only";
const ADAPTER_NAME: &str = "sourceos-smart-tree-adapter";
const TOOL_REPO: &str = "SocioProphet/smart-tree";
const UPSTREAM_REPO: &str = "8b-is/smart-tree";

#[derive(Debug, Parser)]
#[command(
    name = "sourceos-context",
    about = "Policy-gated SourceOS/SocioProphet adapter for Smart Tree repo context",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Produce a bounded, read-only repository context snapshot.
    Snapshot {
        /// Repository or project root to scan.
        repo: PathBuf,
        /// Output format. Only json is currently supported.
        #[arg(long, default_value = "json")]
        format: String,
        /// Maximum traversal depth.
        #[arg(long, default_value_t = 5)]
        max_depth: usize,
    },
    /// Run advisory security scan for an approved repository root.
    Security {
        /// Repository or project root to scan.
        repo: PathBuf,
        /// Output format. Only json is currently supported.
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// Dry-run mapping of repo context into Lampstand local search records.
    LampstandPublish {
        /// Repository or project root to publish.
        repo: PathBuf,
        /// Do not write to Lampstand. Required in this initial implementation.
        #[arg(long, default_value_t = true)]
        dry_run: bool,
        /// Output format. Only json is currently supported.
        #[arg(long, default_value = "json")]
        format: String,
        /// Maximum traversal depth used for the source snapshot.
        #[arg(long, default_value_t = 5)]
        max_depth: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Snapshot {
            repo,
            format,
            max_depth,
        } => run_snapshot(repo, format, max_depth),
        Command::Security { repo, format } => run_security(repo, format),
        Command::LampstandPublish {
            repo,
            dry_run,
            format,
            max_depth,
        } => run_lampstand_publish(repo, dry_run, format, max_depth),
    };

    match result {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value).unwrap());
        }
        Err(err) => {
            let value = adapter_error("scan_failed", &err.to_string(), true);
            println!("{}", serde_json::to_string_pretty(&value).unwrap());
            std::process::exit(1);
        }
    }
}

fn run_snapshot(repo: PathBuf, format: String, max_depth: usize) -> Result<Value> {
    ensure_json(&format)?;
    let approved = approve_repo_root(&repo)?;
    let snapshot = build_snapshot(&approved, max_depth)?;
    Ok(adapter_response("RepoContextSnapshot", snapshot, vec!["repo.tree.read", "repo.stats.read", "repo.git_status.read", "repo.security_scan.read"]))
}

fn run_security(repo: PathBuf, format: String) -> Result<Value> {
    ensure_json(&format)?;
    let approved = approve_repo_root(&repo)?;
    let scanner = SecurityScanner::new();
    let findings = scanner.scan_directory(&approved.canonical_path)?;
    let signals = findings
        .iter()
        .enumerate()
        .map(|(idx, finding)| security_signal(idx, finding, &approved))
        .collect::<Vec<_>>();

    let data = json!({
        "schema_version": "sourceos.security_signal_set.v1",
        "signals": signals
    });

    Ok(adapter_response("SecuritySignalSet", data, vec!["repo.security_scan.read"]))
}

fn run_lampstand_publish(
    repo: PathBuf,
    dry_run: bool,
    format: String,
    max_depth: usize,
) -> Result<Value> {
    ensure_json(&format)?;
    if !dry_run {
        return Ok(adapter_error(
            "policy_denied",
            "Lampstand writes are denied in the initial Smart Tree adapter profile. Use --dry-run.",
            false,
        ));
    }

    let approved = approve_repo_root(&repo)?;
    let snapshot = build_snapshot(&approved, max_depth)?;
    let records = lampstand_records_from_snapshot(&snapshot, &approved);

    let data = json!({
        "schema_version": "sourceos.lampstand_publish_report.v1",
        "dry_run": true,
        "records": records,
        "published_count": 0
    });

    Ok(adapter_response(
        "LampstandPublishReport",
        data,
        vec!["lampstand.search_record.publish.local", "repo.tree.read", "repo.stats.read"],
    ))
}

fn ensure_json(format: &str) -> Result<()> {
    if format == "json" {
        Ok(())
    } else {
        Err(anyhow!("unsupported format '{}'; only json is supported", format))
    }
}

#[derive(Debug, Clone)]
struct ApprovedRoot {
    canonical_path: PathBuf,
    path_ref: String,
    repo_name: String,
}

fn approve_repo_root(path: &Path) -> Result<ApprovedRoot> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("could not resolve home directory"))?;
    let allowed_dev = home.join("dev");

    if is_symlink_path(path) {
        return Err(anyhow!("policy denied: symlink root traversal is not allowed"));
    }

    let canonical = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize repo root {}", path.display()))?;
    let allowed_dev = allowed_dev
        .canonicalize()
        .with_context(|| format!("failed to canonicalize allowed root {}", allowed_dev.display()))?;

    if !canonical.starts_with(&allowed_dev) {
        return Err(anyhow!(
            "policy denied: {} is outside approved root ~/dev/**",
            canonical.display()
        ));
    }

    if is_system_or_sensitive_path(&canonical, &home) {
        return Err(anyhow!("policy denied: sensitive/system path is not allowed"));
    }

    let repo_name = canonical
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("repo")
        .to_string();

    Ok(ApprovedRoot {
        path_ref: path_ref(&canonical),
        canonical_path: canonical,
        repo_name,
    })
}

fn is_symlink_path(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn is_system_or_sensitive_path(path: &Path, home: &Path) -> bool {
    let denied = [
        PathBuf::from("/"),
        PathBuf::from("/etc"),
        PathBuf::from("/proc"),
        PathBuf::from("/sys"),
        PathBuf::from("/dev"),
        PathBuf::from("/run"),
        PathBuf::from("/tmp"),
        home.join(".ssh"),
        home.join(".gnupg"),
        home.join(".aws"),
    ];

    denied.iter().any(|denied_path| path == denied_path || path.starts_with(denied_path))
}

fn build_snapshot(approved: &ApprovedRoot, max_depth: usize) -> Result<Value> {
    let config = ScannerConfig {
        max_depth,
        follow_symlinks: false,
        respect_gitignore: true,
        show_hidden: false,
        show_ignored: false,
        find_pattern: None,
        file_type_filter: None,
        entry_type_filter: None,
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
        use_default_ignores: true,
        search_keyword: None,
        show_filesystems: false,
        sort_field: None,
        top_n: None,
        include_line_content: false,
        compute_interest: true,
        security_scan: true,
        min_interest: 0.0,
        track_traversal: true,
        changes_only: false,
        compare_state: None,
        smart_mode: true,
    };

    let scanner = Scanner::new(&approved.canonical_path, config)?;
    let (nodes, stats) = scanner.scan()?;

    let key_files = key_files(&nodes, approved);
    let interesting_files = interesting_files(&nodes, approved);
    let security_signals = security_signals_from_nodes(&nodes, approved);
    let languages = language_summary(&nodes);
    let branch = nodes.iter().find_map(|node| node.git_branch.clone());

    let memory_candidates = vec![json!({
        "candidate_id": stable_id(&format!("repo_onboarding:{}", approved.path_ref)),
        "candidate_type": "repo_onboarding",
        "confidence": 0.75,
        "content": format!("Repo onboarding candidate for {} generated from a bounded Smart Tree scan.", approved.repo_name),
        "tags": ["repo-onboarding", "smart-tree", "sourceos"],
        "source_refs": [approved.path_ref.clone()],
        "policy_labels": [POLICY_PROFILE],
        "recommended_action": "review"
    })];

    Ok(json!({
        "schema_version": "sourceos.repo_context_snapshot.v1",
        "repo_path_ref": approved.path_ref,
        "repo_identity": {
            "name": approved.repo_name,
            "git_remote": Value::Null,
            "branch": branch,
            "commit": Value::Null
        },
        "lampstand": {
            "source_root_id": Value::Null,
            "local_state_record_ids": [],
            "freshness": Value::Null,
            "publishable_records": []
        },
        "summary": {
            "project_type": project_types(&nodes),
            "languages": languages,
            "frameworks": [],
            "build_systems": build_systems(&nodes),
            "test_systems": test_systems(&nodes)
        },
        "stats": stats_json(&stats),
        "key_files": key_files,
        "interesting_files": interesting_files,
        "git": {
            "branch": branch,
            "remote": Value::Null,
            "commit": Value::Null
        },
        "security_signals": security_signals,
        "symbol_summary": {},
        "memory_candidates": memory_candidates
    }))
}

fn key_files(nodes: &[FileNode], approved: &ApprovedRoot) -> Vec<Value> {
    const KEY_NAMES: &[&str] = &[
        "README.md",
        "LICENSE",
        "Cargo.toml",
        "Cargo.lock",
        "package.json",
        "pyproject.toml",
        "requirements.txt",
        "go.mod",
        "Makefile",
        "Dockerfile",
        "docker-compose.yml",
        "AGENTS.md",
        "CLAUDE.md",
    ];

    nodes
        .iter()
        .filter(|node| !node.is_dir)
        .filter(|node| {
            node.path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| KEY_NAMES.contains(&name))
                .unwrap_or(false)
        })
        .take(50)
        .map(|node| file_observation(node, approved))
        .collect()
}

fn interesting_files(nodes: &[FileNode], approved: &ApprovedRoot) -> Vec<Value> {
    let mut files = nodes
        .iter()
        .filter(|node| !node.is_dir)
        .filter(|node| !node.is_ignored)
        .collect::<Vec<_>>();

    files.sort_by(|a, b| b.size.cmp(&a.size));

    files
        .into_iter()
        .take(25)
        .map(|node| file_observation(node, approved))
        .collect()
}

fn security_signals_from_nodes(nodes: &[FileNode], approved: &ApprovedRoot) -> Vec<Value> {
    let mut idx = 0usize;
    let mut signals = Vec::new();

    for node in nodes {
        for finding in &node.security_findings {
            signals.push(security_signal_with_path(idx, finding, approved, &node.path));
            idx += 1;
        }
    }

    signals
}

fn file_observation(node: &FileNode, approved: &ApprovedRoot) -> Value {
    json!({
        "path_ref": relative_path_ref(&node.path, &approved.canonical_path),
        "object_kind": if node.is_dir { "directory" } else if node.is_symlink { "symlink" } else { "file" },
        "category": format!("{:?}", node.category).to_lowercase(),
        "size_bytes": node.size,
        "mtime": system_time_json(node.modified),
        "content_hash": node.content_hash,
        "metadata_hash": Value::Null,
        "is_hidden": node.is_hidden,
        "is_ignored": node.is_ignored,
        "interest_score": Value::Null,
        "change_status": node.change_status.as_ref().map(|status| format!("{:?}", status)),
        "security_signal_ids": []
    })
}

fn security_signal(idx: usize, finding: &SecurityFinding, approved: &ApprovedRoot) -> Value {
    security_signal_with_path(idx, finding, approved, &finding.file_path)
}

fn security_signal_with_path(
    idx: usize,
    finding: &SecurityFinding,
    approved: &ApprovedRoot,
    file_path: &Path,
) -> Value {
    let path_ref = relative_path_ref(file_path, &approved.canonical_path);
    json!({
        "signal_id": stable_id(&format!("{}:{}:{}", path_ref, finding.line_number, idx)),
        "path_ref": path_ref,
        "line": finding.line_number,
        "pattern_name": finding.pattern_name,
        "risk_level": risk_level(&finding.risk_level),
        "description": finding.description,
        "matched_text_redacted": redacted_match(&finding.matched_text),
        "context_kind": context_kind(file_path),
        "policy_recommendation": policy_recommendation(&finding.risk_level),
        "lampstand_record_id": Value::Null,
        "memory_candidate_id": Value::Null
    })
}

fn lampstand_records_from_snapshot(snapshot: &Value, approved: &ApprovedRoot) -> Vec<Value> {
    let mut records = Vec::new();
    let repo_title = format!("Repo context: {}", approved.repo_name);
    let stats = snapshot.get("stats").cloned().unwrap_or_else(|| json!({}));

    records.push(json!({
        "record_type": "sourceos.lampstand.repo_context_record.v1",
        "title": repo_title,
        "object_kind": "repo_context",
        "source_root_id": Value::Null,
        "path_ref": approved.path_ref,
        "content_hash": Value::Null,
        "metadata_hash": stable_id(&stats.to_string()),
        "snippet": format!("Bounded Smart Tree repo context for {}.", approved.repo_name),
        "handling_tags": ["local-only", "repo-context", "smart-tree"],
        "freshness": Value::Null,
        "policy_decision": policy_decision(vec!["lampstand.search_record.publish.local"]),
        "source": {
            "system": ADAPTER_NAME,
            "repo": TOOL_REPO
        }
    }));

    records.push(json!({
        "record_type": "sourceos.lampstand.repo_structure_record.v1",
        "title": format!("Repo structure: {}", approved.repo_name),
        "object_kind": "repo_structure",
        "source_root_id": Value::Null,
        "path_ref": approved.path_ref,
        "content_hash": Value::Null,
        "metadata_hash": stable_id(&snapshot.get("summary").cloned().unwrap_or_else(|| json!({})).to_string()),
        "snippet": "Repository structure summary generated by Smart Tree adapter.",
        "handling_tags": ["local-only", "repo-structure", "smart-tree"],
        "freshness": Value::Null,
        "policy_decision": policy_decision(vec!["lampstand.search_record.publish.local"]),
        "source": {
            "system": ADAPTER_NAME,
            "repo": TOOL_REPO
        }
    }));

    if let Some(signals) = snapshot.get("security_signals").and_then(|value| value.as_array()) {
        for signal in signals.iter().take(25) {
            let pattern = signal
                .get("pattern_name")
                .and_then(|value| value.as_str())
                .unwrap_or("security signal");
            records.push(json!({
                "record_type": "sourceos.lampstand.security_search_record.v1",
                "title": format!("Security signal: {}", pattern),
                "object_kind": "security_signal",
                "source_root_id": Value::Null,
                "path_ref": signal.get("path_ref").cloned().unwrap_or_else(|| json!(approved.path_ref)),
                "content_hash": Value::Null,
                "metadata_hash": stable_id(&signal.to_string()),
                "snippet": signal.get("description").cloned().unwrap_or(Value::Null),
                "handling_tags": ["local-only", "security-advisory", "smart-tree"],
                "freshness": Value::Null,
                "policy_decision": policy_decision(vec!["lampstand.search_record.publish.local", "repo.security_scan.read"]),
                "source": {
                    "system": ADAPTER_NAME,
                    "repo": TOOL_REPO
                }
            }));
        }
    }

    if let Some(candidates) = snapshot.get("memory_candidates").and_then(|value| value.as_array()) {
        for candidate in candidates.iter().take(10) {
            records.push(json!({
                "record_type": "sourceos.lampstand.memory_candidate_record.v1",
                "title": format!("Memory candidate: {}", candidate.get("candidate_type").and_then(|value| value.as_str()).unwrap_or("unknown")),
                "object_kind": "memory_candidate",
                "source_root_id": Value::Null,
                "path_ref": approved.path_ref,
                "content_hash": Value::Null,
                "metadata_hash": stable_id(&candidate.to_string()),
                "snippet": candidate.get("content").cloned().unwrap_or(Value::Null),
                "handling_tags": ["local-only", "memory-candidate", "smart-tree"],
                "freshness": Value::Null,
                "policy_decision": policy_decision(vec!["lampstand.search_record.publish.local", "memory_mesh.memory_candidate.emit"]),
                "source": {
                    "system": ADAPTER_NAME,
                    "repo": TOOL_REPO
                }
            }));
        }
    }

    records
}

fn project_types(nodes: &[FileNode]) -> Vec<String> {
    let mut types = Vec::new();
    if has_file(nodes, "Cargo.toml") {
        types.push("rust".to_string());
    }
    if has_file(nodes, "package.json") {
        types.push("node".to_string());
    }
    if has_file(nodes, "pyproject.toml") || has_file(nodes, "requirements.txt") {
        types.push("python".to_string());
    }
    if has_file(nodes, "go.mod") {
        types.push("go".to_string());
    }
    types.sort();
    types.dedup();
    types
}

fn language_summary(nodes: &[FileNode]) -> Vec<String> {
    let mut languages = Vec::new();
    for node in nodes.iter().filter(|node| !node.is_dir) {
        if let Some(ext) = node.path.extension().and_then(|ext| ext.to_str()) {
            match ext {
                "rs" => languages.push("rust"),
                "py" => languages.push("python"),
                "js" | "mjs" | "cjs" => languages.push("javascript"),
                "ts" | "tsx" => languages.push("typescript"),
                "go" => languages.push("go"),
                "sh" | "bash" | "zsh" => languages.push("shell"),
                "nix" => languages.push("nix"),
                _ => {}
            }
        }
    }
    let mut out = languages.into_iter().map(str::to_string).collect::<Vec<_>>();
    out.sort();
    out.dedup();
    out
}

fn build_systems(nodes: &[FileNode]) -> Vec<String> {
    let mut systems = Vec::new();
    if has_file(nodes, "Cargo.toml") {
        systems.push("cargo".to_string());
    }
    if has_file(nodes, "package.json") {
        systems.push("npm_or_node".to_string());
    }
    if has_file(nodes, "pyproject.toml") {
        systems.push("pyproject".to_string());
    }
    if has_file(nodes, "Makefile") {
        systems.push("make".to_string());
    }
    systems
}

fn test_systems(nodes: &[FileNode]) -> Vec<String> {
    let mut systems = Vec::new();
    if nodes.iter().any(|node| {
        node.path
            .components()
            .any(|component| component.as_os_str() == "tests")
    }) {
        systems.push("tests_dir".to_string());
    }
    if nodes.iter().any(|node| {
        node.path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.contains("test") || name.contains("spec"))
            .unwrap_or(false)
    }) {
        systems.push("test_files".to_string());
    }
    systems.sort();
    systems.dedup();
    systems
}

fn has_file(nodes: &[FileNode], name: &str) -> bool {
    nodes.iter().any(|node| {
        !node.is_dir
            && node
                .path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(|file_name| file_name == name)
                .unwrap_or(false)
    })
}

fn stats_json(stats: &TreeStats) -> Value {
    json!({
        "total_files": stats.total_files,
        "total_dirs": stats.total_dirs,
        "total_size_bytes": stats.total_size,
        "scan_time_ms": Value::Null,
        "format_time_ms": Value::Null,
        "file_types": stats.file_types
    })
}

fn adapter_response(response_type: &str, data: Value, capabilities: Vec<&str>) -> Value {
    json!({
        "schema_version": "sourceos.adapter_response.v1",
        "response_type": response_type,
        "source": "smart-tree",
        "generated_at": Utc::now().to_rfc3339(),
        "policy_profile": POLICY_PROFILE,
        "policy_decision": policy_decision(capabilities),
        "provenance": provenance(),
        "data": data
    })
}

fn adapter_error(error_code: &str, message: &str, safe_retry: bool) -> Value {
    json!({
        "schema_version": "sourceos.adapter_error.v1",
        "error_code": error_code,
        "message": message,
        "policy_decision": {
            "decision": if error_code == "policy_denied" { "deny" } else { "review_required" },
            "ruleset": POLICY_PROFILE,
            "capabilities": [],
            "redactions": [],
            "reason": message
        },
        "provenance": provenance(),
        "safe_retry": safe_retry
    })
}

fn policy_decision(capabilities: Vec<&str>) -> Value {
    json!({
        "decision": "allow",
        "ruleset": POLICY_PROFILE,
        "capabilities": capabilities,
        "redactions": []
    })
}

fn provenance() -> Value {
    json!({
        "adapter": ADAPTER_NAME,
        "adapter_version": env!("CARGO_PKG_VERSION"),
        "tool": "st",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "tool_repo": TOOL_REPO,
        "mode": "cli",
        "upstream": UPSTREAM_REPO
    })
}

fn path_ref(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(rest) = path.strip_prefix(&home) {
            return format!("~/{}", rest.display());
        }
    }
    path.display().to_string()
}

fn relative_path_ref(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|_| path_ref(path))
}

fn system_time_json(time: SystemTime) -> Value {
    let dt: DateTime<Utc> = time.into();
    json!(dt.to_rfc3339())
}

fn risk_level(level: &RiskLevel) -> &'static str {
    match level {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
        RiskLevel::Critical => "critical",
    }
}

fn policy_recommendation(level: &RiskLevel) -> &'static str {
    match level {
        RiskLevel::Low => "allow_warn",
        RiskLevel::Medium => "review",
        RiskLevel::High => "quarantine_review",
        RiskLevel::Critical => "block_review",
    }
}

fn context_kind(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("md" | "txt" | "rst") => "docs",
        Some("json" | "toml" | "yaml" | "yml") => "config",
        Some("sh" | "bash" | "zsh" | "ps1") => "executable",
        Some("rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c" | "rb") => "code",
        _ => "unknown",
    }
}

fn redacted_match(matched_text: &str) -> Value {
    if matched_text.is_empty() {
        Value::Null
    } else {
        json!(format!("[redacted:{} chars]", matched_text.chars().count()))
    }
}

fn stable_id(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}
