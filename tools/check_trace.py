#!/usr/bin/env python3
"""Traceability check: code -> rule -> evidence.

Fails when
  * source code cites a rule ID (FN-.../H-...) that docs/FORMAT_NOTES.md lacks,
  * a rule in FORMAT_NOTES.md has no "Status:" or no "Evidence:" line,
  * an EXP-NNNN cited in FORMAT_NOTES.md has no research/EXP-NNNN-*.md record.
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
NOTES = ROOT / "docs" / "FORMAT_NOTES.md"
RESEARCH = ROOT / "research"
SOURCES = sorted((ROOT / "crates").rglob("*.rs"))

RULE_ID = re.compile(r"\b(?:FN|H)-[A-Z]+[0-9]+[a-z]?\b")
HEADING = re.compile(r"^### ((?:FN|H)-[A-Z]+[0-9]+[a-z]?)\b")
EXP_ID = re.compile(r"\bEXP-(\d{4})\b")


def parse_rules(text: str) -> dict[str, str]:
    rules: dict[str, str] = {}
    current = None
    for line in text.splitlines():
        match = HEADING.match(line)
        if match:
            current = match.group(1)
            if current in rules:
                raise SystemExit(f"duplicate rule {current} in {NOTES.name}")
            rules[current] = ""
        elif line.startswith("## "):
            current = None
        elif current:
            rules[current] += line + "\n"
    return rules


def main() -> int:
    errors: list[str] = []
    notes = NOTES.read_text(encoding="utf-8")
    rules = parse_rules(notes)

    for rule, body in rules.items():
        if "Status:" not in body:
            errors.append(f"{rule}: no Status line")
        if "Evidence:" not in body:
            errors.append(f"{rule}: no Evidence line")

    cited: dict[str, list[str]] = {}
    for source in SOURCES:
        for number, line in enumerate(source.read_text(encoding="utf-8").splitlines(), 1):
            for rule in RULE_ID.findall(line):
                cited.setdefault(rule, []).append(f"{source.relative_to(ROOT)}:{number}")
    for rule, places in sorted(cited.items()):
        if rule not in rules:
            errors.append(f"{rule} cited at {places[0]} but missing from FORMAT_NOTES.md")

    records = {p.name[:8] for p in RESEARCH.glob("EXP-*.md")}
    for number in sorted(set(EXP_ID.findall(notes))):
        if f"EXP-{number}" not in records:
            errors.append(f"EXP-{number} cited in FORMAT_NOTES.md but no research record")

    uncited = sorted(set(rules) - set(cited))
    print(f"{len(rules)} rules, {len(cited)} cited in code, {len(uncited)} documentation-only")
    if errors:
        print("\n".join(f"error: {e}" for e in errors))
        return 1
    print("traceability check passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
