#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import subprocess
import time
from pathlib import Path


def summarize(results):
    per_plan = {}
    for row in results:
        plan = row["plan"]
        if plan not in per_plan:
            per_plan[plan] = {
                "plan": plan,
                "command_count": 0,
                "pass": 0,
                "fail": 0,
                "skip": 0,
                "status": "PASS",
            }
        per_plan[plan]["command_count"] += 1
        if row.get("skipped"):
            per_plan[plan]["skip"] += 1
        elif row["success"]:
            per_plan[plan]["pass"] += 1
        else:
            per_plan[plan]["fail"] += 1
            per_plan[plan]["status"] = "FAIL"
    return [per_plan[k] for k in sorted(per_plan.keys())]


def is_placeholder_command(command):
    token = command.strip()
    if "<" in token and ">" in token:
        return True
    if "..." in token:
        return True
    if token == "cargo test/clippy":
        return True
    return False


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--matrix", default="target/ocp/plan_test_matrix_v0.1_to_v0.20.json")
    parser.add_argument("--out-dir", default="target/ocp")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    matrix_path = repo_root / args.matrix
    out_dir = repo_root / args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    ts = dt.datetime.now().strftime("%Y%m%d_%H%M%S")
    results_path = out_dir / f"plan_test_run_results_v0.1_to_v0.20_{ts}.json"
    summary_path = out_dir / f"plan_test_run_summary_v0.1_to_v0.20_{ts}.json"

    results = []
    for plan_row in matrix:
        plan = plan_row["plan"]
        commands = plan_row.get("commands", [])
        print(f"[PLAN] {plan} ({len(commands)} commands)")
        for command in commands:
            print(f"[RUN ] {command}")
            if is_placeholder_command(command):
                print("[SKIP] placeholder/non-executable")
                results.append(
                    {
                        "plan": plan,
                        "command": command,
                        "executed": "",
                        "success": True,
                        "skipped": True,
                        "exit_code": 0,
                        "duration_sec": 0.0,
                        "error": "",
                    }
                )
                continue
            t0 = time.time()
            proc = subprocess.run(
                ["powershell", "-NoProfile", "-Command", command],
                cwd=repo_root,
                text=True,
                capture_output=True,
            )
            dt_sec = round(time.time() - t0, 2)
            output = (proc.stdout or "") + ("\n" + proc.stderr if proc.stderr else "")
            err_tail = ""
            if proc.returncode != 0:
                err_tail = "\n".join(output.splitlines()[-40:])
                print(f"[FAIL] exit={proc.returncode} in {dt_sec}s")
            else:
                print(f"[PASS] in {dt_sec}s")
            results.append(
                {
                    "plan": plan,
                    "command": command,
                    "executed": command,
                    "success": proc.returncode == 0,
                    "skipped": False,
                    "exit_code": proc.returncode,
                    "duration_sec": dt_sec,
                    "error": err_tail,
                }
            )

    summary = summarize(results)
    results_path.write_text(json.dumps(results, ensure_ascii=False, indent=2), encoding="utf-8")
    summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")

    total = len(results)
    skipped = sum(1 for r in results if r.get("skipped"))
    failed = sum(1 for r in results if (not r.get("skipped")) and (not r["success"]))
    passed = total - failed - skipped
    print(f"[DONE] total={total} pass={passed} fail={failed} skip={skipped}")
    print(f"[FILE] results={results_path}")
    print(f"[FILE] summary={summary_path}")

    raise SystemExit(1 if failed else 0)


if __name__ == "__main__":
    main()
