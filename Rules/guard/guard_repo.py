#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Iterable


REPO_ROOT = Path(__file__).resolve().parents[2]

PLAN_FILE_RE = re.compile(r"^OCP-OCL-MVP-PLAN-v\d+(\.\d+)?\.md$", re.IGNORECASE)

# Keep only high-signal mojibake tokens; avoid broad tokens that can hit valid Vietnamese text.
MOJIBAKE_PATTERNS = [
    "NgÃ",
    "Tráº",
    "Má»¥c tiÃªu",
    "khÃ´ng",
    "Ä‘",
    "â€”",
    "Ã¡",
]

PLAN_REQUIRED_ALIASES = {
    "Template cập nhật kế hoạch": ["Template cập nhật kế hoạch"],
    "Template cập nhật triển khai": ["Template cập nhật triển khai"],
    "Trạng thái": ["Trạng thái"],
    "Operational commands": [
        "Operational commands",
        "Operational Commands",
        "Lệnh mới",
        "Lệnh kiểm chứng",
    ],
}

CLOSEOUT_REQUIRED_FIELDS = [
    "- Date:",
    "- Gate/Step:",
    "- Implemented:",
    "- Files changed:",
    "- Commands run:",
    "- Test results:",
    "- Notes/risks:",
]

DONE_STATUS_RE = re.compile(
    r"^\s*-\s*([A-Za-z]\d+(?:-[A-Za-z0-9._]+)+)\b.*:\s*`DONE`",
    re.MULTILINE,
)

CONTROL_DOC_RELS = {
    "README.md",
    "AGENTS.md",
    "Rules/AGENTS.md",
    "Rules/README.md",
    "Rules/COVERAGE.md",
}

GENERATED_PATH_MARKERS = (
    "/target/",
    "\\target\\",
    "/.fingerprint/",
    "\\.fingerprint\\",
)

GENERATED_NAME_SET = {
    ".rustc_info.json",
    "invoked.timestamp",
}

GENERATED_EXT_SET = {
    ".exe",
    ".pdb",
    ".dll",
    ".so",
    ".dylib",
    ".o",
    ".obj",
    ".rlib",
    ".rmeta",
    ".a",
    ".lib",
    ".pyc",
    ".pyo",
}


def run_git(*args: str) -> str:
    proc = subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or f"git {' '.join(args)} failed")
    return proc.stdout


def changed_files(staged: bool) -> list[Path]:
    args = ["diff", "--name-only", "--diff-filter=ACMRTUXB"]
    if staged:
        args.insert(1, "--cached")
    out = run_git(*args).strip()
    if not out:
        return []
    files: list[Path] = []
    for raw in out.splitlines():
        p = raw.strip()
        if not p:
            continue
        files.append(REPO_ROOT / p.replace("/", os.sep))
    return files


def all_candidate_files() -> list[Path]:
    out = run_git("ls-files").strip()
    files: list[Path] = []
    for raw in out.splitlines():
        p = raw.strip()
        if not p:
            continue
        files.append(REPO_ROOT / p.replace("/", os.sep))
    return files


def is_plan_file(path: Path) -> bool:
    return PLAN_FILE_RE.match(path.name) is not None


def is_agents_file(path: Path) -> bool:
    rel = path.relative_to(REPO_ROOT).as_posix()
    return rel == "AGENTS.md" or rel == "Rules/AGENTS.md"


def is_control_doc_file(path: Path) -> bool:
    rel = path.relative_to(REPO_ROOT).as_posix()
    if rel in CONTROL_DOC_RELS:
        return True
    if is_plan_file(path):
        return True
    return False


def is_generated_artifact_path(path: Path) -> bool:
    rel = path.relative_to(REPO_ROOT).as_posix()
    rel_lower = rel.lower()
    for marker in GENERATED_PATH_MARKERS:
        if marker.lower() in rel_lower:
            return True
    if path.name in GENERATED_NAME_SET:
        return True
    if path.suffix.lower() in GENERATED_EXT_SET:
        return True
    return False


def read_utf8_strict(path: Path) -> str:
    data = path.read_bytes()
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ValueError(f"{path}: not UTF-8 ({exc})") from exc


