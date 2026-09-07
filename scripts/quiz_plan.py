#!/usr/bin/env python3
"""quiz_plan.py — compute the authoring quota for a course's quiz overhaul.

Why a planner exists at all
---------------------------
Every question is about to gain a distractor. That new string is the only free
lever in the whole overhaul: rewriting an existing option risks breaking truth
or plausibility, while choosing what the NEW option says costs nothing. So the
new option has to be authored to a target, and the target has to be computed
from the course's own measured baseline BEFORE anyone writes anything.

Author first and measure afterwards and you get the same class of defect the
audit found, in a new shape. Three specific traps this planner exists to avoid:

1. **The <=70-char rule erases itself.** The 5-option cohort is defined by the
   mean label length of the CURRENT 3-option set. Append a fourth option of
   typical length and the mean rises, so a question that qualified at 68 chars
   silently stops qualifying — and the cohort evaporates mid-edit. The cohort
   must be frozen from the baseline, which is what `--baseline <ref>` does.
2. **"Put no absolutes in new distractors" has the wrong sign for some courses.**
   The tell is a RATIO between the distractor class and the key class, and one
   course's ratio is already inverted. A blanket rule makes that course worse.
   The required rate is solved for, per course, and it can be high.
3. **Blind dilution redistributes the length tell instead of removing it.**
   Adding an option of random length just moves which rank the key occupies. So
   the new option's length is a prescribed band, derived from the key's current
   rank and a target rank drawn from the same hash that will order the options.

Usage
-----
    scripts/quiz_plan.py <course-dir> [--baseline <git-ref>] [--json]

`--baseline` reads the pre-overhaul file from git (default `HEAD`) so the plan
stays stable no matter how far the edit has already progressed. Re-running it
mid-wave gives the same answer, which is the point.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import statistics
import subprocess
import sys
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover
    sys.exit("quiz_plan.py needs PyYAML: pip install pyyaml")

sys.path.insert(0, str(Path(__file__).parent))
from quiz_stats import (  # noqa: E402
    ABSOLUTE_RE, HEDGE_RE, SHORT_LABEL_CHARS, content_tokens,
)

CARDINAL_RE = re.compile(r"\b(which three|which two|which four|both|all three|"
                         r"select all|how many|the three|the two)\b", re.I)


def git_show(repo: Path, ref: str, rel: Path) -> str | None:
    r = subprocess.run(["git", "-C", str(repo), "show", f"{ref}:{rel}"],
                       capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def baseline_questions(course: Path, ref: str) -> list[dict]:
    """Read every question as it stood at `ref` -- git is the sidecar the content
    compiler cannot strip, so the frozen cohort needs no field in the YAML."""
    repo = Path(subprocess.run(["git", "-C", str(course), "rev-parse", "--show-toplevel"],
                               capture_output=True, text=True).stdout.strip() or ".")
    out = []
    for path in sorted((course / "lessons").glob("*/lesson.yaml")):
        rel = path.resolve().relative_to(repo.resolve())
        text = git_show(repo, ref, rel)
        if text is None:
            text = path.read_text(encoding="utf-8")   # new lesson, not yet committed
        doc = yaml.safe_load(text) or {}
        for block in doc.get("blocks") or []:
            if block.get("type") != "quiz":
                continue
            for q in block.get("questions") or []:
                opts = q.get("options") or []
                if not opts:
                    continue
                labels = [str(o.get("label", "")) for o in opts]
                correct = [i for i, o in enumerate(opts) if o.get("correct")]
                out.append({
                    "lesson": path.parent.name,
                    "block": str(block.get("key", "")),
                    "qid": str(q.get("id", "")),
                    "prompt": str(q.get("prompt", "")),
                    "labels": labels,
                    "correct": correct,
                    "multi": bool(q.get("multiSelect", False)),
                    "k": len(opts),
                    "mean_len": statistics.mean(len(x) for x in labels),
                    "has_feedback": [bool(o.get("feedback")) for o in opts],
                })
    return out


def required_absolutes_rate(qs: list[dict]) -> tuple[float, float, float]:
    """What fraction of NEW distractors must carry an absolute for the
    distractor/key ratio to land at parity once every question gains one?"""
    keys = [q["labels"][q["correct"][0]] for q in qs if len(q["correct"]) == 1]
    dis = [l for q in qs for i, l in enumerate(q["labels"]) if i not in set(q["correct"])]
    if not keys or not dis:
        return 0.0, 0.0, 0.0
    key_rate = sum(1 for l in keys if ABSOLUTE_RE.search(l)) / len(keys)
    dis_hits = sum(1 for l in dis if ABSOLUTE_RE.search(l))
    dis_rate = dis_hits / len(dis)
    # after adding len(qs) new distractors, we want (dis_hits + x) / (len(dis) + n) == key_rate
    n_new = len(qs)
    x = key_rate * (len(dis) + n_new) - dis_hits
    return dis_rate, key_rate, x / n_new if n_new else 0.0


def multiselect_candidates(qs: list[dict]) -> list[dict]:
    """Build the QUEUE, not the decision. A human or an agent still applies the
    five-part test; this just finds questions worth testing."""
    out = []
    for q in qs:
        if q["multi"] or len(q["correct"]) != 1:
            continue
        key = q["labels"][q["correct"][0]]
        others = [l for i, l in enumerate(q["labels"]) if i not in set(q["correct"])]
        signals = []
        if re.search(r",\s*\w+,?\s+and\s|\band\b.*\band\b|;", key):
            signals.append("conjunction-key")
        kt = content_tokens(key)
        if kt and any(content_tokens(o) and content_tokens(o) < kt for o in others):
            signals.append("subset-distractor")   # the strongest single signal
        if CARDINAL_RE.search(q["prompt"]):
            signals.append("cardinal-in-prompt")
        if signals:
            out.append({"lesson": q["lesson"], "qid": q["qid"], "signals": signals})
    return out


def length_band(q: dict) -> tuple[int, int, bool]:
    """Prescribe the new option's length, targeting only a rank that ADDING an
    option can actually reach.

    The arithmetic that the first version got wrong: with the key at rank `cur`
    of k, appending one option leaves it at `cur` (new option longer) or moves it
    to `cur+1` (new option shorter). Nothing else is reachable. So a key that is
    already the longest of three can only end up 3rd or 4th of four -- and a
    course where most keys start longest can never reach a uniform rank
    distribution by appending alone. Asking for an unreachable target produced a
    band that pushed the rank the wrong way, which is exactly what happened on a
    measured range: [0, 4, 23, 10], chi-square p=3.7e-07.

    Returns (lo, hi, tighten_key). `tighten_key` marks the questions where the
    only way to reach a lower rank is to shorten the KEY -- a real authoring
    move (tighten the answer, never weaken a distractor), and the honest signal
    that appending cannot fix this one.
    """
    lens = sorted(len(x) for x in q["labels"])
    key_len = len(q["labels"][q["correct"][0]]) if q["correct"] else statistics.mean(lens)
    cur_rank = sum(1 for l in lens if l < key_len)
    h = int(hashlib.sha256(f"{q['lesson']}|{q['qid']}|len".encode()).hexdigest()[:8], 16)

    # Reachable target ranks after appending exactly one option.
    reachable = (cur_rank, cur_rank + 1)
    target_rank = reachable[h % 2]

    if target_rank == cur_rank:          # key keeps its rank => new option must be LONGER
        lo, hi = int(key_len) + 1, int(max(lens) * 1.15)
    else:                                # key drops one rank => new option must be SHORTER
        lo, hi = int(min(lens) * 0.85), max(int(key_len) - 1, int(min(lens) * 0.85) + 15)

    # If the key is the longest of its set, ranks below cur_rank are unreachable
    # no matter what we append. Flag it rather than pretending otherwise.
    tighten = cur_rank == len(lens) - 1 and (h % 3 == 0)
    lo, hi = max(20, lo), max(lo + 15, hi)
    return lo, hi, tighten


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("course", type=Path)
    ap.add_argument("--baseline", default="HEAD",
                    help="git ref the 5-option cohort is frozen from (default HEAD)")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    if not (args.course / "course.yaml").is_file():
        sys.exit(f"no course.yaml under {args.course}")
    qs = baseline_questions(args.course, args.baseline)
    if not qs:
        sys.exit("no quiz questions found")

    five = [q for q in qs if q["mean_len"] <= SHORT_LABEL_CHARS]
    dis_rate, key_rate, need = required_absolutes_rate(qs)
    keys = [q["labels"][q["correct"][0]] for q in qs if len(q["correct"]) == 1]
    dis = [l for q in qs for i, l in enumerate(q["labels"]) if i not in set(q["correct"])]
    hedge_key = sum(1 for l in keys if HEDGE_RE.search(l)) / max(1, len(keys))
    hedge_dis = sum(1 for l in dis if HEDGE_RE.search(l)) / max(1, len(dis))
    n_new = len(qs)
    hedge_need = (hedge_key * (len(dis) + n_new) - sum(1 for l in dis if HEDGE_RE.search(l))) / n_new

    missing_fb = sum(1 for q in qs for f in q["has_feedback"] if not f)
    cands = multiselect_candidates(qs)
    plan = {
        "course": args.course.name,
        "baseline_ref": args.baseline,
        "questions": len(qs),
        "five_option_cohort": [f"{q['lesson']}/{q['qid']}" for q in five],
        "absolutes": {"distractor_rate": round(dis_rate, 4), "key_rate": round(key_rate, 4),
                      "required_rate_in_new_distractors": round(need, 4)},
        "hedge": {"key_rate": round(hedge_key, 4), "distractor_rate": round(hedge_dis, 4),
                  "required_rate_in_new_distractors": round(hedge_need, 4)},
        "feedback_strings_to_author": missing_fb,
        "multiselect_candidates": cands,
        "length_bands": {f"{q['lesson']}/{q['qid']}": list(length_band(q)[:2]) for q in qs},
        "tighten_key": [f"{q['lesson']}/{q['qid']}" for q in qs if length_band(q)[2]],
    }

    if args.json:
        print(json.dumps(plan, indent=2))
        return 0

    print(f"── {args.course.name} — authoring quota (baseline {args.baseline})")
    print(f"   {len(qs)} questions; {n_new} new distractors to author "
          f"(+{len(five)} extra for the 5-option cohort)")
    print(f"   5-option cohort (FROZEN at baseline — do not recompute after editing): {len(five)}")
    for q in five:
        print(f"     · {q['lesson']}/{q['qid']}  mean label {q['mean_len']:.0f} chars")
    print(f"   absolutes  distractors {dis_rate:.1%} vs keys {key_rate:.1%} → "
          f"author absolutes into {need:.0%} of new distractors", end="")
    print("   ← NEGATIVE: appending cannot fix this course; existing distractors must be rewritten"
          if need < 0 else "")
    print(f"   hedging    keys {hedge_key:.1%} vs distractors {hedge_dis:.1%} → "
          f"author hedges into {max(0.0, hedge_need):.0%} of new distractors")
    print(f"   feedback   {missing_fb} strings to author (every option needs one, correct included)")
    print(f"   multiSelect candidates to TEST (not decisions): {len(cands)}")
    for c in cands[:12]:
        print(f"     · {c['lesson']}/{c['qid']}  [{', '.join(c['signals'])}]")
    if len(cands) > 12:
        print(f"     · … +{len(cands) - 12} more (use --json for the full list)")
    tighten = plan["tighten_key"]
    print(f"   keys worth TIGHTENING: {len(tighten)} — the key is already the longest of its set,")
    print("   so appending can only leave it top or second-from-top. Shortening the key is the only")
    print("   way to reach a lower rank; tighten the answer, never weaken a distractor.")
    print("   per-question length bands: use --json; the new option's length is prescribed,")
    print("   because dilution with a random-length option just moves the tell instead of removing it.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
