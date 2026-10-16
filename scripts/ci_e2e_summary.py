#!/usr/bin/env python3
"""Summarise an e2e CI run as markdown (mission branches only).

usage: ci_e2e_summary.py <playwright.json> <server.log> <run id> <sha>

Prints the failing (and flaky) Playwright tests with their first error lines,
and the server-log lines about stalled requests, panics and errors. Never
prints the environment.
"""

import json
import re
import sys

ANSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")
LOG_PATTERN = re.compile(r"slow request|inflight|panicked|ERROR")
MAX_LOG_LINES = 200
MAX_ERROR_LINES = 20


def walk(suite, path):
    title = suite.get("title") or ""
    here = path + ([title] if title else [])
    for spec in suite.get("specs", []):
        yield here, spec
    for child in suite.get("suites", []):
        yield from walk(child, here)


def main():
    report_path, log_path, run_id, sha = sys.argv[1:5]
    out = [f"# CI e2e summary", "", f"- run: {run_id}", f"- sha: {sha}", ""]

    out.append("## Playwright")
    out.append("")
    try:
        with open(report_path, encoding="utf-8") as fh:
            report = json.load(fh)
    except (OSError, ValueError) as err:
        report = None
        out.append(f"No JSON report ({err.__class__.__name__}): the Playwright step did not run or crashed.")
    if report is not None:
        stats = report.get("stats", {})
        out.append(
            "expected {e}, unexpected {u}, flaky {f}, skipped {s}".format(
                e=stats.get("expected"), u=stats.get("unexpected"),
                f=stats.get("flaky"), s=stats.get("skipped"),
            )
        )
        for err in report.get("errors", []):
            out.append("")
            out.append("Global error:")
            out.append("```")
            out.extend(ANSI.sub("", err.get("message", "")).splitlines()[:MAX_ERROR_LINES])
            out.append("```")
        for suite in report.get("suites", []):
            for path, spec in walk(suite, []):
                for test in spec.get("tests", []):
                    status = test.get("status")
                    if status not in ("unexpected", "flaky"):
                        continue
                    out.append("")
                    out.append(f"### [{status}] {spec.get('file')}:{spec.get('line')} {' > '.join(path[1:] + [spec.get('title', '')])}")
                    for i, res in enumerate(test.get("results", [])):
                        if res.get("status") in ("passed", "skipped"):
                            continue
                        msg = (res.get("error") or {}).get("message") or ""
                        if not msg and res.get("errors"):
                            msg = res["errors"][0].get("message", "")
                        out.append(f"attempt {i + 1}: {res.get('status')} ({res.get('duration')} ms)")
                        out.append("```")
                        out.extend(ANSI.sub("", msg).splitlines()[:MAX_ERROR_LINES])
                        out.append("```")

    out.append("")
    out.append("## Server log (slow request, inflight, panicked, ERROR)")
    out.append("")
    try:
        with open(log_path, encoding="utf-8", errors="replace") as fh:
            lines = [ANSI.sub("", l.rstrip("\n")) for l in fh if LOG_PATTERN.search(l)]
    except OSError as err:
        lines = [f"(no server log: {err.__class__.__name__})"]
    out.append(f"{len(lines)} matching lines" + (f", first {MAX_LOG_LINES} shown" if len(lines) > MAX_LOG_LINES else ""))
    out.append("```")
    out.extend(line[:1000] for line in lines[:MAX_LOG_LINES])
    out.append("```")
    print("\n".join(out))


if __name__ == "__main__":
    main()
