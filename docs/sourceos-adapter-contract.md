# SourceOS Smart Tree Adapter Contract

## Purpose

This contract defines the stable SourceOS/SocioProphet boundary around Smart Tree. Downstream systems must consume these normalized records rather than Smart Tree's native prose, dashboard, CLI banners, or internal terminology.

The adapter's job is to convert bounded Smart Tree observations into policy-aware JSON records for Lampstand, Memory Mesh, Sherlock, AgentPlane, Prophet Workspace, agent-term, and the code registry.

## Non-goals

The adapter does not own canonical memory, local desktop search, policy decisions, agent reasoning, workspace UI, or write operations.

The adapter must not expose Smart Tree hook installation, Smart Tree smart-edit writes, dashboard PTY, network dashboard exposure, or Smart Tree-native memory persistence in the first version.

## Commands

### `sourceos-context snapshot <repo> --format json`

Creates a bounded, read-only repository context snapshot.

Required output type: `RepoContextSnapshot`.

### `sourceos-context search <repo> <query> --format json`

Runs policy-approved content/file search within an approved repository root.

Required output type: `SearchResultSet`.

### `sourceos-context symbols <repo> --format json`

Extracts parse-only symbol/component observations where supported.

Required output type: `SymbolObservationSet`.

### `sourceos-context security <repo> --format json`

Runs advisory security scan and normalizes findings.

Required output type: `SecuritySignalSet`.

### `sourceos-context changed <repo> --format json`

Reports repo changes, if Smart Tree/daemon state or Lampstand freshness allows it.

Required output type: `RepoChangeSet`.

### `sourceos-context lampstand-roots --format json`

Reads Lampstand-discovered project roots and index freshness hints.

Required output type: `LampstandRootSet`.

### `sourceos-context lampstand-publish <repo> --format json`

Publishes approved Smart Tree repo/code/security summaries into Lampstand as local search records.

Required output type: `LampstandPublishReport`.

## Common Envelope

Every adapter response must include this envelope:

```json
{
  "schema_version": "sourceos.adapter_response.v1",
  "response_type": "RepoContextSnapshot",
  "source": "smart-tree",
  "generated_at": "2026-05-02T00:00:00Z",
  "policy_profile": "sourceos.repo_context.read_only",
  "policy_decision": {
    "decision": "allow",
    "ruleset": "sourceos.repo_context.read_only",
    "capabilities": ["repo.tree.read"],
    "redactions": []
  },
  "provenance": {
    "adapter": "sourceos-smart-tree-adapter",
    "tool": "st",
    "tool_version": "8.0.0",
    "tool_repo": "SocioProphet/smart-tree",
    "mode": "daemon_or_mcp_or_cli",
    "upstream": "8b-is/smart-tree"
  },
  "data": {}
}
```

## RepoContextSnapshot

```json
{
  "schema_version": "sourceos.repo_context_snapshot.v1",
  "repo_path_ref": "~/dev/example",
  "repo_identity": {
    "name": "example",
    "git_remote": null,
    "branch": null,
    "commit": null
  },
  "lampstand": {
    "source_root_id": null,
    "local_state_record_ids": [],
    "freshness": null,
    "publishable_records": []
  },
  "summary": {
    "project_type": [],
    "languages": [],
    "frameworks": [],
    "build_systems": [],
    "test_systems": []
  },
  "stats": {
    "total_files": 0,
    "total_dirs": 0,
    "total_size_bytes": 0,
    "scan_time_ms": 0,
    "format_time_ms": 0
  },
  "key_files": [],
  "interesting_files": [],
  "git": {},
  "security_signals": [],
  "symbol_summary": {},
  "memory_candidates": []
}
```

## FileObservation

```json
{
  "path_ref": "src/main.rs",
  "object_kind": "file",
  "category": "rust",
  "size_bytes": 0,
  "mtime": null,
  "content_hash": null,
  "metadata_hash": null,
  "is_hidden": false,
  "is_ignored": false,
  "interest_score": null,
  "change_status": null,
  "security_signal_ids": []
}
```

## SearchHit

```json
{
  "path_ref": "src/main.rs",
  "line": 1,
  "column": 1,
  "snippet": null,
  "snippet_redaction_state": "none_or_redacted_or_withheld",
  "content_hash": null,
  "score": null
}
```

## SymbolObservation

```json
{
  "symbol_id": "sha256-or-registry-id",
  "name": "example",
  "symbol_kind": "function_or_module_or_class_or_impl_or_config_unit",
  "language": "rust",
  "path_ref": "src/lib.rs",
  "line_start": null,
  "line_end": null,
  "content_hash": null,
  "visibility": null,
  "semantic_tags": [],
  "dependencies": [],
  "clearance": "private_or_team_or_internal_or_company_public_or_world_public",
  "lampstand_search_record_id": null,
  "memory_candidate_id": null
}
```

## SecuritySignal

```json
{
  "signal_id": "uuid-or-hash",
  "path_ref": "settings.json",
  "line": null,
  "pattern_name": "Auto Hook",
  "risk_level": "low_or_medium_or_high_or_critical",
  "description": "Advisory security signal.",
  "matched_text_redacted": null,
  "context_kind": "code_or_docs_or_history_or_config_or_executable",
  "policy_recommendation": "allow_warn_quarantine_block_review",
  "lampstand_record_id": null,
  "memory_candidate_id": null
}
```

## MemoryCandidate

```json
{
  "candidate_id": "uuid-or-hash",
  "candidate_type": "repo_onboarding_or_work_episode_or_symbol_or_security_or_procedural",
  "confidence": 0.0,
  "content": "Candidate memory content.",
  "tags": [],
  "source_refs": [],
  "policy_labels": [],
  "recommended_action": "promote_or_review_or_discard"
}
```

## LampstandSearchRecord

```json
{
  "record_type": "lampstand.search_record.v1",
  "title": "Repo summary: example",
  "object_kind": "repo_context_or_symbol_or_security_signal",
  "source_root_id": null,
  "path_ref": "~/dev/example",
  "content_hash": null,
  "metadata_hash": null,
  "snippet": null,
  "handling_tags": [],
  "freshness": null,
  "policy_decision": {},
  "source": {
    "system": "smart-tree-adapter",
    "repo": "SocioProphet/smart-tree"
  }
}
```

## Error Contract

Errors must be structured and policy-aware:

```json
{
  "schema_version": "sourceos.adapter_error.v1",
  "error_code": "policy_denied_or_scan_failed_or_tool_unavailable_or_invalid_root",
  "message": "Human-readable error.",
  "policy_decision": {},
  "provenance": {},
  "safe_retry": false
}
```

## First Implementation Target

Implement `snapshot` first. It should be read-only, bounded to approved roots, and safe to feed into Sherlock and Memory Mesh.

The second implementation target is `lampstand-publish`, which converts approved snapshot/symbol/security summaries into Lampstand local search records.
