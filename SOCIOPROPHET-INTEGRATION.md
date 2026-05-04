# SocioProphet / SourceOS Integration Plan for Smart Tree

## Status

Smart Tree is an upstream-derived Rust project maintained here as a strategic evaluation fork. It should be treated as a useful local context engine and codebase sensor, not as trusted SocioProphet core infrastructure yet.

This document defines the pragmatic integration path for using Smart Tree inside the SocioProphet / SourceOS agentic stack while preserving clear ownership boundaries for Memory Mesh, Sherlock, AgentPlane, Policy Fabric, Prophet Workspace, agent-term, Lampstand, and the code registry.

## Executive Value Thesis

Smart Tree's strategic value is not that it is a better `tree` command. Its strategic value is that it can become a local perception layer for agentic development work.

It can observe a repository, produce bounded context, find files, search contents, summarize structure, detect interesting files, surface security hints, expose MCP tools, run as a daemon, and extract code components for registry submission. That makes it a useful substrate for repo-aware agents.

The integration thesis is:

> Smart Tree should sense and emit structured repo/code observations. SocioProphet systems should own memory, policy, routing, interpretation, UI, durable governance, and local desktop search authority.

Smart Tree should answer:

- What is on disk inside a bounded repo/project root?
- What changed inside that bounded root?
- What matters structurally for an agent?
- What is risky in the repo/tooling surface?
- What symbols/components exist?
- What project context should an agent receive before acting?

SocioProphet systems should answer:

- What should become durable memory?
- Which agents may access or act on which information?
- What is the operational interpretation of the evidence?
- Which workflow or agent should run next?
- What should the human operator see?
- What policy gates apply?
- Which local-state records belong in Lampstand vs Memory Mesh vs Sherlock vs GAIA/OFIF?

## Recommended Disposition

Use Smart Tree. Do not trust it broadly. Do not rewrite it immediately.

The correct posture is controlled adoption through an adapter:

1. Keep this repository as an upstream-derived research fork.
2. Add a narrow SourceOS/SocioProphet adapter around read-only capabilities.
3. Enforce access through Policy Fabric.
4. Feed observations into Memory Mesh as memory candidates, not canonical memory.
5. Publish approved local search records through Lampstand/Sherlock boundaries where appropriate.
6. Let Sherlock interpret the evidence.
7. Register Smart Tree in AgentPlane and the Agent Registry as a constrained tool provider.
8. Extract or rewrite specific primitives only after usage proves which parts matter.

## What We Missed Initially

The first-order assessment treated Smart Tree primarily as a repo-context utility. That was incomplete.

The deeper value is that Smart Tree already contains several pieces of a perception-to-memory pipeline:

- scanner data structures carrying file metadata, security findings, change status, interest scores, search matches, and content hashes;
- MCP and daemon surfaces for tool access;
- MEM8 / wave-memory concepts that can be mapped into Memory Mesh candidates;
- SmartPastCode-style code component extraction;
- security scanner heuristics for MCP/hook/supply-chain risk;
- dashboard and ask-user interaction ideas that can influence Prophet Workspace and agent-term.

The additional correction: Smart Tree must integrate with Lampstand, but it must not replace Lampstand.

Lampstand is the local desktop file indexing and search service. Smart Tree is the bounded repo/code intelligence sensor. They are complementary.

## Lampstand Integration Doctrine

Lampstand is the local-state sampling, indexing, and search membrane for the GNOME/SourceOS desktop. It owns local file metadata, SQLite/FTS indexing, daemon health/stats, reconciliation state, and governed percolation of local-state records.

Smart Tree should not become the desktop search daemon. Smart Tree should feed Lampstand with repo-aware enrichments and consume Lampstand for local discovery when needed.

### Division of Responsibility

| Domain | Owner |
| --- | --- |
| Local desktop file indexing | Lampstand |
| SQLite metadata + FTS index | Lampstand |
| Inotify and reconciliation for local roots | Lampstand |
| Local file search UX | Lampstand / Sherlock Search |
| Bounded repo/code structure analysis | Smart Tree |
| Agent-oriented repo snapshots | Smart Tree adapter |
| Symbol/component extraction | Smart Tree adapter + code registry |
| Memory promotion | Memory Mesh |
| Interpretation and next actions | Sherlock |
| Tool routing | AgentPlane |
| Authorization | Policy Fabric |

### Smart Tree -> Lampstand

Smart Tree should emit repo-aware records that Lampstand can index locally:

