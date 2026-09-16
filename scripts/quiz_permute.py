#!/usr/bin/env python3
"""quiz_permute.py — assign quiz option order deterministically, from a hash.

Why a tool and not an instruction
---------------------------------
The generator that produced these courses was told, in a prompt, to "VARY the
correct option position ... make your first question's correct answer sit at
option index (module_index % 3) and rotate forward from there". It complied. The
result satisfied the gate it was written against -- key positions came out
almost perfectly balanced -- while the answer key became a per-lesson a->b->c
cycle that predicts itself. Asking a language model for a statistical property
gets you a scheme that has that property and nothing else.

So option order is not an authoring decision here. It is computed:

    rank of sha256(salt | lessonSlug | blockKey | questionId | optionId)

The 4-tuple matters. Question ids are NOT unique inside a course -- one shipped
course uses only q1/q2/q3 for all 33 of its questions -- so anything keyed on
the question id alone silently merges questions.

What this guarantees
--------------------
* Only the ORDER of the options array changes. `id`, `label`, `correct`,
  `feedback` and everything else are byte-identical afterwards, and the tool
  asserts that itself rather than trusting the edit.
* Because option `id` is the key a translation binds to -- never array position
  (CONTRIBUTING.md) -- reordering is safe on an already-translated course. That
  is exactly why this belongs in a tool: a prompt-driven rewrite cannot make
  that promise.
* It edits the raw text, not a parsed document, so the diff is a pure block
  reordering with no reformatting noise. `--check` proves it after the fact.

Commands
--------
    quiz_permute.py apply <course-dir> [--epoch N]   rewrite option order in place
    quiz_permute.py check <course-dir> [--epoch N]   exit 1 if order != the hash
    quiz_permute.py plan  <course-dir> [--epoch N]   print the moves, change nothing

Re-rolling
----------
If a course lands on an unlucky arrangement, the ONLY sanctioned remedy is to
bump `--epoch` for that course and re-run, in its own commit, with a reason. Never
hand-tune one question: that is how you get a scheme again. Epochs live in
`scripts/quiz_layout_epochs.txt`.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover
    sys.exit("quiz_permute.py needs PyYAML: pip install pyyaml")

EPOCH_FILE = Path(__file__).with_name("quiz_layout_epochs.txt")
OPTION_START = re.compile(r"^(\s*)-\s+id:\s*(\S+)\s*$")


def load_epochs() -> dict[str, int]:
    epochs: dict[str, int] = {}
    if EPOCH_FILE.is_file():
        for line in EPOCH_FILE.read_text(encoding="utf-8").splitlines():
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            slug, _, value = line.partition(":")
            epochs[slug.strip()] = int(value.strip() or 1)
    return epochs


def order_for(salt: str, lesson: str, block: str, qid: str, option_ids: list[str]) -> list[str]:
    """The canonical order of these option ids. Deterministic, and independent of
    the order they arrive in -- so running twice is a no-op."""
    def h(oid: str) -> str:
        return hashlib.sha256(f"{salt}|{lesson}|{block}|{qid}|{oid}".encode()).hexdigest()
    return sorted(option_ids, key=h)


# --------------------------------------------------------------------------
# Text-level surgery. We locate each options list in the raw file and move whole
# line-spans, so nothing outside those spans can change.
# --------------------------------------------------------------------------

def find_option_spans(lines: list[str], start: int) -> tuple[list[tuple[str, int, int]], int]:
    """Given the index of an `options:` line, return [(optionId, lo, hi)] spans and
    the index just past the list. Returns [] if the list is not block-style."""
    i = start + 1
    spans: list[tuple[str, int, int]] = []
    indent: str | None = None
    cur_id: str | None = None
    cur_lo = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip():                       # blank line inside a list: keep with current item
            i += 1
            continue
        m = OPTION_START.match(line)
        if m and (indent is None or m.group(1) == indent):
            if cur_id is not None:
                spans.append((cur_id, cur_lo, i))
            indent, cur_id, cur_lo = m.group(1), m.group(2), i
            i += 1
            continue
        cur_indent = len(line) - len(line.lstrip())
        if indent is not None and cur_indent <= len(indent) and not line.lstrip().startswith("-"):
            break                                   # dedented out of the list
        if indent is None:
            return [], start + 1                    # flow style or something unexpected
        i += 1
    if cur_id is not None:
        spans.append((cur_id, cur_lo, i))
    return spans, i


def permute_file(path: Path, salt: str, apply: bool) -> tuple[int, int, list[str]]:
    """Returns (questions seen, questions moved, notes)."""
    raw = path.read_text(encoding="utf-8")
    doc = yaml.safe_load(raw) or {}
    lesson = str(doc.get("slug") or path.parent.name)
    lines = raw.splitlines(keepends=True)
    notes: list[str] = []

    # Walk the parsed doc for (block, question) identity, and the raw text for spans.
    # They are matched by scanning forward, which is safe because YAML preserves order.
    wanted: list[tuple[str, str, list[str]]] = []
    for block in doc.get("blocks") or []:
        if block.get("type") != "quiz":
            continue
        bkey = str(block.get("key", ""))
        for q in block.get("questions") or []:
            ids = [str(o.get("id")) for o in (q.get("options") or [])]
            if len(ids) >= 2:
                wanted.append((bkey, str(q.get("id", "")), ids))

    seen = moved = 0
    cursor = 0
    for bkey, qid, ids in wanted:
        # advance to the next `options:` line
        while cursor < len(lines) and not lines[cursor].strip().startswith("options:"):
            cursor += 1
        if cursor >= len(lines):
            notes.append(f"{path.name}: ran out of options blocks at {qid}")
            break
        spans, after = find_option_spans(lines, cursor)
        if not spans:
            notes.append(f"{path.name}: {qid} is not block-style; skipped (fix by hand)")
            cursor = after
            continue
        if [s[0] for s in spans] != ids:
            notes.append(f"{path.name}: {qid} option ids {[s[0] for s in spans]} != parsed {ids}; skipped")
            cursor = after
            continue

        seen += 1
        target = order_for(salt, lesson, bkey, qid, ids)
        if target != ids:
            moved += 1
            if apply:
                chunks = {oid: "".join(lines[lo:hi]) for oid, lo, hi in spans}
                lo, hi = spans[0][1], spans[-1][2]
                lines[lo:hi] = list("".join(chunks[oid] for oid in target).splitlines(keepends=True))
        cursor = after

    if apply and moved:
        new = "".join(lines)
        # Prove the edit was a pure reordering before writing it.
        before, after_doc = yaml.safe_load(raw), yaml.safe_load(new)
        if _option_multiset(before) != _option_multiset(after_doc):
            raise SystemExit(f"{path}: ABORT — permutation changed option content, not just order")
        path.write_text(new, encoding="utf-8")
    return seen, moved, notes


def _option_multiset(doc: dict) -> set:
    """Every option as an order-independent tuple. If this changes, the edit was
    not a reordering and must not be written."""
    out = set()
    for block in doc.get("blocks") or []:
        if block.get("type") != "quiz":
            continue
        for q in block.get("questions") or []:
            for o in q.get("options") or []:
                out.add((str(block.get("key")), str(q.get("id")), str(o.get("id")),
                         str(o.get("label")), bool(o.get("correct")), str(o.get("feedback"))))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", choices=["apply", "check", "plan"])
    ap.add_argument("course", type=Path)
    ap.add_argument("--epoch", type=int, default=None,
                    help="override the course's epoch; bumping it re-rolls the whole course")
    args = ap.parse_args()

    course_yaml = args.course / "course.yaml"
    if not course_yaml.is_file():
        sys.exit(f"no course.yaml under {args.course}")
    slug = (yaml.safe_load(course_yaml.read_text(encoding="utf-8")) or {}).get("slug", args.course.name)
    epoch = args.epoch if args.epoch is not None else load_epochs().get(slug, 1)
    salt = f"{slug}|epoch{epoch}"

    total_seen = total_moved = 0
    all_notes: list[str] = []
    for path in sorted((args.course / "lessons").glob("*/lesson.yaml")):
        seen, moved, notes = permute_file(path, salt, apply=(args.command == "apply"))
        total_seen += seen
        total_moved += moved
        all_notes += notes
        if moved and args.command in ("plan", "check"):
            print(f"  {path.parent.name}: {moved} question(s) not in canonical order")

    for n in all_notes:
        print(f"  note: {n}", file=sys.stderr)

    print(f"{slug} (epoch {epoch}): {total_seen} questions, {total_moved} "
          f"{'reordered' if args.command == 'apply' else 'out of order'}")

    if args.command == "check" and total_moved:
        print("FAIL: option order does not match the hash. Run `apply`, or bump the "
              "epoch in scripts/quiz_layout_epochs.txt if you mean to re-roll.", file=sys.stderr)
        return 1
    return 1 if all_notes and args.command == "check" else 0


if __name__ == "__main__":
    sys.exit(main())
