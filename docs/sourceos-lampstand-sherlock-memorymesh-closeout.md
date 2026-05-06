# SourceOS Smart Tree → Lampstand → Sherlock → Memory Mesh Closeout

Date: 2026-05-06
Status: stable first-phase integration complete

## Purpose

This document closes the first integration arc for Smart Tree as a bounded SourceOS repo/code context sensor that feeds Lampstand local search, Sherlock evidence search, and Memory Mesh review-only promotion packets.

This is not full autonomous memory writeback. It is a governed local evidence pipeline with durable-memory promotion still under Memory Mesh review.

## Merged integration path

```text
Smart Tree / sourceos-context
  -> bounded repo snapshot / advisory security scan
  -> governed Lampstand adapter records
  -> Sherlock local evidence search
  -> Memory Mesh review-only promotion packet
```

## Repositories and merged state

### SocioProphet/smart-tree

Smart Tree now contains the SourceOS adapter baseline.

Merged capabilities:

- `sourceos-context snapshot <repo> --format json`
- `sourceos-context security <repo> --format json`
- `sourceos-context lampstand-roots --format json [--socket <path>]`
- `sourceos-context lampstand-publish <repo> --format json`
- `sourceos-context lampstand-publish <repo> --publish --socket <path> --format json`

Validation coverage:

- schema example validation;
- live adapter output validation;
- allowed `~/dev/**` repo scans;
- denied outside-root scans;
- denied unbounded home-root scans;
- denied symlink-root scans;
- Lampstand `RootHints` unavailable and success paths;
- Lampstand `PublishAdapterRecords` unavailable and success paths;
- security finding redaction.

Boundary:

- default publish behavior remains dry-run;
- real local ingestion requires explicit `--publish`;
- no hooks, PTY, dashboard exposure, external callbacks, smart-edit writes, or Smart Tree-native global memory persistence are enabled.

### SocioProphet/lampstand

Lampstand now owns both local root discovery and governed local adapter-record storage.

Merged capabilities:

- `RootHints` RPC;
- `lampstand roots` CLI;
- `AdapterRecordStore` with separate `adapter_records` and `adapter_records_fts` tables;
- `PublishAdapterRecords` RPC;
- `QueryAdapterRecords` RPC;
- `AdapterRecordStats` RPC;
- `lampstand adapter-records-publish <payload.json|-> [--dry-run]`;
- `lampstand adapter-records-query <query>`;
- `lampstand adapter-records-stats`.

Boundary:

- adapter records are local search summaries/signals/candidates, not canonical filesystem truth;
- canonical file indexing remains separate;
- no external network calls are introduced;
- idempotent record IDs prevent duplicate local records.

### SocioProphet/sherlock-search

Sherlock can now search Lampstand adapter records as local evidence.

Merged capabilities:

- `tools/search_lampstand_adapter_records.py`;
- `tools/smoke_lampstand_adapter_records_search.py`;
- `lampstand-adapter-record-search` workflow.

Boundary:

- Sherlock consumes Lampstand adapter-record evidence;
- Sherlock does not bypass Lampstand;
- Sherlock does not create a second local index;
- Sherlock preserves `policy_decision`, `source`, `classification`, `handling_tags`, and evidence refs;
- no semantic/vector certainty is claimed in this lane.

### SocioProphet/memory-mesh

Memory Mesh now has a review-only promotion-packet contract for Lampstand adapter records.

Merged capabilities:

- `schemas/lampstand-adapter-record-promotion-packet.schema.json`;
- `examples/lampstand/adapter-record-promotion-packet.example.json`;
- `scripts/validate_lampstand_adapter_record_promotion_packet.py`;
- `lampstand-adapter-record-promotion-packet` workflow.

Boundary:

- promotion packets are review-only by default;
- durable memory writeback is not automatic;
- every promotion candidate must reference an included Lampstand record;
- packet examples preserve policy decision refs and evidence refs.

## What is now done

The first-phase local context/search/memory-candidate bridge is complete.

Done means:

- Smart Tree can generate bounded repo context and advisory security signals.
- Smart Tree can publish governed records into Lampstand when explicitly asked.
- Lampstand can store, query, and report stats for those governed records.
- Sherlock can search those records as local evidence while preserving provenance and policy state.
- Memory Mesh can validate review-only promotion packets from those records.

## What remains intentionally deferred

The following are still out of scope until separately reviewed:

- automatic Memory Mesh durable writeback;
- direct Smart Tree memory persistence;
- arbitrary Lampstand writes outside the governed adapter-record store;
- raw file-content publication into Lampstand;
- Symbol / SmartPastCode runtime extraction;
- watch mode;
- AgentPlane runtime registration and dispatch;
- Prophet Workspace UI surfaces;
- agent-term commands;
- Policy Fabric external service integration;
- semantic/vector search certainty;
- dashboard, PTY, hooks, smart-edit writes, and external callbacks.

## Final phase-one completion readout

- Smart Tree bounded context adapter: 85%
- Lampstand root discovery: 95%
- Lampstand governed record publishing: 90%
- Sherlock adapter-record evidence search: 85%
- Memory Mesh review-only promotion packets: 85%
- Cross-repo governance boundary: 90%
- End-to-end runtime automation: 45%

The integration is stable enough to stop the first phase and call it done.

## Recommended next phase

Next work should not expand the bridge blindly. The next phase should be one of these, in order:

1. AgentPlane registration for `sourceos-context` as a constrained local tool provider.
2. Policy Fabric externalization for `sourceos.repo_context.read_only`.
3. Memory Mesh approval workflow for converting review-only promotion packets into explicit durable writeback events.
4. Symbol extraction / SmartPastCode lane only after parse-only boundaries and schema fixtures are reviewed.