- `RepoContextRecord`: bounded summary of a repo/project root.
- `RepoStructureRecord`: compressed tree/digest with key files and stats.
- `SymbolSearchRecord`: function/module/class/component metadata suitable for local search.
- `SecuritySearchRecord`: advisory local security findings.
- `AgentContextRecord`: agent-ready project context summary.
- `MemoryCandidateRecord`: a local record that may later be promoted by Memory Mesh.

These records should be searchable locally through Lampstand without exposing raw repo contents beyond local policy.

### Lampstand -> Smart Tree

Smart Tree should use Lampstand for local root discovery and freshness hints:

- known project roots;
- recently changed repo roots;
- local index freshness;
- file metadata and content fingerprints;
- search hits that should trigger bounded Smart Tree repo analysis;
- reconciliation/health state.

Smart Tree should not rescan the entire desktop when Lampstand already tracks local state. Instead, Lampstand identifies candidate roots and Smart Tree performs deeper repo/code analysis inside approved roots.

### Percolation Flow

```text
Local file/repo change
  -> Lampstand LocalStateRecord / LocalStateDelta
  -> Policy Fabric gate
  -> Smart Tree bounded repo/code enrichment
  -> RepoContextRecord / SymbolSearchRecord / SecuritySearchRecord
  -> Lampstand local index + Sherlock SearchRecord where approved
  -> Memory Mesh candidate promotion where approved
  -> GAIA/OFIF/Lattice Forge only through governed percolation
```

### Lampstand Safety Constraints

- Smart Tree may not bypass Lampstand's local-state authority for desktop-wide search.
- Smart Tree may only analyze repo/project roots approved by Policy Fabric.
- Lampstand records may include path references, hashes, snippets, and metadata only according to classification and handling rules.
- Smart Tree enrichments must include provenance and policy profile.
- Raw content should not percolate upward by default.
- Snippets require classification/redaction checks.

## Ownership Boundaries

### Smart Tree Owns

- Local bounded filesystem scanning.
- Repo tree and project summaries.
- File search and content search within approved roots.
- Basic git-aware project state.
- Security-scan heuristics.
- Interest scoring and change signals.
- Code/symbol extraction where supported.
- MCP/daemon access to those local observations.

### Lampstand Owns

- Desktop/local file indexing.
- SQLite metadata store and FTS5 text index.
- Inotify-driven updates and periodic reconciliation.
- Local search service boundary.
- Local health/stats and index freshness.
- Governed local-state records that can percolate upward.

### Memory Mesh Owns

- Canonical durable memory.
- Memory promotion, decay, retention, and retrieval.
- Entity linking across repos, agents, sessions, PRs, issues, symbols, and decisions.
- Provenance schema.
- Human/agent attribution.
- Cross-session recall.

### Sherlock Owns

- Interpretation of evidence.
- Repo dossiers.
- Drift detection between prior memory and current repo state.
- Next-best-action recommendations.
- Agent handoff context.
- Gap/risk analysis.

### AgentPlane Owns

- Tool routing.
- Agent/service registration.
- Capability dispatch.
- Agent execution boundaries.
- Work-order orchestration.

### Policy Fabric Owns

- Authorization.
- Path allowlists and denylists.
- Capability gates.
- Network egress rules.
- Persistence controls.
- Write controls.
- Audit and enforcement.

### Prophet Workspace / agent-term Own

- Operator UI.
- Terminal/chat interaction model.
- Agent prompts and ask-user flows.
- Repo context panels.
- Memory candidate review.
- Lampstand-backed local search views.

## Integration Architecture

```text
Lampstand local index / local-state records
  -> sourceos-smart-tree-adapter root discovery and freshness hints
    -> Smart Tree bounded repo/code scan
      -> Policy Fabric enforcement
        -> Lampstand local search records
        -> AgentPlane tool provider
        -> Sherlock interpretation
        -> Memory Mesh ingestion
        -> Prophet Workspace / agent-term presentation
        -> Code registry / evidence graph
```

The adapter is the critical control point. No downstream system should depend directly on Smart Tree's prose formats or internal terminology.

## Adapter Contract

The adapter should expose stable SourceOS commands such as:

```bash
sourceos-context snapshot ~/dev/<repo> --format json
sourceos-context search ~/dev/<repo> "query" --format json
sourceos-context symbols ~/dev/<repo> --format json
sourceos-context security ~/dev/<repo> --format json
sourceos-context changed ~/dev/<repo> --format json
sourceos-context lampstand-publish ~/dev/<repo> --format json
sourceos-context lampstand-roots --format json
```

