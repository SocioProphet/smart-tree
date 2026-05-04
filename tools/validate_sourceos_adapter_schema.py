#!/usr/bin/env python3
"""Validate the SourceOS Smart Tree adapter JSON schema and core examples."""

from __future__ import annotations

import json
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas" / "sourceos-smart-tree-adapter.v1.schema.json"


def load_schema() -> dict:
    with SCHEMA_PATH.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def adapter_envelope(response_type: str, data: dict) -> dict:
    return {
        "schema_version": "sourceos.adapter_response.v1",
        "response_type": response_type,
        "source": "smart-tree",
        "generated_at": "2026-05-02T00:00:00Z",
        "policy_profile": "sourceos.repo_context.read_only",
        "policy_decision": {
            "decision": "allow",
            "ruleset": "sourceos.repo_context.read_only",
            "capabilities": ["repo.tree.read"],
            "redactions": [],
        },
        "provenance": {
            "adapter": "sourceos-smart-tree-adapter",
            "adapter_version": "8.0.0",
            "tool": "st",
            "tool_version": "8.0.0",
            "tool_repo": "SocioProphet/smart-tree",
            "mode": "cli",
            "upstream": "8b-is/smart-tree",
        },
        "data": data,
    }


def examples() -> list[dict]:
    return [
        adapter_envelope(
            "RepoContextSnapshot",
            {
                "schema_version": "sourceos.repo_context_snapshot.v1",
                "repo_path_ref": "~/dev/example",
                "repo_identity": {
                    "name": "example",
                    "git_remote": None,
                    "branch": None,
                    "commit": None,
                },
                "lampstand": {
                    "source_root_id": None,
                    "local_state_record_ids": [],
                    "freshness": None,
                    "publishable_records": [],
                },
                "summary": {
                    "project_type": ["rust"],
                    "languages": ["rust"],
                    "frameworks": [],
                    "build_systems": ["cargo"],
                    "test_systems": [],
                },
                "stats": {
                    "total_files": 1,
                    "total_dirs": 1,
                    "total_size_bytes": 1,
                    "scan_time_ms": None,
                    "format_time_ms": None,
                },
                "key_files": [],
                "interesting_files": [],
                "git": {},
                "security_signals": [],
                "symbol_summary": {},
                "memory_candidates": [
                    {
                        "candidate_id": "sha256:example",
                        "candidate_type": "repo_onboarding",
                        "confidence": 0.75,
                        "content": "Repo onboarding candidate.",
                        "recommended_action": "review",
                    }
                ],
            },
        ),
        adapter_envelope(
            "SecuritySignalSet",
            {
                "schema_version": "sourceos.security_signal_set.v1",
                "signals": [
                    {
                        "signal_id": "sha256:signal",
                        "path_ref": "settings.json",
                        "line": 1,
                        "pattern_name": "Auto Hook",
                        "risk_level": "high",
                        "description": "Advisory signal.",
                    }
                ],
            },
        ),
        adapter_envelope(
            "LampstandRootSet",
            {
                "schema_version": "sourceos.lampstand_root_set.v1",
                "roots": [],
                "adapter_mode": "stub",
                "notes": ["No Lampstand RPC bridge configured."],
            },
        ),
        adapter_envelope(
            "LampstandPublishReport",
            {
                "schema_version": "sourceos.lampstand_publish_report.v1",
                "dry_run": True,
                "records": [
                    {
                        "record_type": "sourceos.lampstand.repo_context_record.v1",
                        "title": "Repo context: example",
                        "object_kind": "repo_context",
                        "path_ref": "~/dev/example",
                        "handling_tags": ["local-only"],
                        "source": {"system": "sourceos-smart-tree-adapter"},
                    }
                ],
                "published_count": 0,
            },
        ),
        {
            "schema_version": "sourceos.adapter_error.v1",
            "error_code": "policy_denied",
            "message": "policy denied",
            "policy_decision": {
                "decision": "deny",
                "ruleset": "sourceos.repo_context.read_only",
                "capabilities": [],
            },
            "provenance": {
                "adapter": "sourceos-smart-tree-adapter",
                "tool": "st",
                "tool_repo": "SocioProphet/smart-tree",
                "mode": "cli",
            },
            "safe_retry": False,
        },
    ]


def main() -> None:
    schema = load_schema()
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema)

    for index, example in enumerate(examples(), start=1):
        errors = sorted(validator.iter_errors(example), key=lambda error: error.path)
        if errors:
            formatted = "\n".join(
                f"- {list(error.path)}: {error.message}" for error in errors
            )
            raise SystemExit(f"Example {index} failed schema validation:\n{formatted}")

    print(f"Validated {len(examples())} SourceOS adapter schema examples")


if __name__ == "__main__":
    main()
