# SourceOS Smart Tree Implementation Roadmap

## Purpose

This roadmap turns the SocioProphet integration plan into an executable sequence. It assumes issues may be disabled in this repository, so PR comments and this document serve as the initial work-order surface.

## North Star

Make Smart Tree a constrained, replaceable repo/code sensing engine for the SourceOS/SocioProphet stack.

Smart Tree should produce bounded, policy-aware observations. Lampstand owns local indexing/search. Memory Mesh owns durable memory. Sherlock owns interpretation. AgentPlane owns routing. Policy Fabric owns authorization.

## Milestone 0: Documentation and Governance Baseline

Status: in progress on PR #1.

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

Goal: create the SourceOS adapter entry point without exposing unsafe capabilities.

Deliverables:

- Adapter module or binary stub.
- `sourceos-context --help` style command surface or documented equivalent.
- Adapter response envelope implementation.
- Structured error envelope implementation.
- Schema validation in tests.

Commands:

```bash
sourceos-context snapshot <repo> --format json
sourceos-context search <repo> <query> --format json
sourceos-context security <repo> --format json
sourceos-context lampstand-publish <repo> --dry-run --format json
```

Acceptance:

- Every response includes policy profile and provenance.
- Invalid roots fail closed.
- Denied capabilities are not callable.
- `--dry-run` is available for Lampstand publication.

## Milestone 2: Policy Gate Integration

Goal: enforce `sourceos.repo_context.read_only` before every adapter action.

Deliverables:

- Policy profile loader.
- Root/path validator.
- Capability validator.
- Redaction marker support.
- Policy decision trace in every response.

Acceptance:

- `~/dev/<repo>` is allowed.
- `/`, `/etc`, `/proc`, `~/.ssh`, and unbounded `~` are denied.
- Symlink following is denied.
- Hidden-file reads require explicit escalation.
- External network calls are denied by default.

## Milestone 3: Snapshot Implementation

Goal: produce `RepoContextSnapshot` for approved repo roots.

Deliverables:

- Bounded scan using Smart Tree scanner/daemon/MCP.
- Key files extraction.
- Repo stats mapping.
- Git branch/remote/commit mapping where available.
- Interesting file mapping.
- Security signal mapping where available.
- Memory candidate generation for repo onboarding.

Acceptance:

- Snapshot validates against schema.
- Snapshot is stable on repeated runs.
- Snapshot avoids raw content unless explicitly requested and approved.
- Snapshot includes Lampstand link fields even when Lampstand is unavailable.

## Milestone 4: Lampstand Dry-Run Bridge

Goal: map snapshots into Lampstand-compatible local search records without writing.

Deliverables:

- `lampstand-publish --dry-run`.
- Mapping to:
  - `RepoContextRecord`
  - `RepoStructureRecord`
  - `SecuritySearchRecord`
  - `SymbolSearchRecord` when symbols exist
  - `MemoryCandidateRecord`
- Publication policy checks.

Acceptance:

- Dry-run returns records and policy decisions.
- Raw content is not published.
- Snippets are withheld unless classification allows them.
- Lampstand unavailable does not break normal snapshot mode.

## Milestone 5: Lampstand Write Bridge

Goal: publish approved records into Lampstand through its service boundary.

Deliverables:

- Lampstand RPC/unixjson client adapter.
- Publish report with accepted/rejected counts.
- Idempotency strategy using content/metadata hashes.
- Staleness/freshness metadata.

Acceptance:

- Publication fails closed if Lampstand is unavailable.
- Published records include provenance and policy decision.
- Repeated publication does not duplicate records unnecessarily.
- Lampstand remains source of local search truth.

## Milestone 6: Memory Mesh Candidate Emission

Goal: generate useful Memory Mesh candidates without persisting Smart Tree-native memory.

Deliverables:

