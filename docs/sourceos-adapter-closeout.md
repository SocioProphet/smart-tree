# SourceOS Smart Tree Adapter Closeout Status

## Status

The SourceOS Smart Tree adapter has reached a stable read-only baseline.

It is safe to treat the current integration as done for the first phase once the final closeout PR is merged and CI is green.

## What is Done

### Runtime Commands

Implemented:

```bash
sourceos-context snapshot <repo> --format json
sourceos-context security <repo> --format json
sourceos-context lampstand-publish <repo> --dry-run --format json
sourceos-context lampstand-roots --format json
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
- Lampstand roots empty stub.
- Redacted security findings.

## What is Intentionally Not Done

The following are explicitly deferred:

- Real Lampstand RPC/unixjson write bridge.
- Lampstand root discovery over live RPC.
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

The first phase objective is not to connect every system. The objective is to establish a safe, bounded, validated contract.

Lampstand must remain the local desktop indexing/search authority. Memory Mesh must remain the durable memory authority. Sherlock must remain the interpretation layer. Policy Fabric must remain the authorization layer. AgentPlane must remain the routing layer.

The adapter should not collapse those boundaries.

## Current Architecture

```text
sourceos-context
  -> read-only Smart Tree scanner/security primitives
  -> SourceOS adapter envelope
  -> Policy profile trace
  -> Lampstand dry-run records
  -> Memory candidate records
  -> schema-validated JSON outputs
```

## Stable Done Definition

The first phase is done when:

- CI is green for schema examples, build, live-output validation, and smoke tests.
- The final closeout PR is merged.
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

Expected completion after final closeout merge:

- Doctrine: 100%
- Adapter contract: 95%
- Lampstand dry-run bridge: 90%
- Policy profile: 90%
- Schema baseline: 95%
- Agent Registry manifest: 70%
- Runtime implementation: 60%
- Tests: 75%
- Validation: 80%

This is enough to call the first phase done. It is not enough to call full cross-stack integration done.
