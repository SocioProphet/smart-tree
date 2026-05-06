# SourceOS Smart Tree Adapter Closeout Status

## Status

The SourceOS Smart Tree adapter has reached a stable bounded baseline. It now consumes Lampstand-owned root hints over unixjson and can explicitly publish governed adapter records into Lampstand's local adapter-record store.

This remains a controlled integration. Lampstand owns local root discovery and local search. Smart Tree still applies `sourceos.repo_context.read_only` before any enrichment scan. Adapter-record publication requires explicit `--publish`; default behavior remains dry-run.

## What is Done

### Runtime Commands

Implemented:

```bash
sourceos-context snapshot <repo> --format json
sourceos-context security <repo> --format json
sourceos-context lampstand-publish <repo> --format json
sourceos-context lampstand-publish <repo> --publish --socket <path> --format json
sourceos-context lampstand-roots --format json [--socket <path>]
```

### Governance

Implemented:

- SourceOS/SocioProphet integration doctrine.
- Lampstand-first local-state/search doctrine.
- Read-only repo scan policy profile.
- Explicit governed adapter-record publish path.
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
cargo test --test sourceos_context_lampstand_publish -- --nocapture
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
- Lampstand `PublishAdapterRecords` unavailable failure path.
- Lampstand `PublishAdapterRecords` unixjson success path.
- Redacted security findings.

## What is Intentionally Not Done

The following are explicitly deferred:

- Arbitrary Lampstand writes outside the governed adapter-record store.
- Raw file-content publication into Lampstand.
- Unreviewed adapter record types.
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

The adapter now has a live root-discovery consumer and an explicit governed local-record publisher, but it still must not collapse platform boundaries.

Lampstand remains the local desktop indexing/search, root-discovery, and local adapter-record authority. Memory Mesh remains the durable memory authority. Sherlock remains the interpretation layer. Policy Fabric remains the authorization layer. AgentPlane remains the routing layer.

Root hints are discovery data only. They do not authorize Smart Tree scans. Adapter records are local search summaries/signals/candidates only. They do not become durable memory without Memory Mesh promotion.

## Current Architecture

```text
sourceos-context
  -> read-only Smart Tree scanner/security primitives
  -> Lampstand RootHints unixjson consumer
  -> Lampstand PublishAdapterRecords unixjson publisher
  -> SourceOS adapter envelope
  -> Policy profile trace
  -> Lampstand dry-run records
  -> Memory candidate records
  -> schema-validated JSON outputs
```

## Stable Done Definition

The current phase is done when:

- Lampstand exposes `RootHints` and `lampstand roots`.
- Lampstand exposes governed `PublishAdapterRecords` adapter-record ingestion.
- Smart Tree consumes `RootHints` over unixjson.
- Smart Tree publishes generated governed records only when `--publish` is explicit.
- CI is green for schema examples, build, live-output validation, smoke tests, and publish-path tests.
- The adapter remains bounded and policy-gated.
- The deferred lanes remain documented and gated.

## Next Phase Entry Criteria

Do not begin arbitrary Lampstand writes until the following exist:

- Expanded record-type registry.
- Idempotent record behavior for each new record kind.
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
- Lampstand root-hints consumer: 95%
- Lampstand governed record publisher: 85%
- Policy profile: 90%
- Schema baseline: 95%
- Agent Registry manifest: 70%
- Runtime implementation: 75%
- Tests: 88%
- Validation: 88%

This is enough to call the root-discovery and governed local-record publication integration done. It is not enough to call full cross-stack integration done.