Internally, the adapter may call Smart Tree through CLI, MCP, or daemon HTTP. Prefer the daemon path where possible because it already models structured scan requests and responses.

The adapter should normalize all output into SocioProphet-owned types:

- `RepoContextSnapshot`
- `RepoStructureDigest`
- `FileObservation`
- `SearchHit`
- `SymbolObservation`
- `SecuritySignal`
- `GitSignal`
- `LampstandLocalStateLink`
- `LampstandSearchRecord`
- `MemoryCandidate`
- `PolicyDecisionTrace`

## Initial Policy Profile

Profile name: `sourceos.repo_context.read_only`

Allowed by default:

- scan paths under `~/dev/**`;
- analyze Lampstand-discovered project roots only after policy approval;
- respect `.gitignore`;
- use default ignore rules;
- read file metadata;
- read bounded file contents for explicit search;
- collect repo statistics;
- collect git status;
- run security scanner in advisory mode;
- extract symbols/components in parse-only mode;
- emit memory candidates to Memory Mesh ingestion;
- publish local search metadata to Lampstand when classification allows.

Denied by default:

- scanning outside approved roots;
- scanning the entire home directory;
- bypassing Lampstand for desktop-wide search;
- scanning system paths;
- following symlinks;
- exposing dashboard over the network;
- spawning PTY through Smart Tree dashboard;
- installing hooks;
- mutating hooks;
- using Smart Tree smart-edit writes;
- persisting Smart Tree-native global memory;
- calling external update/feedback endpoints;
- submitting registry data to non-approved endpoints.

Escalation-only:

- hidden file reads;
- local dashboard use;
- local daemon persistence;
- continuous watch mode;
- registry submission;
- file mutation;
- branch/PR creation through downstream GitOps tools;
- snippet publication to Lampstand/Sherlock;
- upward percolation to GAIA/OFIF/Lattice Forge.

## Memory Mesh Integration

Smart Tree must not become Memory Mesh. It should produce memory candidates.

### Memory Candidate Types

#### Repo Onboarding Memory

A compact summary of what a repo is, how it is structured, key files, languages, test/build signals, security notes, Lampstand local-state links, and recommended agent entry points.

#### Work Episode Memory

A record of what changed during an agent run, linked to repo, branch, issue, PR, files, symbols, policy decisions, Lampstand records, and agent identities.

#### Symbol Memory

Extracted components such as functions, modules, classes, impl blocks, scripts, config units, and architecture docs. Each symbol should carry origin, content hash, semantic tags, language, path, clearance level, and optional Lampstand search-record linkage.

#### Security Memory

Advisory security signals from Smart Tree's scanner, linked to files, paths, patterns, policy decisions, Lampstand local-state records, and remediation state.

#### Procedural Memory

Repeatable repo-specific workflows, gotchas, setup commands, build/test commands, branch rules, and agent handoff instructions.

### Ingestion Flow

1. Lampstand identifies local state and candidate project roots.
2. Policy Fabric approves bounded roots for Smart Tree enrichment.
3. Smart Tree scans or indexes the repo.
4. Adapter normalizes the observations.
5. Policy Fabric labels sensitivity and allowed visibility.
6. Sherlock interprets and scores relevance.
7. Lampstand indexes approved local search records.
8. Memory Mesh stores promoted memories with provenance.

Smart Tree-native `.m8` files may be imported or inspected, but they are not canonical memory storage.

## Sherlock Integration

Sherlock should use Smart Tree as a live repo evidence source and Lampstand as a local-state discovery/search substrate.

Recommended loop:

1. Lampstand reports local roots, freshness, and search hits.
2. Memory Mesh retrieves what we already know about the repo.
3. Smart Tree refreshes current repo/code state.
4. Sherlock compares current state to prior memory and Lampstand freshness.
5. Sherlock emits drift, risk, gaps, next-best actions, and handoff context.
6. Memory Mesh stores the interpreted episode.
7. Lampstand indexes approved search summaries for local recall.

Initial Sherlock capabilities:

- repo onboarding dossier;
- architecture delta report;
- agent handoff context;
- security/risk report;
- next-best-action recommendation;
- branch/PR readiness report;
- memory-vs-disk drift analysis;
- Lampstand-local freshness/reconciliation report.

## AgentPlane / Agent Registry Integration

Register Smart Tree as a tool provider, not a reasoning agent.

Suggested registry entry:

```yaml
id: smart-tree-context-provider
kind: local_tool_provider
runtime: rust_binary_or_daemon
source_repo: SocioProphet/smart-tree
upstream: 8b-is/smart-tree
trust_tier: quarantined_read_only
policy_profile: sourceos.repo_context.read_only
integrates_with:
  - lampstand
  - memory-mesh
  - sherlock
  - policy-fabric
  - prophet-workspace
  - agent-term
allowed_capabilities:
  - repo.tree.read
  - repo.search.read
  - repo.stats.read
  - repo.git_status.read
  - repo.security_scan.read
  - repo.symbols.read
  - lampstand.search_record.publish.local
  - lampstand.project_root.consume
denied_capabilities:
  - repo.write
  - hooks.install
  - hooks.modify
  - dashboard.expose
  - memory.persist.native
  - network.callback
  - pty.spawn
  - desktop.search.bypass_lampstand
persistence: disabled_by_default
network: denied_by_default
owner: sourceos-agent-infrastructure
```

## SmartPastCode / Code Registry Integration

This is one of the highest-value lanes.

Smart Tree's existing SmartPastCode integration should be used as a reference implementation for code-component extraction, not necessarily as the final canonical registry protocol.

Desired canonical flow:

1. Lampstand identifies project roots and freshness.
2. Parse files safely without execution.
3. Extract functions, modules, classes, impl blocks, scripts, and configuration units.
4. Attach origin metadata: repo, commit, branch, file, line range, contributor/agent if known.
5. Attach semantic metadata: language, domain, purpose, keywords, dependencies.
6. Hash content for identity.
7. Assign clearance and policy labels.
8. Submit to the SocioProphet evidence graph / code registry.
9. Link to Memory Mesh symbol memory.
10. Publish approved local symbol search records to Lampstand.

Initial languages:

- Rust
- Python
- TypeScript / JavaScript
- Go
- Shell / Nix
- Markdown architecture docs
- YAML/TOML/JSON config later

## Policy Fabric Security Signal Integration

Smart Tree's security scanner should feed advisory signals into Policy Fabric and approved local records into Lampstand.

Examples:

- risky MCP hook detected;
- auto-executing package command detected;
- volatile npm/NPX tag detected;
- suspicious IPFS/IPNS endpoint detected;
- fake verification pattern detected;
- suspicious hidden agent directory detected;
- dependency or script pattern requiring review.

Policy Fabric decides whether to block, warn, quarantine, request review, publish local search records, promote to Memory Mesh, or ignore as documentation-only.

## Prophet Workspace Integration

Do not embed Smart Tree's dashboard directly in the first phase.

Instead, Prophet Workspace should render adapter and Lampstand outputs:

- repo context panel;
- key files panel;
- interesting files panel;
- symbol/component panel;
- security signal panel;
- Lampstand local search/freshness panel;
- Memory Mesh candidates panel;
- live scan/delta timeline;
- agent handoff summary;
- policy decision trace.

Smart Tree's `ask_user` pattern is useful, but the actual operator prompt should be implemented through Prophet Workspace / agent-term with authenticated operator controls.

## agent-term Integration

agent-term should expose repo context commands backed by AgentPlane, Lampstand, and the adapter:

```text
/context snapshot
/context search <query>
/context symbols
/context security
/context changed
/context remember
/context diff-memory
/context lampstand-roots
/context lampstand-publish
```

Agents should not call Smart Tree directly from agent-term unless routed through AgentPlane and Policy Fabric.

## Continuous Watch Mode

Watch mode is valuable later, but dangerous early.

Lampstand should remain the primary local watcher/reconciler. Smart Tree should only run deeper repo/code enrichment when Lampstand or AgentPlane requests it for approved roots.

Queued observations:

- file changed;
- key file changed;
- symbol changed;
- dependency manifest changed;
- suspicious hook/config appeared;
- architecture document changed;
- test/build file changed.

Memory Mesh promotion should be explicit, policy-gated, and deduplicated.

## Write Features

Smart Tree's smart-edit/write capabilities are quarantined initially.

If later enabled, writes must go through GitOps:

1. Create branch.
2. Apply patch.
3. Store diff.
4. Run tests/lints.
5. Open PR.
6. Require policy review.
7. Never write directly to protected branches.

## Implementation Lanes

### Lane 1: Fork Hygiene

- Add this integration plan.
- Add upstream attribution statement.
- Reconcile MIT/ISC metadata discrepancy.
- Normalize repository URLs where appropriate.
- Mark unsafe surfaces as quarantined in documentation.
- Keep upstream compatibility visible.

### Lane 2: Adapter Spec

- Define SourceOS JSON schemas.
- Define command contract.
- Define policy profile.
- Define allowed and denied capabilities.
- Define error handling and provenance envelope.
- Define Lampstand record mapping.

