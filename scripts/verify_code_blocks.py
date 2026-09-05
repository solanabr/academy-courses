#!/usr/bin/env python3
"""Verify graded code blocks that content-lint's gate 6 defers.

Gate 6 executes buildType-standard TypeScript blocks in CI and defers Rust and
buildable-TS blocks to runtime grading (fail-closed). This script closes that
gap in this repo's own CI: for every deferred block it asserts the repo rule —
the solution passes every test in tests.json and the starter fails at least one
(a starter that does not compile also satisfies "fails").

Usage:
    verify_code_blocks.py <course-dir> [<course-dir> ...] [--only SUBSTR] [--work DIR]

Exit codes: 0 = all deferred blocks verified, 1 = >=1 violation, 2 = usage/env.

Rust harness model: a challenge submission is a file defining one or more free
functions; tests.json rows are {id, input, expectedOutput} where `input` is a
verbatim Rust argument list (`vec![100, 50], 75, 3` / `"a", "b"` / `[7u8; 32]`).
The harness appends a generated main() that pastes that list into a call to the
entry function once per test and prints TEST:<id>:<value>. The entry function is
the LAST-defined top-level fn (helpers precede the graded fn by convention).
String expectedOutput values carry their own surrounding quotes; they are
stripped before comparison, matching Display-formatted output.
"""
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover
    print("verify_code_blocks: pyyaml is required (pip install pyyaml)", file=sys.stderr)
    sys.exit(2)

RUN_TIMEOUT = 30
CARGO_ADD_MAX = 6
# Versions a challenge may pull in, pinned so CI matches the graders.
KNOWN_CRATES = {
    "serde": "1",
    "serde_json": "1",
    "anchor-lang": "=1.1.2",
    "thiserror": "1",
}

FN_RE = re.compile(r"^\s*(?:pub\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(", re.M)


def pick_entry_fn(src: str):
    """Last-defined top-level fn that isn't main — helpers come first by convention."""
    names = [m.group(1) for m in FN_RE.finditer(src) if m.group(1) != "main"]
    return names[-1] if names else None


def build_rust_harness(src: str, tests: list, debug_fmt: bool):
    """tests.json `input` is a verbatim Rust argument list (`vec![1, 2], 75, 3`,
    `"a", "b"`, `[7u8; 32], ...`) — paste it directly into the call."""
    name = pick_entry_fn(src)
    if name is None:
        raise ValueError("no callable top-level fn found")
    fmt = "{:?}" if debug_fmt else "{}"
    calls = [
        f'    println!("TEST:{t["id"]}:{fmt}", {name}({t.get("input", "")}));'
        for t in tests
    ]
    main = "\n#[allow(clippy::all)]\nfn main() {\n" + "\n".join(calls) + "\n}\n"
    return src + main


def cargo_check_loop(crate: Path, subject_pins: dict):
    """cargo check, iteratively cargo-adding crates the submission imports."""
    for _ in range(CARGO_ADD_MAX):
        proc = subprocess.run(
            ["cargo", "check", "--quiet"], cwd=crate, capture_output=True, text=True, timeout=600
        )
        if proc.returncode == 0:
            return True, ""
        missing = set(re.findall(r"(?:unresolved import|can't find crate for|use of unresolved module or unlinked crate) `?([a-z_][a-z0-9_-]*)`?", proc.stderr))
        missing = {m.replace("_", "-") for m in missing} - {"crate", "self", "super", "std", "core"}
        if not missing:
            return False, proc.stderr
        for crate_name in sorted(missing):
            pin = subject_pins.get(crate_name) or KNOWN_CRATES.get(crate_name)
            spec = f"{crate_name}@{pin}" if pin else crate_name
            add = subprocess.run(["cargo", "add", spec, "--quiet"], cwd=crate, capture_output=True, text=True)
            if add.returncode != 0:
                return False, proc.stderr + "\n" + add.stderr
    return False, "dependency resolution did not converge"


def run_rust_submission(work: Path, tag: str, src: str, tests: list, subject_pins: dict):
    """Returns (compiled, results dict id->output or None, stderr)."""
    crate = work / tag
    if crate.exists():
        shutil.rmtree(crate)
    subprocess.run(["cargo", "init", "--name", "challenge", "--vcs", "none", str(crate)], capture_output=True, check=True)
    shared_target = work / "target"
    (crate / ".cargo").mkdir(exist_ok=True)
    (crate / ".cargo" / "config.toml").write_text(f'[build]\ntarget-dir = "{shared_target}"\n')
    for debug_fmt in (False, True):
        try:
            harness = build_rust_harness(src, tests, debug_fmt)
        except ValueError as e:
            return False, None, f"harness: {e}"
        (crate / "src" / "main.rs").write_text(harness)
        ok, err = cargo_check_loop(crate, subject_pins)
        if not ok:
            if debug_fmt:
                return False, None, err
            # Display may be the only problem; retry with Debug formatting.
            if "std::fmt::Display" in err or "doesn't implement" in err:
                continue
            return False, None, err
        try:
            proc = subprocess.run(["cargo", "run", "--quiet"], cwd=crate, capture_output=True, text=True, timeout=RUN_TIMEOUT)
        except subprocess.TimeoutExpired:
            return True, {}, "run timeout"
        results = {}
        for line in proc.stdout.splitlines():
            if line.startswith("TEST:"):
                _, tid, val = line.split(":", 2)
                if debug_fmt and len(val) >= 2 and val[0] == '"' and val[-1] == '"':
                    val = val[1:-1]
                results[tid] = val
        return True, results, proc.stderr[-2000:]
    return False, None, "unreachable"


