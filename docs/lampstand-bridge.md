# Lampstand Bridge for Smart Tree

## Purpose

This bridge defines how Smart Tree integrates with Lampstand without replacing or bypassing Lampstand.

Lampstand is the SourceOS local desktop indexing and search authority. Smart Tree is a bounded repo/code intelligence sensor. The bridge connects them through governed records.

## Core Rule

Smart Tree enriches approved project roots. Lampstand owns local-state indexing, search, reconciliation, and freshness.

Smart Tree must not perform desktop-wide local search or unbounded home-directory scanning. Lampstand discovers local state; Policy Fabric approves enrichment; Smart Tree performs deeper repo/code analysis only inside approved roots.

## Data Flow

```text
Lampstand indexed local state
  -> project root / freshness hint
  -> Policy Fabric approval
  -> Smart Tree bounded repo/code scan
  -> normalized adapter records
  -> Lampstand local search records
  -> Sherlock / Memory Mesh / AgentPlane as approved
```

## Lampstand to Smart Tree Inputs

### ProjectRootHint

```json
{
  "schema_version": "lampstand.project_root_hint.v1",
  "source_root_id": "root-id",
  "path_ref": "~/dev/example",
  "root_kind": "git_repo_or_project_or_document_root",
  "freshness": {
    "last_indexed_at": null,
    "last_reconciled_at": null,
    "dirty": false
  },
  "classification": "local_only",
  "handling_tags": [],
  "content_fingerprint_summary": null
}
```

### FreshnessHint

```json
{
  "schema_version": "lampstand.freshness_hint.v1",
  "source_root_id": "root-id",
  "path_ref": "~/dev/example",
  "changed_since": null,
  "candidate_files": [],
  "confidence": 1.0
}
```

### SearchTrigger

```json
{
  "schema_version": "lampstand.search_trigger.v1",
  "query": "policy fabric",
  "candidate_roots": [],
  "candidate_records": [],
  "requested_enrichment": "repo_context_or_symbols_or_security"
}
```

## Smart Tree to Lampstand Outputs

### RepoContextRecord

```json
{
  "schema_version": "sourceos.lampstand.repo_context_record.v1",
  "record_id": "uuid-or-hash",
  "source_root_id": "lampstand-root-id",
  "path_ref": "~/dev/example",
  "title": "Repo context: example",
  "snippet": "Bounded repo summary for local search.",
  "object_kind": "repo_context",
  "content_hash": null,
  "metadata_hash": null,
  "handling_tags": [],
  "classification": "local_only",
  "policy_decision": {},
  "provenance": {
    "system": "smart-tree-adapter",
    "tool": "st",
    "mode": "daemon_or_mcp_or_cli"
  }
}
```

### RepoStructureRecord

```json
{
  "schema_version": "sourceos.lampstand.repo_structure_record.v1",
  "record_id": "uuid-or-hash",
  "source_root_id": "lampstand-root-id",
  "path_ref": "~/dev/example",
  "object_kind": "repo_structure",
  "summary": {
    "languages": [],
    "key_files": [],
    "build_systems": [],
    "test_systems": []
  },
  "stats": {},
  "content_hash": null,
  "metadata_hash": null,
  "handling_tags": [],
  "classification": "local_only"
}
```

### SymbolSearchRecord

```json
{
  "schema_version": "sourceos.lampstand.symbol_search_record.v1",
  "record_id": "uuid-or-hash",
  "source_root_id": "lampstand-root-id",
  "path_ref": "src/lib.rs",
  "object_kind": "symbol",
  "symbol": {
    "name": "example",
    "kind": "function",
    "language": "rust",
    "line_start": null,
    "line_end": null,
    "semantic_tags": []
  },
  "content_hash": null,
  "classification": "local_only",
  "handling_tags": []
}
```

### SecuritySearchRecord

```json
{
  "schema_version": "sourceos.lampstand.security_search_record.v1",
  "record_id": "uuid-or-hash",
  "source_root_id": "lampstand-root-id",
  "path_ref": "settings.json",
  "object_kind": "security_signal",
  "signal": {
    "pattern_name": "Auto Hook",
    "risk_level": "high",
    "description": "Advisory signal only. Policy Fabric adjudicates."
  },
  "matched_text_redacted": null,
  "classification": "local_only",
  "handling_tags": ["security-advisory"]
}
```

### MemoryCandidateRecord

```json
{
  "schema_version": "sourceos.lampstand.memory_candidate_record.v1",
  "record_id": "uuid-or-hash",
  "source_root_id": "lampstand-root-id",
  "object_kind": "memory_candidate",
  "candidate_type": "repo_onboarding_or_symbol_or_security_or_procedural",
  "title": "Memory candidate: repo onboarding",
  "snippet": "Candidate memory summary for local review.",
  "memory_mesh_candidate_id": null,
  "classification": "local_only",
  "handling_tags": []
}
```

## Publication Rules

Publication to Lampstand must be explicit and policy-gated.

Default allowed:

- metadata summaries;
- file path references under approved roots;
- content hashes;
- repo stats;
- language/build/test signals;
- symbol names and locations;
- redacted security signals;
- local-only memory candidate summaries.

Default denied:

- raw file content;
- unredacted secrets;
- full home-directory path publication where path-tokenization is required;
- snippets from sensitive files;
- unpublished external registry submissions;
- Smart Tree-native memory blobs.

## Reconciliation

Lampstand owns reconciliation. Smart Tree enrichment records should include enough provenance for Lampstand to mark them stale when the underlying source root changes.

Required provenance fields:

- `source_root_id`
- `path_ref`
- `content_hash` where available
- `metadata_hash` where available
- `generated_at`
- `tool_version`
- `adapter_version`
- `policy_profile`

## Health and Freshness

The bridge should expose health checks:

```bash
sourceos-context lampstand-roots --format json
sourceos-context lampstand-publish ~/dev/example --dry-run --format json
```

`--dry-run` must show publishable records and policy decisions without writing to Lampstand.

## Failure Modes

If Lampstand is unavailable:

- snapshot may still run against an explicitly approved repo root;
- `lampstand-publish` must fail closed;
- no desktop-wide discovery may occur;
- Memory Mesh candidates may still be emitted if policy allows.

If Policy Fabric denies publication:

- no Lampstand records are written;
- adapter returns structured denial;
- Sherlock and Memory Mesh may receive only redacted denial context if policy allows.

## First Milestone

Implement dry-run publication mapping from a `RepoContextSnapshot` to:

- one `RepoContextRecord`;
- one `RepoStructureRecord`;
- zero or more `SecuritySearchRecord` entries;
- zero or more `SymbolSearchRecord` entries when symbol extraction exists;
- zero or more `MemoryCandidateRecord` entries.
