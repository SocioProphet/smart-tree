#!/usr/bin/env python3
"""Run sourceos-context and validate live outputs against the adapter schema."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas" / "sourceos-smart-tree-adapter.v1.schema.json"
BIN_PATH = ROOT / "target" / "debug" / "sourceos-context"


def load_schema() -> dict:
    with SCHEMA_PATH.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def run_json(args: list[str], *, home: Path | None = None, expect_code: int = 0) -> dict:
    env = os.environ.copy()
    if home is not None:
        env["HOME"] = str(home)

    completed = subprocess.run(
        [str(BIN_PATH), *args],
        cwd=ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )

    if completed.returncode != expect_code:
        raise SystemExit(
            f"command failed with unexpected code {completed.returncode}; expected {expect_code}\n"
            f"args={args}\nstdout={completed.stdout}\nstderr={completed.stderr}"
        )

    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise SystemExit(
            f"command did not return valid JSON: {args}\nstdout={completed.stdout}\nstderr={completed.stderr}"
        ) from exc


def make_repo(home: Path) -> Path:
    repo = home / "dev" / "example"
    (repo / "src").mkdir(parents=True)
    (repo / "Cargo.toml").write_text(
        '[package]\nname = "example"\nversion = "0.1.0"\n',
        encoding="utf-8",
    )
    (repo / "README.md").write_text("# Example\n", encoding="utf-8")
    (repo / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
    (repo / "settings.json").write_text(
        '{"hooks":{"PreToolUse":["npx claude-flow@alpha swarm init"]}}\n',
        encoding="utf-8",
    )
    return repo


def validate(value: dict, validator: Draft202012Validator, label: str) -> None:
    errors = sorted(validator.iter_errors(value), key=lambda error: error.path)
    if errors:
        formatted = "\n".join(
            f"- {list(error.path)}: {error.message}" for error in errors
        )
        raise SystemExit(f"{label} failed schema validation:\n{formatted}\nvalue={json.dumps(value, indent=2)}")


def main() -> None:
    if not BIN_PATH.exists():
        raise SystemExit(
            f"missing {BIN_PATH}; run `cargo build --bin sourceos-context` before live validation"
        )

    schema = load_schema()
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema)

    with tempfile.TemporaryDirectory() as tmp:
        home = Path(tmp)
        repo = make_repo(home)
        outside = Path(tempfile.mkdtemp())
        (outside / "README.md").write_text("# Outside\n", encoding="utf-8")

        cases = [
            (
                "snapshot",
                run_json(["snapshot", str(repo), "--format", "json"], home=home),
            ),
            (
                "security",
                run_json(["security", str(repo), "--format", "json"], home=home),
            ),
            (
                "lampstand-publish",
                run_json(
                    ["lampstand-publish", str(repo), "--dry-run", "--format", "json"],
                    home=home,
                ),
            ),
            (
                "lampstand-roots",
                run_json(["lampstand-roots", "--format", "json"], home=home),
            ),
            (
                "policy-denied",
                run_json(["snapshot", str(outside), "--format", "json"], home=home, expect_code=2),
            ),
        ]

        for label, value in cases:
            validate(value, validator, label)

    print(f"Validated {len(cases)} live sourceos-context outputs against schema")


if __name__ == "__main__":
    main()
