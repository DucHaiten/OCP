#!/usr/bin/env python3
import argparse
import collections
import json
from pathlib import Path


def pick_latest_results(base: Path) -> Path:
    candidates = sorted(base.glob("plan_test_run_results_v0.1_to_v0.20_*.json"))
    if not candidates:
        raise FileNotFoundError("No results file found in target/ocp")
    return candidates[-1]


def short_error(err: str) -> str:
    lines = [line.strip() for line in (err or "").splitlines() if line.strip()]
    if not lines:
        return ""
    for line in lines:
        low = line.lower()
        if low.startswith("error:") or "failed" in low or "panicked" in low:
            return line
    return lines[-1]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--results",
        default="",
        help="Path to plan_test_run_results JSON. If empty, auto-pick latest in target/ocp.",
    )
    parser.add_argument(
        "--out",
        default="",
        help="Output markdown path. If empty, write next to results file.",
    )
    args = parser.parse_args()

    repo = Path.cwd()
    base = repo / "target" / "ocp"
    results_path = Path(args.results) if args.results else pick_latest_results(base)
    if not results_path.is_absolute():
        results_path = repo / results_path

    data = json.loads(results_path.read_text(encoding="utf-8"))
    total = len(data)
    skips = [x for x in data if x.get("skipped")]
    fails = [x for x in data if (not x.get("skipped")) and (not x.get("success"))]
    passes = total - len(skips) - len(fails)

    by_plan = collections.defaultdict(list)
    for row in fails:
        by_plan[row["plan"]].append(row)

    skip_by_plan = collections.Counter(x["plan"] for x in skips)
    fail_cmd_counter = collections.Counter(x["command"] for x in fails)

    if args.out:
        out_path = Path(args.out)
        if not out_path.is_absolute():
            out_path = repo / out_path
    else:
        stem = results_path.stem.replace("plan_test_run_results", "plan_test_fail_list")
        out_path = results_path.with_name(f"{stem}.md")

    lines = []
    lines.append("# Danh sách FAIL v0.1 -> v0.20")
    lines.append("")
    lines.append(f"- Nguồn dữ liệu: `{results_path}`")
    lines.append(f"- Tổng lệnh: `{total}`")
    lines.append(f"- PASS: `{passes}`")
    lines.append(f"- FAIL: `{len(fails)}`")
    lines.append(f"- SKIP placeholder: `{len(skips)}`")
    lines.append("")
    lines.append("## FAIL theo plan")
    lines.append("")
    lines.append("| Plan | FAIL | SKIP |")
    lines.append("|---|---:|---:|")
    all_plans = sorted({x["plan"] for x in data})
    for plan in all_plans:
        lines.append(f"| {plan} | {len(by_plan.get(plan, []))} | {skip_by_plan.get(plan, 0)} |")

    lines.append("")
    lines.append("## Top lệnh FAIL (theo tần suất)")
    lines.append("")
    lines.append("| Count | Command |")
    lines.append("|---:|---|")
    for command, count in fail_cmd_counter.most_common():
        cmd = command.replace("|", "\\|")
        lines.append(f"| {count} | `{cmd}` |")

    lines.append("")
    lines.append("## Chi tiết từng FAIL")
    lines.append("")
    for plan in sorted(by_plan.keys()):
        rows = by_plan[plan]
        lines.append(f"### {plan} ({len(rows)} fail)")
        lines.append("")
        for idx, row in enumerate(rows, 1):
            hint = short_error(row.get("error", ""))
            lines.append(f"{idx}. `{row['command']}`")
            lines.append(f"   - exit: `{row.get('exit_code')}`")
            if hint:
                lines.append(f"   - hint: `{hint}`")
        lines.append("")

    if skips:
        lines.append("## Placeholder bị SKIP")
        lines.append("")
        for idx, row in enumerate(skips, 1):
            lines.append(f"{idx}. `{row['plan']}` - `{row['command']}`")
        lines.append("")

    out_path.write_text("\n".join(lines), encoding="utf-8")
    print(out_path)
    print(f"total={total} pass={passes} fail={len(fails)} skip={len(skips)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