def check_mojibake(path: Path, text: str) -> list[str]:
    errors: list[str] = []
    for pattern in MOJIBAKE_PATTERNS:
        if pattern in text:
            errors.append(f"{path}: mojibake pattern detected: `{pattern}`")
    return errors


def extract_blocks(lines: list[str], heading_pattern: re.Pattern[str]) -> list[tuple[int, int]]:
    idxs = [i for i, line in enumerate(lines) if heading_pattern.search(line)]
    blocks: list[tuple[int, int]] = []
    for start in idxs:
        end = len(lines)
        for j in range(start + 1, len(lines)):
            if re.match(r"^#{2,}\s", lines[j]):
                end = j
                break
        blocks.append((start, end))
    return blocks


def check_closeout_block_evidence(path: Path, title: str, block: str) -> list[str]:
    errors: list[str] = []
    commands_match = re.search(
        r"-\s*Commands run:\s*(.*?)(?:\n-\s*[A-Za-z][^:\n]*:|\Z)",
        block,
        flags=re.IGNORECASE | re.DOTALL,
    )
    if commands_match is not None:
        if re.search(r"`[^`]+`", commands_match.group(1)) is None:
            errors.append(f"{path}: `{title}` missing command literals in `Commands run`")

    tests_match = re.search(
        r"-\s*Test results:\s*(.*?)(?:\n-\s*[A-Za-z][^:\n]*:|\Z)",
        block,
        flags=re.IGNORECASE | re.DOTALL,
    )
    if tests_match is not None:
        if re.search(r"\bPASS\b|\bFAIL\b", tests_match.group(1), flags=re.IGNORECASE) is None:
            errors.append(
                f"{path}: `{title}` must include PASS/FAIL in `Test results`"
            )
    return errors


def check_done_gates_have_entries(path: Path, text: str) -> list[str]:
    errors: list[str] = []
    done_gates = sorted(set(DONE_STATUS_RE.findall(text)))
    lines = text.splitlines()
    closeout_blocks = extract_blocks(
        lines,
        re.compile(
            r"^#{3,}\s+.*(implementation closeout|completion closeout|closeout)",
            re.IGNORECASE,
        ),
    )
    closeout_texts = ["\n".join(lines[start:end]) for start, end in closeout_blocks]
    for gate in done_gates:
        has_plan = re.search(
            rf"{re.escape(gate)}.*planning freeze", text, flags=re.IGNORECASE
        ) is not None
        if not has_plan:
            errors.append(f"{path}: gate `{gate}` = DONE but missing Planning Freeze entry")

        gate_closeouts = [
            block for block in closeout_texts if re.search(rf"\b{re.escape(gate)}\b", block, re.IGNORECASE)
        ]
        if not gate_closeouts:
            errors.append(
                f"{path}: gate `{gate}` = DONE but missing Implementation Closeout entry"
            )
            continue
        has_pass = any(
            re.search(r"\bPASS\b", block, flags=re.IGNORECASE) is not None
            for block in gate_closeouts
        )
        if not has_pass:
            errors.append(
                f"{path}: gate `{gate}` = DONE but closeout has no PASS evidence"
            )
    return errors


def check_plan_structure(path: Path, text: str) -> list[str]:
    errors: list[str] = []
    lower = text.lower()
    for marker, aliases in PLAN_REQUIRED_ALIASES.items():
        if not any(alias.lower() in lower for alias in aliases):
            errors.append(f"{path}: missing required marker `{marker}`")

    if re.search(r"planning freeze", text, flags=re.IGNORECASE) is None:
        errors.append(f"{path}: missing any Planning Freeze section")

    closeout_heading_re = re.compile(
        r"^#{3,}\s+.*(implementation closeout|completion closeout|closeout)",
        re.IGNORECASE | re.MULTILINE,
    )
    if closeout_heading_re.search(text) is None:
        errors.append(f"{path}: missing any Closeout section")

    lines = text.splitlines()
    closeout_blocks = extract_blocks(
        lines,
        re.compile(
            r"^#{3,}\s+.*(implementation closeout|completion closeout|closeout)",
            re.IGNORECASE,
        ),
    )
    if not closeout_blocks:
        errors.append(f"{path}: no closeout blocks found")
    else:
        for start, end in closeout_blocks:
            block = "\n".join(lines[start:end])
            title = lines[start].strip()
            for field in CLOSEOUT_REQUIRED_FIELDS:
                if field not in block:
                    errors.append(f"{path}: `{title}` missing field `{field}`")
            errors.extend(check_closeout_block_evidence(path, title, block))

    errors.extend(check_done_gates_have_entries(path, text))
    return errors