- Repo onboarding memory candidate.
- Security memory candidates.
- Procedural memory candidates where command/build/test signals are detected.
- Candidate IDs and source references.
- Recommended action: promote/review/discard.

Acceptance:

- Smart Tree-native `.m8` persistence remains disabled by default.
- Candidates are bounded, deduplicated, and tagged.
- Candidate output validates against schema.

## Milestone 7: Sherlock Consumption

Goal: make Sherlock able to use snapshots as live repo evidence.

Deliverables:

- Documented Sherlock adapter input.
- Example repo dossier generated from a snapshot.
- Drift comparison contract: prior memory vs current snapshot.

Acceptance:

- Sherlock can ingest a snapshot without Smart Tree-specific parsing.
- Sherlock can report gaps, drift, risks, and next actions.

## Milestone 8: Symbol / SmartPastCode Lane

Goal: normalize code component extraction for the code registry and Memory Mesh symbol memory.

Deliverables:

- Rust symbol extraction mapping.
- `SymbolObservationSet` output.
- Registry candidate mapping.
- Lampstand `SymbolSearchRecord` mapping.
- Future language expansion notes.

Acceptance:

- Symbol output validates against schema.
- Extraction is parse-only and does not execute code.
- Each symbol includes origin, path, line range where possible, content hash, semantic tags, and clearance.

## Milestone 9: Security Signal Lane

Goal: route Smart Tree security scanner findings into Policy Fabric and Lampstand records.

Deliverables:

- Security scanner mapping to `SecuritySignalSet`.
- Severity normalization.
- Context-kind normalization: docs/history/config/executable/code.
- Policy recommendations.
- False-positive handling notes.

Acceptance:

- Documentation-only matches are downgraded or marked appropriately.
- High/critical findings are reviewable.
- Lampstand publication is redacted and local-only by default.

## Milestone 10: Agent Registry Registration

Goal: register Smart Tree as a constrained tool provider.

Deliverables:

- Registry manifest.
- Capability allow/deny list.
- Policy profile pointer.
- Trust tier: `quarantined_read_only`.

Acceptance:

- Smart Tree is not registered as a reasoning agent.
- Agents can discover read-only repo context capabilities.
- Denied capabilities are explicit.

## Milestone 11: Prophet Workspace / agent-term UX

Goal: expose the capability to operators and terminal agents without bypassing policy.

Deliverables:

- `/context snapshot`
- `/context search <query>`
- `/context symbols`
- `/context security`
- `/context lampstand-roots`
- `/context lampstand-publish`

Acceptance:

- Calls route through AgentPlane and Policy Fabric.
- Operator sees policy decision trace.
- Lampstand freshness/search context is visible.

## Milestone 12: Watch Mode and Controlled Writes

Goal: defer risky capabilities until read-only value is proven.

Rules:

- Lampstand remains watcher/reconciler.
- Smart Tree only enriches approved roots.
- Write operations require GitOps branch/patch/PR flow.
- Dashboard/PTY requires explicit security review.

Acceptance:

- No uncontrolled write path exists.
- No Smart Tree dashboard network exposure exists.
- No Smart Tree-native global memory persistence is enabled by default.

## Validation Matrix

Test against these repo types:

- Rust repo
- Python repo
- TypeScript repo
- mixed monorepo
- docs-heavy repo
- repo with suspicious MCP/hook config
- repo with large ignored directories

Required checks:

- schema validation
- policy denial behavior
- path redaction behavior
- Lampstand dry-run mapping
- Memory Mesh candidate quality
- security signal normalization

## Progress Readout

Current status after PR #1 docs additions:

- Fork hygiene / integration doctrine: 70%
- Adapter contract: 60%
- Policy profile: 60%
- Lampstand bridge: 60%
- Schema baseline: 50%
- Implementation code: 0%
- Tests: 0%
- Runtime validation: 0%

## Immediate Next Step

Implement Milestone 1 adapter skeleton and Milestone 2 policy gate before any runtime integration.
