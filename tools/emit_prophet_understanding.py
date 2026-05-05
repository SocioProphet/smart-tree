#!/usr/bin/env python3
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import subprocess
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "prophet-understanding.v0"
DEFAULT_OUT = ".prophet/prophet-understanding.json"
SKIP_DIRS = {
    ".git",
    ".hg",
    ".svn",
    ".prophet",
    ".venv",
    "venv",
    "node_modules",
    "target",
    "dist",
    "build",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".next",
    ".turbo",
    "coverage",
    "vendor",
}
SKIP_SUFFIXES = {
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".ico",
    ".pdf",
    ".zip",
    ".gz",
    ".tar",
    ".tgz",
    ".xz",
    ".bz2",
    ".7z",
    ".dmg",
    ".pkg",
    ".exe",
    ".dll",
    ".so",
    ".dylib",
    ".class",
    ".o",
    ".a",
    ".pyc",
}
DOC_SUFFIXES = {".md", ".mdx", ".rst", ".txt"}
SOURCE_SUFFIXES = {".py", ".rs", ".go", ".ts", ".tsx", ".js", ".jsx", ".java", ".c", ".cc", ".cpp", ".h", ".hpp", ".rb", ".php", ".swift", ".kt", ".cs"}
SCHEMA_SUFFIXES = {".json", ".yaml", ".yml", ".toml"}


