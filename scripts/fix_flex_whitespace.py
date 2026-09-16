#!/usr/bin/env python3
"""fix_flex_whitespace.py — stop flex containers eating the spaces around <b>.

The defect
----------
Shipped figures render words jammed together: "in a separateProgramDataaccount",
"in thesameaccount", "the contract writes itsownstorage", "tens oftrillions". An
audit called it a renderer bug and said to fix the pipeline. It is not a renderer
bug, and the pipeline is fine.

It is CSS working as specified. When an element is `display: flex`, each run of
text between its children becomes an *anonymous flex item*, and the flexbox spec
discards whitespace between flex items. So

    <div class="cell">in a separate <b>ProgramData</b> account</div>

with `.cell { display: flex }` becomes three items -- "in a separate",
<b>ProgramData</b>, "account" -- and both spaces are dropped. Browsers do this
too; it is not specific to WeasyPrint. Verified with a minimal three-case render:
`display:flex` collapses, `display:block` does not, and `display:flex` with the
text wrapped in a single <span> does not.

The fix
-------
Wrap the container's inline content in one <span>, so the container has exactly
one flex item and the whitespace inside it is ordinary inline whitespace. This
changes no layout: the span is inline, inherits everything, and the flex
container still sees a single item to align. Chosen over switching to
`display:block` because these containers use `align-items:center` for vertical
centering inside a stretched row, which block layout would lose.

Usage
-----
    scripts/fix_flex_whitespace.py <course-dir> [--apply]

Without --apply it only reports. Re-render every file it touches afterwards:
`scripts/render_visual.py --render-stale <ref>` will pick them up.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

INLINE = r"(?:b|strong|em|i|span|code|small|sup|sub)"
# text ... <inline>…</inline> ... text  -- the shape whose spaces get eaten
MIXED = re.compile(rf"\w\s+<{INLINE}[^>]*>[^<]*</{INLINE}>\s+\w")
OPEN_DIV = re.compile(r'<div\b([^>]*\bclass="([^"]+)"[^>]*)>')


def flex_classes(css: str) -> set[str]:
    """Classes whose rule block sets display:flex (or inline-flex)."""
    out: set[str] = set()
    for m in re.finditer(r"([^{}]+)\{([^}]*)\}", css):
        if not re.search(r"display:\s*(inline-)?flex", m.group(2)):
            continue
        for sel in m.group(1).split(","):
            names = re.findall(r"\.([A-Za-z0-9_-]+)", sel)
            if names:
                out.add(names[-1])          # the class the rule actually targets
    return out


def find_close(text: str, start: int) -> int | None:
    """Index just past the </div> matching the <div> whose body starts at `start`."""
    depth, i = 1, start
    tag = re.compile(r"</?div\b", re.I)
    while depth:
        m = tag.search(text, i)
        if not m:
            return None
        depth += 1 if m.group(0)[1] != "/" else -1
        i = m.end()
    return text.rfind("<", 0, i)


def fix_file(path: Path, apply: bool) -> tuple[int, list[str]]:
    src = path.read_text(encoding="utf-8")
    flex = flex_classes(src)
    if not flex:
        return 0, []

    edits, notes, out, pos = 0, [], [], 0
    for m in OPEN_DIV.finditer(src):
        if m.start() < pos:
            continue
        if not (set(m.group(2).split()) & flex):
            continue
        body_start = m.end()
        body_end = find_close(src, body_start)
        if body_end is None:
            continue
        body = src[body_start:body_end]
        if "<div" in body or not MIXED.search(body):
            continue                        # nested containers, or nothing to fix
        stripped = body.strip()
        if stripped.startswith("<span") and stripped.endswith("</span>"):
            continue                        # already a single item
        out.append(src[pos:body_start])
        out.append(f"<span>{stripped}</span>")
        pos = body_end
        edits += 1
        notes.append(re.sub(r"\s+", " ", stripped)[:70])
    out.append(src[pos:])

    if apply and edits:
        path.write_text("".join(out), encoding="utf-8")
    return edits, notes


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("course", type=Path)
    ap.add_argument("--apply", action="store_true", help="write the fix; otherwise report only")
    args = ap.parse_args()

    files = sorted(args.course.glob("visual-src/*/*.html"))
    if not files:
        sys.exit(f"no visual-src/*/*.html under {args.course}")

    total, touched = 0, 0
    for f in files:
        n, notes = fix_file(f, args.apply)
        if n:
            touched += 1
            total += n
            print(f"  {f.relative_to(args.course)}: {n}")
            for note in notes[:2]:
                print(f"      {note}")
    verb = "fixed" if args.apply else "would fix"
    print(f"{verb} {total} element(s) across {touched} file(s)")
    if total and not args.apply:
        print("re-run with --apply, then re-render those figures")
    return 0


if __name__ == "__main__":
    sys.exit(main())
