#!/usr/bin/env python3
import argparse
import json
import re
from pathlib import Path


PLAN_DIR = Path("docs/plans/history")
OUT = Path("target/ocp/plan_test_matrix_v0.1_to_v0.20_refreshed.json")
PLAN_NAME_RE = re.compile(r"^OCP-MVP-PLAN-v0\.(\d+)(?:\.(\d+))?\.md$")


def plan_sort_key(path: Path):
    m = PLAN_NAME_RE.match(path.name)
    if not m:
        return (999, 999, path.name)
    major = int(m.group(1))
    patch = int(m.group(2)) if m.group(2) else 0
    return (major, patch, path.name)


def is_command(line: str) -> bool:
    x = line.strip()
    if not x or x.startswith("#"):
        return False
    if x.startswith("$ "):
        x = x[2:].strip()
    prefixes = (
        "cargo ",
        "$env:",
        "set ",
        "python ",
        "pytest ",
        "npm ",
        "pnpm ",
        "yarn ",
        "powershell ",
        "pwsh ",
        "bash ",
        "./",
    )
    return x.startswith(prefixes)


def normalize(line: str) -> str:
    x = line.strip()
    if x.startswith("$ "):
        x = x[2:].strip()
    return x


BACKTICK_RE = re.compile(r"`([^`]+)`")


def extract_commands_anywhere(text: str):
    cmds = []
    for raw in text.splitlines():
        line = raw.rstrip()
        stripped = line.strip()

        # Inline command in backticks: - `cargo test`
        for token in BACKTICK_RE.findall(line):
            if is_command(token):
                cmds.append(normalize(token))

        # Bullet/plain command without backticks.
        if stripped.startswith("-"):
            body = stripped[1:].strip()
            if is_command(body):
                cmds.append(normalize(body))
        elif is_command(stripped):
            cmds.append(normalize(stripped))
    return cmds


def extract_commands_from_commands_run_blocks(text: str):
    cmds = []
    in_commands_block = False
    for raw in text.splitlines():
        line = raw.rstrip()
        stripped = line.strip()
        lower = stripped.lower()

        if not in_commands_block:
            if lower == "commands run:" or lower == "- commands run:":
                in_commands_block = True
            continue

        if (
            lower.startswith("test results:")
            or lower.startswith("- test results:")
            or lower.startswith("notes/risks:")
            or lower.startswith("- notes/risks:")
            or stripped.startswith("### ")
            or stripped == "---"
        ):
            in_commands_block = False
            continue

        for token in BACKTICK_RE.findall(line):
            if is_command(token):
                cmds.append(normalize(token))

        if stripped.startswith("-"):
            body = stripped[1:].strip()
            if is_command(body):
                cmds.append(normalize(body))
        elif is_command(stripped):
            cmds.append(normalize(stripped))
    return cmds


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--mode",
        choices=("commands-run", "all"),
        default="commands-run",
        help="commands-run: only parse under 'Commands run:' blocks; all: parse all lines",
    )
    args = parser.parse_args()

    plans = sorted(PLAN_DIR.glob("OCP-MVP-PLAN-v0*.md"), key=plan_sort_key)
    matrix = []
    for plan in plans:
        text = plan.read_text(encoding="utf-8")
        if args.mode == "all":
            commands = extract_commands_anywhere(text)
        else:
            commands = extract_commands_from_commands_run_blocks(text)
        matrix.append(
            {
                "plan": plan.name,
                "command_count": len(commands),
                "commands": commands,
            }
        )
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(matrix, ensure_ascii=False, indent=2), encoding="utf-8")
    print(OUT)
    print(len(matrix))
    print(sum(x["command_count"] for x in matrix))


if __name__ == "__main__":
    main()