def check_agents_structure(path: Path, text: str) -> list[str]:
    errors: list[str] = []
    required = [
        "## 0)",
        "## 1)",
        "## 2)",
        "## 3)",
        "## 4)",
        "## 5)",
        "## 6)",
        "## 7)",
        "## 8)",
        "## 9)",
        "## 10)",
    ]
    for marker in required:
        if marker not in text:
            errors.append(f"{path}: missing section marker `{marker}`")
    if "CẤM TUYỆT ĐỐI" not in text and "Cấm tuyệt đối" not in text:
        errors.append(f"{path}: missing strict prohibition section")
    return errors


def check_large_plan_rewrite(path: Path) -> list[str]:
    if os.environ.get("OCL_RULES_ALLOW_LARGE_DOC", "").strip() == "1":
        return []
    rel = path.relative_to(REPO_ROOT).as_posix()
    try:
        out = run_git("diff", "--cached", "--numstat", "--", rel).strip()
    except RuntimeError:
        return []
    if not out:
        return []
    line = out.splitlines()[0].split("\t")
    if len(line) < 2:
        return []
    try:
        add = int(line[0]) if line[0].isdigit() else 0
        delete = int(line[1]) if line[1].isdigit() else 0
    except ValueError:
        return []
    threshold = 250
    if add > threshold or delete > threshold:
        return [
            (
                f"{path}: large plan rewrite detected (+{add}/-{delete}). "
                "Use smaller apply_patch hunks or set OCL_RULES_ALLOW_LARGE_DOC=1 for intentional large updates."
            )
        ]
    return []


def check_change_scope(candidates: Iterable[Path]) -> list[str]:
    errors: list[str] = []
    if os.environ.get("OCL_RULES_ALLOW_GENERATED", "").strip() == "1":
        return errors
    for path in candidates:
        if not path.exists():
            continue
        if not path.is_file():
            continue
        if is_generated_artifact_path(path):
            errors.append(
                (
                    f"{path}: generated/build artifact is staged. "
                    "Unstage build outputs (target, .exe, .pdb, fingerprints)."
                )
            )
    return errors


def unique_existing(paths: Iterable[Path]) -> list[Path]:
    seen: set[Path] = set()
    out: list[Path] = []
    for p in paths:
        if p in seen:
            continue
        seen.add(p)
        if p.exists() and p.is_file():
            out.append(p)
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="OCP-OCL Rules Guard")
    parser.add_argument(
        "--mode",
        choices=["changed", "all"],
        default="changed",
        help="scan changed files or all tracked files",
    )
    parser.add_argument(
        "--staged",
        action="store_true",
        help="when mode=changed, scan staged diff instead of working tree diff",
    )
    args = parser.parse_args()

    try:
        if args.mode == "all":
            candidates = all_candidate_files()
        else:
            candidates = changed_files(staged=args.staged)
    except RuntimeError as exc:
        print(f"[rules] guard cannot read git state: {exc}", file=sys.stderr)
        return 2

    errors: list[str] = []
    if args.mode == "changed":
        errors.extend(check_change_scope(candidates))

    targets = unique_existing(
        p for p in candidates if is_plan_file(p) or is_agents_file(p)
    )

    if not targets and not errors:
        print("[rules] no target files to validate")
        return 0

    for path in targets:
        try:
            text = read_utf8_strict(path)
        except ValueError as exc:
            errors.append(str(exc))
            continue

        errors.extend(check_mojibake(path, text))

        if is_plan_file(path):
            errors.extend(check_plan_structure(path, text))
            errors.extend(check_large_plan_rewrite(path))
        elif is_agents_file(path):
            errors.extend(check_agents_structure(path, text))

    if errors:
        print("[rules] FAIL", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(f"[rules] PASS ({len(targets)} file(s) validated)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
