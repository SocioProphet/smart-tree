# SourceOS Smart Tree Implementation Roadmap

## Purpose

This roadmap turns the SocioProphet integration plan into an executable sequence. It assumes issues may be disabled in this repository, so PR comments and this document serve as the initial work-order surface.

## North Star

Make Smart Tree a constrained, replaceable repo/code sensing engine for the SourceOS/SocioProphet stack.

Smart Tree should produce bounded, policy-aware observations. Lampstand owns local indexing/search. Memory Mesh owns durable memory. Sherlock owns interpretation. AgentPlane owns routing. Policy Fabric owns authorization.

## Stable Done Target for the Current Closeout

The near-term target is not full Lampstand write integration. The stable target is a merged, tested, read-only adapter baseline that is safe for downstream integration work.

Done means:

- `sourceos-context` builds in CI.
- Adapter smoke tests pass in CI.
- JSON schema examples validate in CI.
- `snapshot`, `security`, `lampstand-publish --dry-run`, and `lampstand-roots` exist.
- Lampstand remains first-class and is not bypassed.
- No hooks, writes, dashboard exposure, PTY, external callbacks, or native Smart Tree global memory persistence are exposed.
- Real Lampstand RPC/unixjson writes, Sherlock ingestion, Memory Mesh promotion, and symbol extraction are explicitly deferred behind follow-up gates.

## Milestone 0: Documentation and Governance Baseline

Status: merged in PR #1.

Deliverables:

- `SOCIOPROPHET-INTEGRATION.md`
- `docs/sourceos-adapter-contract.md`
- `docs/lampstand-bridge.md`
- `policy/sourceos.repo_context.read_only.yaml`
- `schemas/sourceos-smart-tree-adapter.v1.schema.json`

Acceptance:

- Lampstand integration is first-class.
- Read-only policy boundary is explicit.
- Adapter output schemas exist.
- No code path grants write, hook, dashboard, PTY, or external network capabilities.

## Milestone 1: Adapter Skeleton

Status: merged in PR #1.

Commands:

```bash
sourceos-context snapshot <repo> --format json
sourceos-context security <repo> --format json
sourceos-context lampstand-publish <repo> --dry-run --format json
```

Acceptance:

- Every response includes policy profile and provenance.
- Invalid roots fail closed.
- Denied capabilities are not callable.
- `--dry-run` is available for Lampstand publication.

## Milestone 2: Policy Gate Integration

Status: merged in PR #1, hardened in PR #2.

Acceptance:

- `~/dev/<repo>` is allowed.
- `/`, `/etc`, `/proc`, `~/.ssh`, and unbounded `~` are denied.
- Symlink following is denied.
- Hidden-file reads require explicit escalation.
- External network calls are denied by default.

## Milestone 3: Snapshot Implementation

Status: merged in PR #1.

Acceptance:

- Snapshot avoids raw content unless explicitly requested and approved.
- Snapshot includes Lampstand link fields even when Lampstand is unavailable.
- Snapshot is covered by CLI smoke tests.

Remaining hardening:

- Add full schema validation against live command outputs.
- Add richer git remote/commit capture later.

## Milestone 4: Lampstand Dry-Run Bridge

Status: merged in PR #1, hardened in PR #2.

Acceptance:

- Dry-run returns records and policy decisions.
- Raw content is not published.
- Snippets are limited to generated summaries.
- Lampstand unavailable does not break normal snapshot mode.
- `lampstand-roots` returns an explicit empty stub until real RPC/unixjson integration exists.

## Milestone 5: Lampstand Write Bridge

Status: deferred.

This is intentionally not part of the current stable done target.

Required before implementation:

- Lampstand owner review.
- Policy Fabric review.
- Exact RPC/unixjson contract from `SocioProphet/lampstand`.
- Idempotency and stale-record behavior.
- Local-only classification and redaction rules.

## Milestone 6: Memory Mesh Candidate Emission

Status: initial repo-onboarding candidate merged in PR #1.

Remaining:

- Security memory candidates.
- Procedural memory candidates.
- Deduplication strategy.
- Real Memory Mesh promotion API integration.

## Milestone 7: Sherlock Consumption

Status: deferred.

Remaining:

- Document Sherlock adapter input.
- Add example repo dossier generated from snapshot.
- Add drift comparison contract: prior memory vs current snapshot.

## Milestone 8: Symbol / SmartPastCode Lane

Status: deferred.

Remaining:

- Rust symbol extraction mapping.
- `SymbolObservationSet` output.
- Registry candidate mapping.
- Lampstand `SymbolSearchRecord` mapping.
- Future language expansion notes.

## Milestone 9: Security Signal Lane

Status: initial security signal normalization merged in PR #1.

Remaining:

- Documentation-only downgrade rules.
- More false-positive controls.
- Security memory candidates.
- Policy Fabric advisory handoff.

## Milestone 10: Agent Registry Registration

Status: draft manifest merged in PR #1.

Remaining:

- Register/consume from the actual Agent Registry repo or service.
- Add validation for manifest shape.

## Milestone 11: Prophet Workspace / agent-term UX

Status: deferred.

Expected commands:

```text
/context snapshot
/context security
/context lampstand-roots
/context lampstand-publish
```

Search and symbol commands remain future work until those adapter operations exist.

## Milestone 12: Watch Mode and Controlled Writes

Status: deferred and explicitly out of scope.

Rules:

- Lampstand remains watcher/reconciler.
- Smart Tree only enriches approved roots.
- Write operations require GitOps branch/patch/PR flow.
- Dashboard/PTY requires explicit security review.

## Validation Matrix

Current validated checks:

- CI build for `sourceos-context`.
- CI smoke tests for snapshot, denied root, Lampstand dry-run, and security redaction.
- CI schema example validation is being added in PR #2.

Remaining checks:

- Live output schema validation.
- Policy denial matrix expansion.
- Symlink traversal denial test.
- Unbounded home-directory denial test.
- Lampstand record idempotency when real write bridge exists.

## Progress Readout

Current status after PR #1 merge and PR #2 hardening work:

- Fork hygiene / integration doctrine: 90%
- Adapter contract: 85%
- Policy profile: 85%
- Lampstand bridge: 80%
- Schema baseline: 80%
- Implementation code: 45%
- Tests: 45%
- Runtime validation: 45%

## Immediate Next Step

Merge PR #2 after CI passes, then add live-output schema validation and final closeout documentation.
