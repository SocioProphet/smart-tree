# SourceOS Smart Tree Adapter Closeout Status

## Status

The SourceOS Smart Tree adapter has reached a stable read-only baseline and now consumes Lampstand-owned root hints over unixjson when a Lampstand daemon is available.

It remains safe to treat this integration as bounded and read-only: Lampstand owns local root discovery, and Smart Tree still applies `sourceos.repo_context.read_only` before any enrichment scan.

## What is Done

### Runtime Commands

Implemented:

```bash
sourceos-context snapshot <repo> --format json
sourceos-context security <repo> --format json
sourceos-context lampstand-publish <repo> --dry-run --format json
sourceos-context lampstand-roots --format json [--socket <path>]
```

### Governance

Implemented:

- SourceOS/SocioProphet integration doctrine.
- Lampstand-first local-state/search doctrine.
- Read-only policy profile.
- Agent Registry draft manifest.
- Implementation roadmap.
- Stable closeout target.

### Validation

Implemented CI checks:

```bash
python tools/validate_sourceos_adapter_schema.py
cargo build --bin sourceos-context
python tools/validate_sourceos_adapter_live_outputs.py
cargo test --test sourceos_context_cli -- --nocapture
```

Validation coverage:

- Static schema examples.
- Live adapter output schema validation.
- Allowed repo snapshot under `~/dev/**`.
- Denied repo outside `~/dev/**`.
- Denied unbounded home-root snapshot.
- Denied symlink-root snapshot.
- Lampstand dry-run records.
- Lampstand unavailable failure path.
- Lampstand `RootHints` unixjson success path.
- Redacted security findings.

## What is Intentionally Not Done

The following are explicitly deferred:

- Real Lampstand RPC/unixjson write bridge.
- Sherlock ingestion.
- Memory Mesh promotion API integration.
- Symbol extraction / SmartPastCode runtime integration.
- AgentPlane service registration runtime consumption.
- Prophet Workspace UI integration.
- agent-term command integration.
- Watch mode.
- Smart Tree smart-edit writes.
- Hook installation or mutation.
- Dashboard exposure.
- PTY exposure.
- External callbacks, update checks, or feedback submission through the adapter.
- Smart Tree-native global memory persistence.

## Why Those Deferrals Are Correct

The adapter now has a live root-discovery consumer, but it still must not collapse platform boundaries.

Lampstand remains the local desktop indexing/search and root-discovery authority. Memory Mesh remains the durable memory authority. Sherlock remains the interpretation layer. Policy Fabric remains the authorization layer. AgentPlane remains the routing layer.

Root hints are discovery data only. They do not authorize Smart Tree scans.

## Current Architecture

```text
sourceos-context
  -> read-only Smart Tree scanner/security primitives
  -> Lampstand RootHints unixjson consumer
  -> SourceOS adapter envelope
  -> Policy profile trace
  -> Lampstand dry-run records
  -> Memory candidate records
  -> schema-validated JSON outputs
```

## Stable Done Definition

The current phase is done when:

- Lampstand exposes `RootHints` and `lampstand roots`.
- Smart Tree consumes `RootHints` over unixjson.
- CI is green for schema examples, build, live-output validation, and smoke tests.
- The adapter remains read-only.
- The deferred lanes remain documented and gated.

## Next Phase Entry Criteria

Do not begin a real Lampstand write bridge until the following exist:

- Confirmed Lampstand RPC/unixjson publish contract.
- Idempotent record write behavior.
- Local-only classification and redaction review.
- Policy Fabric approval path.
- Lampstand owner review.

Do not begin SmartPastCode/symbol extraction until the following exist:

- Parse-only extractor boundary.
- Symbol output schema test fixtures.
- Code registry candidate schema review.
- Memory Mesh symbol memory review.

Do not begin watch mode until the following exist:

- Lampstand watcher/reconciler integration plan.
- Queue semantics.
- Deduplication strategy.
- Explicit policy review.

## Final Readout After Merge

Expected completion after this merge:

- Doctrine: 100%
- Adapter contract: 98%
- Lampstand root-hints consumer: 90%
- Lampstand dry-run bridge: 90%
- Policy profile: 90%
- Schema baseline: 95%
- Agent Registry manifest: 70%
- Runtime implementation: 68%
- Tests: 82%
- Validation: 85%

This is enough to call the root-discovery integration done. It is not enough to call full cross-stack integration done.