def grade(results, tests):
    """Returns (passed_ids, failed_ids). Missing output counts as failed."""
    passed, failed = [], []
    for t in tests:
        got = (results or {}).get(t["id"])
        want = str(t["expectedOutput"]).strip()
        if len(want) >= 2 and want[0] == '"' and want[-1] == '"':
            want = want[1:-1]
        if got is not None and got.strip() == want:
            passed.append(t["id"])
        else:
            failed.append(t["id"])
    return passed, failed


def verify_course(course_dir: Path, only: str, work: Path):
    course_yaml = course_dir / "course.yaml"
    subject_pins = {}
    if course_yaml.exists():
        cy = yaml.safe_load(course_yaml.read_text()) or {}
        sv = cy.get("subjectVersion", "")
        if isinstance(sv, str) and "@" in sv:
            pkg, _, ver = sv.rpartition("@")
            subject_pins[pkg] = f"={ver}"
    violations, checked, deferred_ts = [], 0, []
    for lesson_yaml in sorted(course_dir.glob("lessons/*/lesson.yaml")):
        lesson = lesson_yaml.parent.name
        if only and only not in lesson:
            continue
        data = yaml.safe_load(lesson_yaml.read_text()) or {}
        for block in data.get("blocks", []):
            if block.get("type") != "code":
                continue
            lang = block.get("language")
            buildable = block.get("buildType") == "buildable"
            if lang == "typescript" and not buildable:
                continue  # gate 6 already executes these in CI
            key = f"{course_dir.name}/{lesson}/{block.get('key')}"
            if lang == "typescript" and buildable:
                deferred_ts.append(key)
                continue
            if lang != "rust":
                violations.append(f"{key}: unhandled deferred language {lang!r}")
                continue
            checked += 1
            ldir = lesson_yaml.parent
            tests = json.loads((ldir / block["tests"]).read_text())
            sol_src = (ldir / block["solution"]).read_text()
            start_src = (ldir / block["starter"]).read_text()
            compiled, results, err = run_rust_submission(work, f"{lesson}-sol", sol_src, tests, subject_pins)
            if not compiled:
                violations.append(f"{key}: SOLUTION does not compile: {err[:400]}")
            else:
                passed, failed = grade(results, tests)
                if failed:
                    violations.append(f"{key}: SOLUTION fails tests {failed} (err tail: {err[:200]})")
                print(f"  {key}: solution {len(passed)}/{len(tests)}")
            s_compiled, s_results, _ = run_rust_submission(work, f"{lesson}-start", start_src, tests, subject_pins)
            if not s_compiled:
                print(f"  {key}: starter fails to compile (accepted as failing)")
            else:
                s_passed, s_failed = grade(s_results, tests)
                if not s_failed:
                    violations.append(f"{key}: STARTER passes all {len(tests)} tests — challenge is a no-op")
                else:
                    print(f"  {key}: starter fails {len(s_failed)}/{len(tests)}")
    for key in deferred_ts:
        print(f"  {key}: buildType=buildable TS — compile-checked only (tsc) NOT IMPLEMENTED; flagging")
        violations.append(f"{key}: buildable-TS block present; extend this script before merging one")
    return checked, violations


def main(argv):
    args = [a for a in argv[1:] if not a.startswith("--")]
    only = ""
    work = None
    for i, a in enumerate(argv):
        if a == "--only" and i + 1 < len(argv):
            only = argv[i + 1]
        if a == "--work" and i + 1 < len(argv):
            work = Path(argv[i + 1])
    args = [a for a in args if a != only and (work is None or a != str(work))]
    if not args:
        print(__doc__)
        return 2
    work = work or Path(tempfile.mkdtemp(prefix="verify-blocks-"))
    work.mkdir(parents=True, exist_ok=True)
    total, all_violations = 0, []
    for course in args:
        cdir = Path(course).resolve()
        if not (cdir / "course.yaml").exists():
            print(f"skip {cdir}: no course.yaml")
            continue
        print(f"== {cdir.name}")
        checked, violations = verify_course(cdir, only, work)
        total += checked
        all_violations += violations
    print(f"\n{total} deferred block(s) checked, {len(all_violations)} violation(s)")
    for v in all_violations:
        print(f"VIOLATION: {v}")
    return 1 if all_violations else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
