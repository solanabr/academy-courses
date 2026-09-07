#!/usr/bin/env python3
"""render_visual.py — re-render one course figure from its committed HTML source.

Why this exists
---------------
A round-1 fix wave corrected prose and left the figure beside it teaching the
retracted model. Six shipped images still do. The obvious remedy -- re-render
them -- was recorded as unsafe, because "the brand corner blobs change on every
render". That belief is what stopped the figures being fixed.

It is wrong, and the distinction matters:

* `weasyprint` is deterministic. Two runs of the same untouched HTML produce a
  byte-identical PDF and a byte-identical PNG. Verified.
* The decoration is deterministic too: the generator seeds it with
  sha256(asset_id) and bakes the resulting <div class="stbr-decor"> INLINE into
  the committed HTML, which even carries the comment "Leave the .stbr-decor
  block untouched".
* What is NOT stable is the generator's `decorate` command, whose blob COUNT is
  a function of content density. Edit a caption, re-run it, and the arrangement
  re-rolls.

So the rule is simply: render from the committed HTML, never re-decorate. This
tool enforces that rather than trusting anyone to remember it -- it refuses to
run if the decor block differs from the one in git.

Second reason it exists: the recipe moved and the notes did not. Shipped assets
are WebP now (PR #52), not PNG, chosen per image -- lossless where it wins,
lossy where it wins by a wide margin and only above 42 dB PSNR. Re-rendering to
PNG would silently undo that.

Usage
-----
    scripts/render_visual.py courses/<slug>/visual-src/<lesson>/<name>.html
    scripts/render_visual.py --check courses/<slug>/visual-src/<lesson>/<name>.html

`--check` renders to a temp file and reports whether the shipped asset is
already up to date, without writing anything.

Needs: weasyprint, sips (macOS), cwebp. On macOS weasyprint needs
DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/lib.
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

CANVAS_WIDTH = 2400
PSNR_FLOOR = 42.0          # the gate PR #52 used for lossy encodes
QUALITY_LADDER = (80, 85, 90, 93, 95, 97)
DECOR_RE = re.compile(r'<div class="stbr-decor".*?</div>\s*(?=<div class="viz")', re.DOTALL)


def run(cmd: list[str], **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def decor_block(html: str) -> str | None:
    m = DECOR_RE.search(html)
    return m.group(0) if m else None


def repo_of(path: Path) -> Path | None:
    """The worktree containing `path` -- not the one containing this script.

    Courses live in per-branch worktrees, so the script is routinely run from one
    checkout against files in another. Resolving the repo from the script's own
    location silently compares against the wrong HEAD.
    """
    r = run(["git", "-C", str(path.resolve().parent), "rev-parse", "--show-toplevel"])
    return Path(r.stdout.strip()) if r.returncode == 0 and r.stdout.strip() else None


def committed_version(path: Path) -> str | None:
    repo = repo_of(path)
    if repo is None:
        return None
    try:
        rel = path.resolve().relative_to(repo.resolve())
    except ValueError:
        return None
    r = run(["git", "-C", str(repo), "show", f"HEAD:{rel}"])
    return r.stdout if r.returncode == 0 else None


def asset_for(src: Path) -> Path:
    """visual-src/<lesson>/<name>.html -> lessons/<lesson>/assets/<name>.webp"""
    course = src.parent.parent.parent
    return course / "lessons" / src.parent.name / "assets" / f"{src.stem}.webp"


def _parse_psnr(text: str) -> float | None:
    """cwebp -print_psnr prints two lines; the per-channel one ends in `Total:NN.NN`,
    and the summary one ends in `NN.NN dB`. Prefer Total, fall back to the dB figure."""
    m = re.search(r"Total:\s*([\d.]+)", text)
    if m:
        return float(m.group(1))
    m = re.search(r"([\d.]+)\s*dB", text)
    return float(m.group(1)) if m else None


def encode_webp(png: Path, out: Path) -> tuple[str, float, int]:
    """Pick lossless or lossy the way PR #52 did: whichever is smaller, with lossy
    only permitted once it clears the PSNR floor."""
    lossless = out.with_suffix(".lossless.webp")
    run(["cwebp", "-quiet", "-lossless", "-z", "9", str(png), "-o", str(lossless)])
    best_mode, best_psnr = "lossless", float("inf")
    best_file, best_size = lossless, lossless.stat().st_size if lossless.exists() else 1 << 60

    for q in QUALITY_LADDER:
        cand = out.with_suffix(f".q{q}.webp")
        # NOT -quiet: that suppresses the very PSNR line we gate on.
        r = run(["cwebp", "-print_psnr", "-q", str(q), str(png), "-o", str(cand)])
        if not cand.exists():
            continue
        psnr = _parse_psnr(r.stderr + r.stdout)
        if psnr is None or psnr < PSNR_FLOOR:
            cand.unlink(missing_ok=True)
            continue  # climb the ladder until one clears the floor
        if cand.stat().st_size < best_size:
            best_mode, best_psnr, best_file, best_size = f"lossy q{q}", psnr, cand, cand.stat().st_size
        break  # ascending quality means the first clearing entry is the smallest

    shutil.copyfile(best_file, out)
    for stray in out.parent.glob(out.stem + ".*.webp"):
        stray.unlink(missing_ok=True)
    return best_mode, best_psnr, best_size


def list_stale(ref: str) -> int:
    """Every figure whose HTML source changed since `ref` but whose asset did not.

    This is the sweep driver. Content fixes edit `visual-src/*.html` and
    deliberately do NOT re-render -- rendering is a separate pass so the decor
    rule is applied once, by one person, with the diffs reviewed together. That
    leaves a window where the source is corrected and the shipped image still
    teaches the old model, which is precisely the state six figures have been in
    since round 1. This finds them instead of relying on someone's notes.
    """
    repo = repo_of(Path.cwd() / "x") or Path(__file__).resolve().parent.parent
    r = run(["git", "-C", str(repo), "diff", "--name-only", ref, "--", "courses"])
    if r.returncode != 0:
        sys.exit(f"git diff against {ref} failed: {r.stderr.strip()}")
    changed = [Path(p) for p in r.stdout.split()]
    html = [p for p in changed if p.suffix == ".html" and "visual-src" in p.parts]
    assets = {p for p in changed if p.suffix in (".webp", ".png")}

    stale = []
    for p in html:
        asset = asset_for(repo / p)
        rel = asset.relative_to(repo)
        if rel not in assets:
            stale.append((p, rel))

    if not stale:
        print(f"no stale figures against {ref}: every changed visual source was re-rendered.")
        return 0
    print(f"{len(stale)} figure(s) whose source changed since {ref} but whose shipped asset did not:")
    for src_p, asset_p in stale:
        exists = "" if (repo / asset_p).exists() else "   [asset missing]"
        print(f"  {src_p}{exists}")
    print("\nRe-render each with:  scripts/render_visual.py <source>")
    return 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("source", type=Path, nargs="?", help="the visual-src HTML")
    ap.add_argument("--check", action="store_true", help="report only; write nothing")
    ap.add_argument("--stale", metavar="REF",
                    help="list figures whose source changed since REF but were never re-rendered")
    ap.add_argument("--render-stale", metavar="REF",
                    help="re-render every figure --stale would list")
    args = ap.parse_args()

    if args.stale:
        return list_stale(args.stale)

    if args.render_stale:
        repo = repo_of(Path.cwd() / "x") or Path(__file__).resolve().parent.parent
        r = run(["git", "-C", str(repo), "diff", "--name-only", args.render_stale, "--", "courses"])
        changed = [Path(p) for p in r.stdout.split()]
        assets = {p for p in changed if p.suffix in (".webp", ".png")}
        todo = [p for p in changed
                if p.suffix == ".html" and "visual-src" in p.parts
                and asset_for(repo / p).relative_to(repo) not in assets]
        if not todo:
            print(f"nothing stale against {args.render_stale}.")
            return 0
        print(f"re-rendering {len(todo)} figure(s) against {args.render_stale}\n")
        failures = 0
        for p in todo:
            rc = subprocess.run([sys.executable, __file__, str(repo / p)]).returncode
            failures += 1 if rc else 0
        print(f"\n{len(todo) - failures} rendered, {failures} failed.")
        return 1 if failures else 0
    if args.source is None:
        ap.error("give a source file, or --stale <ref>")

    src = args.source
    if not src.is_file():
        sys.exit(f"no such file: {src}")
    for tool in ("weasyprint", "sips", "cwebp"):
        if not shutil.which(tool):
            sys.exit(f"missing required tool: {tool}")

    html = src.read_text(encoding="utf-8")
    now, was = decor_block(html), None
    committed = committed_version(src)
    if committed is not None:
        was = decor_block(committed)
    if was is not None and now is not None and was != now:
        sys.exit(f"{src}: REFUSING — the .stbr-decor block differs from the committed one.\n"
                 f"  The decoration is generated data, not authored content, and re-rolling it is\n"
                 f"  what made round 1 call re-rendering unsafe. Restore the committed block and\n"
                 f"  re-run; never run the generator's `decorate` command on an existing asset.")

    asset = asset_for(src)
    with tempfile.TemporaryDirectory() as tmp:
        pdf, png = Path(tmp) / "o.pdf", Path(tmp) / "o.png"
        # Absolute path: weasyprint resolves `<link href="_brand.css">` against the
        # document's own location, so cwd is irrelevant and a relative path is a trap.
        r = run(["weasyprint", str(src.resolve()), str(pdf)])
        if not pdf.exists():
            sys.exit(f"weasyprint failed for {src}:\n{r.stderr[-800:]}")
        run(["sips", "-s", "format", "png", "--resampleWidth", str(CANVAS_WIDTH),
             str(pdf), "--out", str(png)])
        if not png.exists():
            sys.exit(f"sips failed for {src}")

        out = Path(tmp) / "o.webp" if args.check else asset
        out.parent.mkdir(parents=True, exist_ok=True)
        before = asset.stat().st_size if asset.exists() else 0
        mode, psnr, size = encode_webp(png, out)

        dims = run(["sips", "-g", "pixelWidth", "-g", "pixelHeight", str(png)]).stdout
        w = h = "?"
        for line in dims.splitlines():
            if "pixelWidth" in line:
                w = line.split(":")[-1].strip()
            if "pixelHeight" in line:
                h = line.split(":")[-1].strip()

        verb = "would write" if args.check else "wrote"
        psnr_s = "" if psnr == float("inf") else f", {psnr:.1f} dB"
        delta = f" (was {before:,} B)" if before else ""
        print(f"{src.parent.name}/{src.name}: {verb} {asset.relative_to(asset.parents[3])} "
              f"— {w}x{h}, {mode}{psnr_s}, {size:,} B{delta}")
        if args.check and before and abs(size - before) / before > 0.25:
            print("  note: size moved more than 25% — inspect the image before committing.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
