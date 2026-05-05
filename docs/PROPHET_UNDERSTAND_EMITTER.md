# Prophet Understand Emitter

## Purpose

`smart-tree` owns the structural scanner and deterministic graph emitter for Prophet Understand / Repo Intelligence v0.

The platform contract lives in `SocioProphet/prophet-platform`:

- `docs/PROPHET_UNDERSTAND_REPO_INTELLIGENCE.md`
- `schemas/repo-intelligence/prophet-understanding.schema.json`
- `examples/repo-intelligence/prophet-understanding.fixture.json`

## Target command

```bash
smart-tree understand --repo . --out .prophet/prophet-understanding.json
```

The command must not require network access for baseline graph emission.

## Required output

The emitter writes `.prophet/prophet-understanding.json` with:

- repo metadata: full name when known, branch, commit, generated timestamp, artifact hash
- generator metadata: smart-tree version, parser versions, ignored rules version
- nodes for repo, directory, file, module, package, service, endpoint, schema, contract, document, workflow, test, config, runtime, policy, domain, and concept where detectable
- edges for contains, imports, depends_on, defines, documents, tests, configures, calls, owns, generates, validates, governed_by, impacted_by, and related_to
- source anchors with repo-relative path, start line, end line, and content hash
- provenance receipts for every generated node, edge, summary, tour, and diff-impact set
- validation results and skipped-file receipts
- policy status placeholder in v0: allow, warn, require_review, deny, or unknown

## Determinism rules

Stable IDs are mandatory. IDs must not depend on:

- wall-clock time
- host path
- traversal order
- agent name
- local username
- generated UUIDs without deterministic input

Prefer IDs derived from repo-relative path, symbol name, node kind, and edge kind.

Output must be sorted consistently so identical commits produce equivalent graph content except for explicit generation metadata.

## Safety rules

The emitter must skip generated folders, dependency caches, build artifacts, vendored packages, lockfile-only noise, binary blobs, and secret-like paths by default. Skips must be explicit receipts, not silent omissions.

No post-commit hook should be installed by default. Hook installation requires a separate reviewed command and documented threat model.

## PR impact mode

A later v0 command should support:

```bash
smart-tree understand-diff --repo . --base <sha> --head <sha> --out .prophet/diff-impact.json
```

The diff output should map changed paths to affected nodes, edges, tests, docs, contracts, and policies.

## Acceptance criteria

- Two scans of the same commit produce stable IDs and stable sorted output.
- The artifact validates against `prophet-understanding.schema.json`.
- Every non-repo factual node has a source anchor or is marked inferred with lower confidence.
- Every graph fact carries at least one provenance receipt.
- Skipped paths are visible.
- No mutation authority is granted by graph output alone.