def sha256_bytes(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def sha256_text(text: str) -> str:
    return sha256_bytes(text.encode("utf-8"))


def run_git(repo: Path, args: list[str], default: str) -> str:
    try:
        result = subprocess.run(["git", *args], cwd=repo, text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, check=False)
    except OSError:
        return default
    value = result.stdout.strip()
    return value if result.returncode == 0 and value else default


def stable_id(kind: str, value: str) -> str:
    safe = value.strip().replace("\\", "/")
    safe = safe.replace(" ", "%20")
    return f"{kind}:{safe}"


def rel(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def count_lines(data: bytes) -> int:
    if not data:
        return 1
    try:
        text = data.decode("utf-8", errors="replace")
    except Exception:
        return 1
    return max(1, text.count("\n") + (0 if text.endswith("\n") else 1))


def file_kind(path: Path, root: Path) -> str:
    rel_path = rel(path, root)
    name = path.name.lower()
    suffix = path.suffix.lower()
    parts = set(Path(rel_path).parts)

    if ".github" in parts and "workflows" in parts:
        return "workflow"
    if "test" in name or "tests" in parts or suffix in {".spec.ts", ".test.ts"}:
        return "test"
    if suffix in DOC_SUFFIXES:
        return "document"
    if "policy" in rel_path.lower():
        return "policy"
    if "schema" in rel_path.lower() or suffix == ".json" and ("schema" in name or "contract" in rel_path.lower()):
        return "schema"
    if "contract" in rel_path.lower() or rel_path.startswith("contracts/"):
        return "contract"
    if name in {"cargo.toml", "package.json", "pyproject.toml", "go.mod", "pom.xml", "build.gradle", "requirements.txt"}:
        return "package"
    if name in {"dockerfile", "makefile"} or suffix in {".yaml", ".yml", ".toml", ".ini", ".cfg", ".conf"}:
        return "config"
    if suffix in SOURCE_SUFFIXES:
        return "module"
    return "file"


def should_skip(path: Path, root: Path) -> tuple[bool, str | None]:
    rel_path = rel(path, root)
    parts = set(Path(rel_path).parts)
    if parts & SKIP_DIRS:
        return True, "ignored-directory"
    if path.suffix.lower() in SKIP_SUFFIXES:
        return True, "binary-or-archive"
    lowered = rel_path.lower()
    if any(token in lowered for token in ["secret", "private_key", "id_rsa", ".pem", ".p12"]):
        return True, "secret-like-path"
    try:
        if path.stat().st_size > 1_000_000:
            return True, "large-file"
    except OSError:
        return True, "unreadable-stat"
    return False, None


def add_receipt(receipts: list[dict[str, Any]], receipt_id: str, claim_type: str, input_hash: str, generated_at: str, confidence: float = 1.0, warnings: list[str] | None = None) -> str:
    receipts.append(
        {
            "id": receipt_id,
            "claim_type": claim_type,
            "generator": "smart-tree",
            "parser_version": "smart-tree-prophet-understand-v0",
            "input_source_hash": input_hash,
            "generated_at": generated_at,
            "confidence": confidence,
            "validation_state": "warning" if warnings else "valid",
            "warnings": warnings or [],
        }
    )
    return receipt_id


def emit(repo: Path, out: Path, repo_full_name: str | None) -> dict[str, Any]:
    repo = repo.resolve()
    generated_at = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    commit = run_git(repo, ["rev-parse", "HEAD"], "unknown")
    branch = run_git(repo, ["rev-parse", "--abbrev-ref", "HEAD"], "unknown")
    if not repo_full_name:
        origin = run_git(repo, ["remote", "get-url", "origin"], "")
        repo_full_name = origin.rstrip("/").removesuffix(".git").split(":")[-1].split("github.com/")[-1] if origin else repo.name

    receipts: list[dict[str, Any]] = []
    nodes: list[dict[str, Any]] = []
    edges: list[dict[str, Any]] = []
    validation_results: list[dict[str, Any]] = []
    summaries: list[dict[str, Any]] = []
    skipped: list[dict[str, str]] = []

    run_hash = sha256_text(f"{repo_full_name}:{commit}:{branch}")
    run_receipt = add_receipt(receipts, "receipt:smart-tree-run", "repo-scan", run_hash, generated_at)

    repo_node_id = stable_id("repo", repo_full_name)
    nodes.append(
        {
            "id": repo_node_id,
            "kind": "repo",
            "label": repo_full_name,
            "path": ".",
            "confidence": 1.0,
            "provenance_receipt_ids": [run_receipt],
            "metadata": {"branch": branch, "commit": commit},
        }
    )

    dir_nodes: dict[str, str] = {".": repo_node_id}

    for current, dirs, files in os.walk(repo):
        current_path = Path(current)
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
        rel_current = "." if current_path == repo else rel(current_path, repo)
        if rel_current != "." and rel_current not in dir_nodes:
            dir_id = stable_id("directory", rel_current)
            dir_nodes[rel_current] = dir_id
            parent_rel = "." if Path(rel_current).parent.as_posix() == "." else Path(rel_current).parent.as_posix()
            parent_id = dir_nodes.get(parent_rel, repo_node_id)
            nodes.append({"id": dir_id, "kind": "directory", "label": Path(rel_current).name, "path": rel_current, "confidence": 1.0, "provenance_receipt_ids": [run_receipt], "metadata": {}})
            edges.append({"id": stable_id("edge", f"{parent_id}->contains->{dir_id}"), "kind": "contains", "source": parent_id, "target": dir_id, "confidence": 1.0, "provenance_receipt_ids": [run_receipt], "metadata": {}})

        for filename in sorted(files):
            path = current_path / filename
            skip, reason = should_skip(path, repo)
            rel_path = rel(path, repo)
            if skip:
                skipped.append({"path": rel_path, "reason": reason or "skipped"})
                continue
            try:
                data = path.read_bytes()
            except OSError:
                skipped.append({"path": rel_path, "reason": "unreadable"})
                continue
            content_hash = sha256_bytes(data)
            kind = file_kind(path, repo)
            node_id = stable_id(kind, rel_path)
            receipt_id = add_receipt(receipts, stable_id("receipt", rel_path), f"{kind}-node", content_hash, generated_at)
            line_count = count_lines(data)
            node = {
                "id": node_id,
                "kind": kind,
                "label": filename,
                "path": rel_path,
                "source_anchor": {"path": rel_path, "start_line": 1, "end_line": line_count, "content_hash": content_hash},
                "confidence": 1.0 if kind != "file" else 0.7,
                "provenance_receipt_ids": [receipt_id],
                "metadata": {"size_bytes": len(data)},
            }
            nodes.append(node)
            parent_rel = rel_current
            parent_id = dir_nodes.get(parent_rel, repo_node_id)
            edges.append({"id": stable_id("edge", f"{parent_id}->contains->{node_id}"), "kind": "contains", "source": parent_id, "target": node_id, "confidence": 1.0, "provenance_receipt_ids": [run_receipt, receipt_id], "metadata": {}})

            if kind in {"document", "schema", "contract", "policy", "workflow", "test"}:
                summaries.append({"id": stable_id("summary", rel_path), "node_id": node_id, "text": f"{kind} artifact at {rel_path}.", "confidence": 0.75, "provenance_receipt_ids": [receipt_id]})

    if skipped:
        add_receipt(receipts, "receipt:skipped-paths", "skip-receipts", sha256_text(json.dumps(skipped, sort_keys=True)), generated_at, 0.9, ["Some paths were skipped by default safety rules."])
        validation_results.append({"id": "validation:skipped-paths", "check_id": "skip-receipts-present", "target_id": repo_node_id, "status": "warn", "severity": "warning", "message": f"{len(skipped)} paths skipped; inspect metadata.skipped_paths."})

    validation_results.append({"id": "validation:source-anchors", "check_id": "source-anchor-coverage", "target_id": repo_node_id, "status": "pass", "severity": "info", "message": "All emitted file-like nodes include source anchors."})
    validation_results.append({"id": "validation:stable-ids", "check_id": "stable-id-shape", "target_id": repo_node_id, "status": "pass", "severity": "info", "message": "Node and edge IDs are derived from repo-relative paths and relationship tuples."})

    architecture_steps = []
    for index, node in enumerate([n for n in nodes if n["kind"] in {"document", "schema", "contract", "workflow", "package"}][:12], start=1):
        architecture_steps.append({"order": index, "node_id": node["id"], "edge_ids": [], "summary": f"Review {node['label']} as a {node['kind']} artifact."})

    tours = []
    if architecture_steps:
        tours.append({"id": "tour:architecture", "kind": "architecture", "title": "Smart Tree architecture tour", "steps": architecture_steps, "provenance_receipt_ids": [run_receipt]})

    artifact: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "repo": {"full_name": repo_full_name, "default_branch": branch, "commit": commit, "generated_at": generated_at, "artifact_hash": "sha256:pending"},
        "generator": {"name": "smart-tree", "version": "smart-tree-prophet-understand-v0", "parser_versions": {"filesystem": "v0", "git": "v0"}},
        "agent_identity": {"kind": "local", "id": "agent://smart-tree/local-emitter", "did": None},
        "nodes": sorted(nodes, key=lambda x: x["id"]),
        "edges": sorted(edges, key=lambda x: x["id"]),
        "summaries": sorted(summaries, key=lambda x: x["id"]),
        "tours": tours,
        "diff_impact_sets": [],
        "provenance_receipts": sorted(receipts, key=lambda x: x["id"]),
        "validation_results": validation_results,
        "policy_status": {"state": "warn" if skipped else "allow", "checks": [{"id": "policy:local-scan-only", "state": "allow", "message": "Baseline graph emission is local and read-only.", "evidence_receipt_ids": [run_receipt]}, {"id": "policy:skipped-path-review", "state": "warn" if skipped else "allow", "message": "Skipped paths require review before claiming complete coverage." if skipped else "No skipped paths were recorded.", "evidence_receipt_ids": ["receipt:skipped-paths"] if skipped else [run_receipt]}]},
    }
    artifact["repo"]["artifact_hash"] = sha256_text(json.dumps(artifact, sort_keys=True, separators=(",", ":")))
    artifact["repo"]["metadata"] = {"skipped_paths": skipped}
    return artifact


def main() -> None:
    parser = argparse.ArgumentParser(description="Emit Prophet Understand repo intelligence artifact.")
    parser.add_argument("--repo", default=".", help="Repository root to scan")
    parser.add_argument("--out", default=DEFAULT_OUT, help="Output artifact path")
    parser.add_argument("--repo-full-name", default=None, help="Override owner/name repo identifier")
    args = parser.parse_args()

    repo = Path(args.repo)
    out = Path(args.out)
    artifact = emit(repo, out, args.repo_full_name)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(artifact, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