### Lane 3: Read-Only Snapshot

- Implement `snapshot` adapter path.
- Include repo structure, key files, stats, git signal, security hints, and Lampstand local-state links.
- Emit `RepoContextSnapshot`.
- Test on core SocioProphet / SourceOS repos.

### Lane 4: Lampstand Bridge

- Consume Lampstand project-root discovery and freshness hints.
- Publish approved Smart Tree repo summaries as Lampstand local search records.
- Publish approved symbol/security summaries as Lampstand records.
- Preserve Lampstand as the desktop/local search authority.

### Lane 5: Memory Mesh Ingestion

- Convert snapshot output into memory candidates.
- Add repo onboarding memory.
- Add work episode memory.
- Add symbol memory later.
- Ensure Memory Mesh owns persistence.
- Link memories to Lampstand local-state/search records where available.

### Lane 6: Sherlock Integration

- Add Sherlock live-state refresh using adapter output.
- Compare live state against prior Memory Mesh entries and Lampstand freshness.
- Produce drift/gap/next-action reports.

### Lane 7: Agent Registry Registration

- Register Smart Tree as constrained local tool provider.
- Add capability metadata.
- Add policy profile metadata.
- Add trust tier and denied operations.
- Add Lampstand integration capabilities.

### Lane 8: SmartPastCode / Symbol Indexing

- Adapt Rust extraction path.
- Normalize component schema.
- Add provenance and clearance.
- Feed code registry and Memory Mesh symbol memory.
- Publish approved local symbol records into Lampstand.
- Expand language support only after Rust path is stable.

### Lane 9: Security Signals

- Route scanner findings into Policy Fabric.
- Add severity normalization.
- Track false positives and documentation-only matches.
- Link findings to Lampstand records, repo memory, and remediation state.

### Lane 10: Workspace / agent-term UX

- Render context snapshots and memory candidates.
- Render Lampstand local freshness/search context.
- Add `/context` commands.
- Add operator approval prompts.
- Add policy trace view.

### Lane 11: Watch Mode and Controlled Writes

- Keep Lampstand as watcher/reconciler.
- Add Smart Tree enrichment queue for approved roots.
- Keep Memory Mesh promotion controlled.
- Only consider smart-edit writes after read-only value is proven.

## Success Metrics

- Agent repo onboarding time reduced by 50-80%.
- Fewer repeated discovery scans across sessions.
- Sherlock repo dossiers become faster and more evidence-based.
- Memory Mesh receives useful memory candidates with low noise.
- Lampstand local search improves with repo-aware summaries and symbol records.
- Security scanner catches meaningful hook/config risks.
- Symbol registry improves cross-repo reuse and agent orientation.
- No unauthorized filesystem access.
- No unapproved network egress.
- No uncontrolled Smart Tree-native memory persistence.
- No bypass of Lampstand for desktop-wide local search.

## Kill Criteria

- Output is too noisy to normalize.
- Daemon is unstable.
- Scanner is slow on core repos.
- Security surface cannot be constrained.
- Memory candidate noise pollutes Memory Mesh.
- Lampstand integration duplicates or undermines local search authority.
- Upstream maintenance cost exceeds rewrite cost.
- Code quality blocks reliable integration.

## First Implementation Target

The first usable integration should be:

> `sourceos-context snapshot ~/dev/<repo> --format json`

Second target:

> `sourceos-context lampstand-publish ~/dev/<repo> --format json`

Minimum snapshot fields:

```json
{
  "schema_version": "sourceos.repo_context_snapshot.v1",
  "source": "smart-tree",
  "repo_path": "~/dev/example",
  "policy_profile": "sourceos.repo_context.read_only",
  "provenance": {
    "tool": "st",
    "repo": "SocioProphet/smart-tree",
    "version": "8.0.0",
    "mode": "daemon_or_mcp_or_cli"
  },
  "lampstand": {
    "source_root_id": null,
    "local_state_record_ids": [],
    "freshness": null,
    "publishable_records": []
  },
  "summary": {},
  "stats": {},
  "key_files": [],
  "interesting_files": [],
  "git": {},
  "security_signals": [],
  "memory_candidates": []
}
```

## Final Decision

Smart Tree is useful enough to integrate, but only as a replaceable repo/code sensing engine behind SocioProphet-owned policy, memory, interpretation, routing, UI, and Lampstand local-state boundaries.

Use it now. Keep it constrained. Make Lampstand first-class. Measure value. Extract or rewrite later only where usage proves strategic demand.
