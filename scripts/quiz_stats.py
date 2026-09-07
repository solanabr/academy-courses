#!/usr/bin/env python3
"""quiz_stats.py — statistical gate on a course's quiz layer.

Why this exists
---------------
A quiz layer can satisfy every naive fairness check and still be free to a
learner who never reads the lesson. The wave-2 courses are the worked example:
their correct-answer positions are near-perfectly balanced globally (35/34/35,
33/30/32, 29/30/29, 46/46/43) while the answer key cycles a -> b -> c from one
lesson to the next, so the position of the first answer predicts every answer
after it. The generator produced that on purpose, to satisfy a gate that
hard-failed ">50% of keys in one slot".

So this script does not test one statistic. Its primary metric is
*exploitability*: the accuracy of the best order-1 and order-2 Markov predictor
over the key sequence, measured against a permutation null that holds the
marginal distribution fixed. Position uniformity owns the marginal; the Markov
test owns everything else. Every deterministic scheme a prompt can induce --
constant, forward or reverse rotation, alternation, per-lesson reset, any
period <= 3 -- is order <= 2 and is caught.

Around that sit the surface tells a content-blind learner can also play:
correct-is-longest, key/distractor length ratio, hedging concentrated in keys,
absolutes concentrated in distractors, and the key's own words appearing in the
lesson title.

Usage
-----
    scripts/quiz_stats.py courses/<slug> [courses/<slug> ...]
    scripts/quiz_stats.py --json courses/<slug>
    scripts/quiz_stats.py --all            # every course under courses/

Exit 0 when no metric is at error severity; exit 1 otherwise. Warnings never
fail the run. Metrics without the sample size to be conclusive say so and
downgrade to a warning -- a small course must not manufacture green.

Depends on PyYAML and the standard library. No network.
"""

from __future__ import annotations

import argparse
import json
import math
import random
import re
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover
    sys.exit("quiz_stats.py needs PyYAML: pip install pyyaml")

# --------------------------------------------------------------------------
# Tuning. Every threshold here is stated in the plan; change both together.
# --------------------------------------------------------------------------

PERM_B = 999           # permutation-null resamples; p floors at 1/(B+1) = 0.001
PERM_SEED = 20260907   # fixed so a run is reproducible and a diff is meaningful
MARKOV_P_ERROR = 0.01  # sequential exploitability fails at or below this
CHI2_P_ERROR = 0.001   # marginal uniformity fails below this
CHI2_MIN_N = 20        # below this the marginal test is inconclusive, not passed
SHORT_LABEL_CHARS = 70 # mean option label at or under this must carry 5 options
MIN_OPTIONS = 4
LEN_RATIO_ERROR = (0.80, 1.25)    # mean(len key) / mean(len distractor)
LEN_RATIO_WARN = (0.90, 1.12)
SPREAD_FLAG = 1.7      # per-question longest/shortest label ratio worth counting
SPREAD_ERROR = 2.2     # a single question this lopsided is a defect on its own
ABSOLUTES_WARN, ABSOLUTES_ERROR = 1.5, 2.0   # distractor rate / key rate
HEDGE_WARN, HEDGE_ERROR = 1.5, 2.0           # key rate / distractor rate
DEDUP_JACCARD = 0.55   # cross-lesson key similarity that flags a repeat-keyed fact
MIN_EXPECTED_CELL = 5  # chi-square cell floor before a metric is downgraded

HEDGE_RE = re.compile(
    r"\b(usually|often|typically|generally|in practice|tends to|sometimes|"
    r"not always|roughly|approximately|mostly|primarily|largely|depends on|"
    r"in most cases|can be|may be|might|verify|re-?check|as of)\b",
    re.I,
)
ABSOLUTE_RE = re.compile(
    r"\b(always|never|only|cannot|can't|impossible|guarantees?|every|all of|"
    r"none of|must be|automatically|entirely|completely)\b",
    re.I,
)
WORD_RE = re.compile(r"[A-Za-z0-9_]{4,}")
EMDASH_RE = re.compile(r"[–—]")

STOPWORDS = {
    "that", "this", "with", "from", "have", "which", "what", "when", "your",
    "they", "them", "then", "than", "into", "over", "only", "will", "does",
    "each", "same", "both", "some", "more", "most", "other", "there", "these",
    "those", "because", "before", "after", "while", "would", "could", "about",
}

ERROR, WARN, INFO = "error", "warning", "info"


# --------------------------------------------------------------------------
# Statistics (stdlib only -- no scipy in CI)
# --------------------------------------------------------------------------

def _gser(a: float, x: float) -> float:
    """Series form of the regularized lower incomplete gamma P(a, x)."""
    ap, s, d = a, 1.0 / a, 1.0 / a
    for _ in range(1000):
        ap += 1.0
        d *= x / ap
        s += d
        if abs(d) < abs(s) * 1e-15:
            break
    return s * math.exp(-x + a * math.log(x) - math.lgamma(a))


def _gcf(a: float, x: float) -> float:
    """Continued-fraction form of the regularized upper incomplete gamma Q(a, x)."""
    tiny = 1e-300
    b, c = x + 1.0 - a, 1.0 / tiny
    d = 1.0 / b if b != 0 else 1.0 / tiny
    h = d
    for i in range(1, 1000):
        an = -i * (i - a)
        b += 2.0
        d = an * d + b
        if abs(d) < tiny:
            d = tiny
        c = b + an / c
        if abs(c) < tiny:
            c = tiny
        d = 1.0 / d
        delta = d * c
        h *= delta
        if abs(delta - 1.0) < 1e-15:
            break
    return math.exp(-x + a * math.log(x) - math.lgamma(a)) * h


def chi2_sf(chi2: float, df: int) -> float:
    """P(X >= chi2) for X ~ chi-square with df degrees of freedom."""
    if df <= 0 or chi2 <= 0:
        return 1.0
    a, x = df / 2.0, chi2 / 2.0
    return 1.0 - _gser(a, x) if x < a + 1.0 else _gcf(a, x)


def chi2_uniform(counts: list[int]) -> tuple[float, int, float]:
    """Goodness-of-fit against a uniform distribution. Returns (chi2, df, p)."""
    n, k = sum(counts), len(counts)
    if n == 0 or k < 2:
        return 0.0, 0, 1.0
    exp = n / k
    chi2 = sum((c - exp) ** 2 / exp for c in counts)
    return chi2, k - 1, chi2_sf(chi2, k - 1)


def markov_accuracy(seq: list[int], order: int) -> float | None:
    """Training accuracy of the best order-`order` predictor of `seq`.

    order 0 is the best constant predictor -- exactly what the generator's own
    gate measured, and exactly what a rotation makes look perfect.
    """
    if order == 0:
        return (Counter(seq).most_common(1)[0][1] / len(seq)) if seq else None
    if len(seq) <= order:
        return None
    ctx: dict[tuple, Counter] = defaultdict(Counter)
    for i in range(order, len(seq)):
        ctx[tuple(seq[i - order:i])][seq[i]] += 1
    hits = sum(c.most_common(1)[0][1] for c in ctx.values())
    total = sum(sum(c.values()) for c in ctx.values())
    return hits / total if total else None


def permutation_p(seq: list[int], order: int) -> tuple[float | None, float | None]:
    """Observed order-`order` accuracy and its p-value under a permutation null.

    Shuffling preserves the multiset, so the null is conditional on the marginal:
    this asks "is the ORDER structured?", never "is the mix uneven?". It also
    degrades honestly -- a short honest sequence simply returns a large p.
    """
    obs = markov_accuracy(seq, order)
    if obs is None:
        return None, None
    rng = random.Random(PERM_SEED + order)
    scratch, at_least = list(seq), 0
    for _ in range(PERM_B):
        rng.shuffle(scratch)
        acc = markov_accuracy(scratch, order)
        if acc is not None and acc >= obs - 1e-12:
            at_least += 1
    return obs, (at_least + 1) / (PERM_B + 1)


def binomial_band(p: float, n: int, sigmas: float = 3.0) -> tuple[float, float]:
    """Two-sided normal-approximation band around a rate, so a threshold that is
    fair to 135 questions does not fire spuriously on 33."""
    if n <= 0:
        return 0.0, 1.0
    half = sigmas * math.sqrt(max(p * (1.0 - p), 1e-12) / n)
    return max(0.0, p - half), min(1.0, p + half)


def binom_pmf(k: int, n: int, p: float) -> float:
    if k < 0 or k > n:
        return 0.0
    return math.comb(n, k) * (p ** k) * ((1.0 - p) ** (n - k))


def binom_two_sided_p(obs: int, n: int, p: float) -> float:
    """Exact two-sided binomial p: the total probability of every outcome no more
    likely than the observed one. Exact matters here because the courses that most
    need judging are the small ones."""
    if n <= 0:
        return 1.0
    target = binom_pmf(obs, n, p) * (1.0 + 1e-9)
    return min(1.0, sum(pm for i in range(n + 1) if (pm := binom_pmf(i, n, p)) <= target))


def binom_accept_region(n: int, p: float, alpha: float) -> tuple[int, int]:
    """Counts the two-sided exact test would NOT reject at `alpha`."""
    keep = [i for i in range(n + 1) if binom_two_sided_p(i, n, p) >= alpha]
    return (keep[0], keep[-1]) if keep else (0, n)


def power_against(n: int, p_null: float, p_alt: float, alpha: float) -> float:
    """P(reject | the alternative is true). Used to say UNDERPOWERED instead of PASS.

    The point-mass case matters most: a scheme that never repeats a slot puts all
    its mass at 0, so if 0 sits inside the acceptance region the test is blind to
    exactly the artifact it exists to find.
    """
    lo, hi = binom_accept_region(n, p_null, alpha)
    return sum(binom_pmf(i, n, p_alt) for i in range(n + 1) if i < lo or i > hi)


def jaccard(a: set[str], b: set[str]) -> float:
    return len(a & b) / len(a | b) if (a or b) else 0.0


def content_tokens(text: str) -> set[str]:
    return {w.lower() for w in WORD_RE.findall(text)} - STOPWORDS


# --------------------------------------------------------------------------
# Course model
# --------------------------------------------------------------------------

@dataclass
class Question:
    course: str
    lesson_slug: str
    lesson_title: str
    module_index: int
    block_key: str
    qid: str
    prompt: str
    explanation: str
    multi: bool
    labels: list[str]
    correct_idx: list[int]
    feedback_present: list[bool]

    @property
    def k(self) -> int:
        return len(self.labels)

    @property
    def single(self) -> bool:
        return not self.multi and len(self.correct_idx) == 1

    @property
    def key_label(self) -> str:
        return self.labels[self.correct_idx[0]] if self.correct_idx else ""

    @property
    def distractors(self) -> list[str]:
        return [l for i, l in enumerate(self.labels) if i not in set(self.correct_idx)]

    @property
    def keys(self) -> list[str]:
        return [self.labels[i] for i in self.correct_idx]

    @property
    def mean_label_len(self) -> float:
        return sum(len(l) for l in self.labels) / max(1, self.k)

    def texts(self) -> list[str]:
        return [self.prompt, self.explanation, *self.labels]


@dataclass
class Finding:
    severity: str
    metric: str
    message: str


@dataclass
class CourseReport:
    slug: str
    questions: list[Question]
    findings: list[Finding] = field(default_factory=list)
    stats: dict = field(default_factory=dict)
    enforced: bool = True

    def add(self, severity: str, metric: str, message: str) -> None:
        self.findings.append(Finding(severity, metric, message))

    @property
    def failed(self) -> bool:
        return any(f.severity == ERROR for f in self.findings)


def load_course(course_dir: Path) -> list[Question]:
    """Read a course's quiz questions in display order.

    Display order is what a learner experiences and therefore what the
    sequential test must see, so lessons are ordered by course.yaml's module
    list rather than by directory name.
    """
    course_yaml = course_dir / "course.yaml"
    if not course_yaml.is_file():
        raise FileNotFoundError(f"no course.yaml under {course_dir}")
    course = yaml.safe_load(course_yaml.read_text(encoding="utf-8")) or {}

    # (module index, position within the module) -- the authoritative order.
    # Sorting by directory name inside a module is WRONG: lesson dirs are bare
    # slugs, so `como-funciona` sorts ahead of `por-que-solana` even though it is
    # the second lesson. Every sequential metric here reads the key sequence in
    # reading order, so a wrong order understates the structure it is looking for.
    order: dict[str, tuple[int, int]] = {}
    for m_idx, module in enumerate(course.get("modules") or []):
        for l_idx, lesson_id in enumerate(module.get("lessons") or []):
            order[lesson_id] = (m_idx, l_idx)

    lesson_files = sorted((course_dir / "lessons").glob("*/lesson.yaml"))
    parsed = []
    unplaced = 0
    for path in lesson_files:
        doc = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
        lesson_id = doc.get("id", "")
        if lesson_id in order:
            m_idx, l_idx = order[lesson_id]
        else:                       # not listed in course.yaml: park it at the end
            m_idx, l_idx, unplaced = len(order), unplaced, unplaced + 1
        parsed.append((m_idx, l_idx, path.parent.name, doc))
    parsed.sort(key=lambda t: (t[0], t[1], t[2]))

    questions: list[Question] = []
    for module_index, _lesson_index, slug, doc in parsed:
        for block in doc.get("blocks") or []:
            if block.get("type") != "quiz":
                continue
            for q in block.get("questions") or []:
                options = q.get("options") or []
                labels = [str(o.get("label", "")) for o in options]
                questions.append(Question(
                    course=course_dir.name,
                    lesson_slug=slug,
                    lesson_title=str(doc.get("title", "")),
                    module_index=module_index,
                    block_key=str(block.get("key", "")),
                    qid=str(q.get("id", "")),
                    prompt=str(q.get("prompt", "")),
                    explanation=str(q.get("explanation", "")),
                    multi=bool(q.get("multiSelect", False)),
                    labels=labels,
                    correct_idx=[i for i, o in enumerate(options) if o.get("correct")],
                    feedback_present=[bool(o.get("feedback")) for o in options],
                ))
    return questions


# --------------------------------------------------------------------------
# Metrics
# --------------------------------------------------------------------------

def m_structure(rep: CourseReport) -> None:
    """Option-count policy and feedback completeness. Per-question, no sampling."""
    too_few, needs_five, no_feedback = [], [], []
    for q in rep.questions:
        if q.k < MIN_OPTIONS:
            too_few.append(f"{q.lesson_slug}/{q.qid} (k={q.k})")
        elif q.mean_label_len <= SHORT_LABEL_CHARS and q.k < 5:
            needs_five.append(f"{q.lesson_slug}/{q.qid} (mean label {q.mean_label_len:.0f} chars, k={q.k})")
        if not all(q.feedback_present):
            missing = sum(1 for f in q.feedback_present if not f)
            no_feedback.append(f"{q.lesson_slug}/{q.qid} ({missing}/{q.k})")

    rep.stats["questions"] = len(rep.questions)
    rep.stats["option_counts"] = dict(Counter(q.k for q in rep.questions))
    rep.stats["multiselect"] = sum(1 for q in rep.questions if q.multi)

    if too_few:
        rep.add(ERROR, "option-count", f"{len(too_few)} question(s) below {MIN_OPTIONS} options: " + _sample(too_few))
    if needs_five:
        rep.add(ERROR, "option-count",
                f"{len(needs_five)} short-label question(s) must carry 5 options: " + _sample(needs_five))
    if no_feedback:
        rep.add(ERROR, "feedback", f"{len(no_feedback)} question(s) with options lacking feedback: " + _sample(no_feedback))
    if rep.stats["multiselect"] == 0 and len(rep.questions) >= 20:
        rep.add(WARN, "multiselect",
                "no question in this course has a set-valued answer; confirm that is true of the subject matter")


def m_sequence(rep: CourseReport) -> None:
    """Sequential exploitability -- the metric a rotation cannot survive."""
    seq = [q.correct_idx[0] for q in rep.questions if q.single]
    rep.stats["single_select"] = len(seq)
    if len(seq) < 3:
        rep.add(WARN, "sequence", f"only {len(seq)} single-select question(s); the sequential test is inconclusive, not passed")
        return

    acc0 = markov_accuracy(seq, 0)
    rep.stats["acc0"] = acc0
    for order in (1, 2):
        obs, p = permutation_p(seq, order)
        if obs is None:
            continue
        rep.stats[f"acc{order}"] = obs
        rep.stats[f"p_acc{order}"] = p
        if p <= MARKOV_P_ERROR:
            rep.add(ERROR, "sequence",
                    f"order-{order} predictor scores {obs:.1%} on the key sequence "
                    f"(p={p:.3f} vs a permutation null holding the marginal fixed) — "
                    f"the answer order is predictable from the answers before it")
        elif p <= 0.05:
            rep.add(WARN, "sequence", f"order-{order} predictor scores {obs:.1%} (p={p:.3f}) — borderline structure")


def m_marginal(rep: CourseReport) -> None:
    """Position uniformity, per option-count stratum."""
    strata: dict[int, list[int]] = defaultdict(list)
    for q in rep.questions:
        if q.single:
            strata[q.k].append(q.correct_idx[0])

    dist = {}
    for k, positions in sorted(strata.items()):
        counts = [positions.count(i) for i in range(k)]
        dist[k] = counts
        n = len(positions)
        chi2, df, p = chi2_uniform(counts)
        if n < CHI2_MIN_N or n / k < MIN_EXPECTED_CELL:
            rep.add(WARN, "marginal",
                    f"k={k}: n={n} is too small for the uniformity test "
                    f"(expected cell {n/k:.1f} < {MIN_EXPECTED_CELL}) — inconclusive, not passed [{counts}]")
            continue
        if p < CHI2_P_ERROR:
            rep.add(ERROR, "marginal",
                    f"k={k}: key positions are not uniform, {counts} (chi2={chi2:.1f}, df={df}, p={p:.2g})")
        elif p > 1.0 - CHI2_P_ERROR:
            # A balancing scheme lands too close to perfect. Real randomness is lumpy.
            rep.add(ERROR, "marginal",
                    f"k={k}: key positions are TOO uniform, {counts} (chi2={chi2:.2f}, p={p:.4f}) — "
                    f"honest randomness is lumpier than this; a balancing scheme is the usual cause")
    rep.stats["position_distribution"] = dist


def m_module_seed(rep: CourseReport) -> None:
    """Is a lesson's first key position a function of its module index?

    This is the exact shape of the generator's `${mi % 3}` seed, so it is worth
    testing directly rather than hoping the sequential test notices.
    """
    firsts: dict[str, Question] = {}
    for q in rep.questions:
        if q.single and q.lesson_slug not in firsts:
            firsts[q.lesson_slug] = q
    trials = [(q.module_index, q.correct_idx[0], q.k) for q in firsts.values()]
    if len(trials) < 8:
        return
    hits = sum(1 for mi, pos, k in trials if pos == mi % k)
    n = len(trials)
    p_chance = sum(1.0 / k for _, _, k in trials) / n
    lo, hi = binomial_band(p_chance, n)
    rep.stats["module_seed_hits"] = f"{hits}/{n}"
    if hits / n > hi:
        rep.add(ERROR, "module-seed",
                f"each lesson's first key sits at (module index mod k) in {hits}/{n} lessons "
                f"({hits/n:.0%} vs {p_chance:.0%} chance) — the key position is a function of the module index")


def m_repeat(rep: CourseReport) -> None:
    """Does the key ever land on the same slot twice running?

    This is the keystone, and it is the metric with real power at small n. Any
    scheme that "spreads the keys evenly" -- a rotation, a Latin square, a
    per-lesson stratified permutation -- produces far FEWER repeats than chance,
    which is why the test is two-sided. Being too even is a failure.

    Measured on the pre-fix trees: pr49 0/60, pr54 1/98, pr44 1/74, pr48 2/65,
    b2s 0/18, against a null of 1/3. All five fail, including the two the audit
    called rotation-free.
    """
    trans, chances = 0, []
    repeats = 0
    by_lesson: dict[str, list[Question]] = defaultdict(list)
    for q in rep.questions:
        if q.single:
            by_lesson[q.lesson_slug].append(q)
    for questions in by_lesson.values():
        for a, b in zip(questions, questions[1:]):
            if a.k != b.k:
                continue  # an offset across different option counts is not comparable
            trans += 1
            chances.append(1.0 / b.k)
            if a.correct_idx[0] == b.correct_idx[0]:
                repeats += 1

    if trans < 6:
        return
    p_null = sum(chances) / len(chances)
    p_val = binom_two_sided_p(repeats, trans, p_null)
    rep.stats["repeat_rate"] = f"{repeats}/{trans}"
    rep.stats["p_repeat"] = p_val

    # Can this test even see a never-repeats scheme at this n?
    power = power_against(trans, p_null, 0.0, CHI2_P_ERROR)
    rep.stats["repeat_power"] = power
    if p_val < CHI2_P_ERROR:
        direction = "far fewer" if repeats / trans < p_null else "far more"
        rep.add(ERROR, "repeat-rate",
                f"the key repeats its slot {repeats}/{trans} times ({repeats/trans:.1%}) — "
                f"{direction} than the {p_null:.1%} chance rate (exact two-sided p={p_val:.2g}). "
                f"Keys that never repeat are as predictable as keys that always do")
    elif power < 0.80:
        rep.add(WARN, "repeat-rate",
                f"repeat rate {repeats}/{trans} is within tolerance, but at n={trans} this test has "
                f"only {power:.0%} power against a never-repeats scheme — UNDERPOWERED, not passed")


def m_length_rank(rep: CourseReport) -> None:
    """Uniformity of the key's length RANK among its own options.

    Rate-of-longest is blind to two real tells. pr44 keys the SHORTEST option
    46.2% of the time; pr49's longest-correct rate is a blameless 33.0% while its
    key is the middle length in 61% and the shortest in 5.7%, so "never pick the
    shortest" eliminates an option in 94% of its questions. Rank uniformity sees
    all three; longest-correct sees one.
    """
    singles = [q for q in rep.questions if q.single]
    if not singles:
        return

    by_k: dict[int, list[int]] = defaultdict(list)
    longest = shortest = 0.0
    spread_hits, worst_spread = 0, 0.0
    for q in singles:
        lens = [len(l) for l in q.labels]
        # rank by (length, option index) so ties resolve deterministically
        order = sorted(range(q.k), key=lambda i: (lens[i], i))
        by_k[q.k].append(order.index(q.correct_idx[0]))
        top, bot = max(lens), min(lens)
        longest += (1.0 / lens.count(top)) if len(q.key_label) == top else 0.0
        shortest += (1.0 / lens.count(bot)) if len(q.key_label) == bot else 0.0
        spread = top / max(1, bot)
        worst_spread = max(worst_spread, spread)
        if spread > SPREAD_FLAG:
            spread_hits += 1

    n = len(singles)
    p_chance = sum(1.0 / q.k for q in singles) / n
    rep.stats["chance"] = p_chance
    rep.stats["longest_correct"] = longest / n
    rep.stats["shortest_correct"] = shortest / n

    ranks = {}
    for k, positions in sorted(by_k.items()):
        counts = [positions.count(i) for i in range(k)]
        ranks[k] = counts
        if len(positions) < CHI2_MIN_N or len(positions) / k < MIN_EXPECTED_CELL:
            continue
        chi2, df, p = chi2_uniform(counts)
        if p < CHI2_P_ERROR:
            rep.add(ERROR, "length-rank",
                    f"k={k}: the key's length rank is not uniform, {counts} shortest→longest "
                    f"(chi2={chi2:.1f}, df={df}, p={p:.2g}) — option length predicts the answer")
        elif p < 0.01:
            rep.add(WARN, "length-rank", f"k={k}: key length rank drifting, {counts} (p={p:.3f})")
    rep.stats["length_rank"] = ranks

    # Reported because these are the numbers humans reason about, even though rank subsumes them.
    lo, hi = binomial_band(p_chance, n)
    for name, rate in (("longest", longest / n), ("shortest", shortest / n)):
        if rate > hi or rate < lo:
            rep.add(WARN, f"{name}-correct",
                    f"the key is the {name} option in {rate:.1%} of questions (chance {p_chance:.1%}, "
                    f"3-sigma band {lo:.1%}–{hi:.1%})")

    rep.stats["spread_over_flag"] = f"{spread_hits}/{n}"
    rep.stats["worst_spread"] = worst_spread
    if worst_spread > SPREAD_ERROR or spread_hits / n > 0.10:
        rep.add(ERROR, "option-spread",
                f"{spread_hits}/{n} questions have a longest/shortest label ratio above {SPREAD_FLAG} "
                f"(worst {worst_spread:.1f}x) — options are not parallel, so length alone carries signal")

    key_lens = [len(q.key_label) for q in singles]
    dis_lens = [len(l) for q in singles for l in q.distractors]
    if key_lens and dis_lens:
        ratio = (sum(key_lens) / len(key_lens)) / max(1e-9, sum(dis_lens) / len(dis_lens))
        rep.stats["key_distractor_len_ratio"] = ratio
        if not (LEN_RATIO_ERROR[0] <= ratio <= LEN_RATIO_ERROR[1]):
            rep.add(ERROR, "length-ratio",
                    f"mean key length / mean distractor length = {ratio:.2f}, outside {LEN_RATIO_ERROR}")
        elif not (LEN_RATIO_WARN[0] <= ratio <= LEN_RATIO_WARN[1]):
            rep.add(WARN, "length-ratio", f"mean key/distractor length ratio {ratio:.2f} is drifting")


def m_heuristics(rep: CourseReport) -> None:
    """Score the strategies a learner who never read the lesson would actually use.

    This is the metric that cannot be gamed by word choice, because it IS the
    adversary's objective function. Adding a heuristic rescores the whole corpus
    for free; tuning phrasing to dodge one lexicon does nothing here.
    """
    singles = [q for q in rep.questions if q.single]
    if len(singles) < 12:
        return

    def pick_longest(q): return max(range(q.k), key=lambda i: (len(q.labels[i]), -i))
    def pick_shortest(q): return min(range(q.k), key=lambda i: (len(q.labels[i]), i))

    def pick_no_absolute(q):
        free = [i for i in range(q.k) if not ABSOLUTE_RE.search(q.labels[i])]
        return free[0] if len(free) == 1 else None

    def pick_hedged(q):
        hedged = [i for i in range(q.k) if HEDGE_RE.search(q.labels[i])]
        return hedged[0] if len(hedged) == 1 else None

    def pick_prompt_overlap(q):
        pt = content_tokens(q.prompt)
        scores = [len(content_tokens(q.labels[i]) & pt) for i in range(q.k)]
        best = max(scores)
        return scores.index(best) if best and scores.count(best) == 1 else None

    def pick_least_similar(q):
        toks = [content_tokens(l) for l in q.labels]
        scores = [sum(jaccard(toks[i], toks[j]) for j in range(q.k) if j != i) for i in range(q.k)]
        low = min(scores)
        return scores.index(low) if scores.count(low) == 1 else None

    def pick_len_rank(rank: int):
        """"Always pick the Nth-shortest option."

        longest/shortest are just the two ends of this family, and the middle is
        where a real course lands: one measured range had the key third-longest
        in 23 of 37 questions, which a learner can play for 62% while both ends
        sit at chance. Testing only the extremes declares that clean.
        """
        def pick(q):
            order = sorted(range(q.k), key=lambda i: (len(q.labels[i]), i))
            return order[rank] if rank < q.k else None
        return pick

    suite = {
        "longest": pick_longest, "shortest": pick_shortest,
        "only-option-without-an-absolute": pick_no_absolute,
        "only-hedged-option": pick_hedged,
        "most-prompt-overlap": pick_prompt_overlap,
        "least-like-the-others": pick_least_similar,
    }
    # Interior length ranks. Rank 0 and the top rank duplicate shortest/longest
    # for a fixed k, so only the middles add anything.
    for r in range(1, max((q.k for q in singles), default=3) - 1):
        suite[f"length-rank-{r}"] = pick_len_rank(r)

    # Per-question record of which strategies land on the key. This is the
    # authoring queue: a question three strategies solve needs real work, and a
    # question none solve is already doing its job. Aggregate rates say a course
    # is exploitable; this says WHERE.
    solved: dict[str, list[str]] = defaultdict(list)

    results = {}
    for name, fn in suite.items():
        hits = attempts = 0
        chances = []
        for q in singles:
            guess = fn(q)
            if guess is None:
                continue
            attempts += 1
            chances.append(1.0 / q.k)
            if guess == q.correct_idx[0]:
                hits += 1
                solved[f"{q.lesson_slug}/{q.qid}"].append(name)
        if attempts < 12:
            continue
        p_null = sum(chances) / len(chances)
        p_val = binom_two_sided_p(hits, attempts, p_null)
        results[name] = {"hits": hits, "n": attempts, "rate": hits / attempts, "p": p_val}
        if hits / attempts > p_null and p_val < CHI2_P_ERROR:
            rep.add(ERROR, "content-blind",
                    f"the '{name}' strategy scores {hits}/{attempts} ({hits/attempts:.1%}) against "
                    f"{p_null:.1%} chance (exact p={p_val:.2g}) — this question set is partly solvable "
                    f"without reading the lesson")

    rep.stats["heuristics"] = results
    rep.stats["review_queue"] = {k: v for k, v in sorted(
        solved.items(), key=lambda kv: (-len(kv[1]), kv[0])) if len(v) >= 2}
    if results:
        best = max(results.values(), key=lambda r: r["rate"])
        rep.stats["best_heuristic"] = best["rate"]


def _rate(texts: list[str], pattern: re.Pattern) -> tuple[float, int]:
    if not texts:
        return 0.0, 0
    hits = sum(1 for t in texts if pattern.search(t))
    return hits / len(texts), hits


def m_lexical(rep: CourseReport) -> None:
    """Hedging concentrated in keys, absolutes concentrated in distractors."""
    keys = [l for q in rep.questions for l in q.keys]
    distractors = [l for q in rep.questions for l in q.distractors]
    if not keys or not distractors:
        return

    hedge_k, hk = _rate(keys, HEDGE_RE)
    hedge_d, hd = _rate(distractors, HEDGE_RE)
    abs_k, ak = _rate(keys, ABSOLUTE_RE)
    abs_d, ad = _rate(distractors, ABSOLUTE_RE)
    rep.stats["hedge_key_rate"], rep.stats["hedge_distractor_rate"] = hedge_k, hedge_d
    rep.stats["absolutes_key_rate"], rep.stats["absolutes_distractor_rate"] = abs_k, abs_d

    if hk + hd >= 8 and hedge_d > 0:
        ratio = hedge_k / hedge_d
        rep.stats["hedge_ratio"] = ratio
        if ratio >= HEDGE_ERROR:
            rep.add(ERROR, "hedge-tell",
                    f"keys hedge {ratio:.2f}x as often as distractors ({hedge_k:.1%} vs {hedge_d:.1%}) — "
                    f"'pick the cautious option' is a working strategy")
        elif ratio >= HEDGE_WARN:
            rep.add(WARN, "hedge-tell", f"keys hedge {ratio:.2f}x as often as distractors")

    if ak + ad >= 8 and abs_k > 0:
        ratio = abs_d / abs_k
        rep.stats["absolutes_ratio"] = ratio
        if ratio >= ABSOLUTES_ERROR:
            rep.add(ERROR, "absolutes-tell",
                    f"distractors carry absolutes {ratio:.2f}x as often as keys ({abs_d:.1%} vs {abs_k:.1%}) — "
                    f"'eliminate the absolute' is a working strategy")
        elif ratio >= ABSOLUTES_WARN:
            rep.add(WARN, "absolutes-tell", f"distractors carry absolutes {ratio:.2f}x as often as keys")


def m_answer_leak(rep: CourseReport) -> None:
    """Does the key's distinguishing vocabulary appear in the lesson title or prompt?"""
    leaks = []
    for q in rep.questions:
        if not q.single:
            continue
        key_tokens = content_tokens(q.key_label)
        other = set().union(*(content_tokens(l) for l in q.distractors)) if q.distractors else set()
        unique = key_tokens - other
        if not unique:
            continue
        for where, text in (("title", q.lesson_title), ("prompt", q.prompt)):
            if len(unique & content_tokens(text)) >= 2:
                leaks.append(f"{q.lesson_slug}/{q.qid} (in {where})")
                break
    rep.stats["answer_leaks"] = len(leaks)
    if leaks:
        rep.add(WARN, "answer-leak",
                f"{len(leaks)} question(s) whose key vocabulary appears in the lesson title or prompt: " + _sample(leaks))


def m_dedup(rep: CourseReport) -> None:
    """Repeat-keyed facts -- a FLOOR, not a count.

    Lexical similarity badly under-reports this. Measured on the real corpus, a
    Jaccard scan finds 4 near-duplicate pairs across all five courses, while one
    course keys the same thesis in at least eight lessons that share almost no
    vocabulary. The real duplicates are semantic, so this metric is a cheap
    backstop for the careless cases and nothing more. A course with zero hits
    here has NOT been shown to be free of repeat-keyed facts; only an authored
    claim ledger can show that.
    """
    singles = [q for q in rep.questions if q.single and len(content_tokens(q.key_label)) >= 4]
    pairs = []
    for i, a in enumerate(singles):
        ta = content_tokens(a.key_label)
        for b in singles[i + 1:]:
            if a.lesson_slug == b.lesson_slug:
                continue
            if jaccard(ta, content_tokens(b.key_label)) >= DEDUP_JACCARD:
                pairs.append(f"{a.lesson_slug}/{a.qid} ~ {b.lesson_slug}/{b.qid}")
    rep.stats["repeat_keyed_pairs"] = len(pairs)
    if pairs:
        rep.add(WARN, "repeat-keyed",
                f"at least {len(pairs)} cross-lesson pair(s) key near-identical claims "
                f"(lexical floor — semantic repeats are invisible here): " + _sample(pairs))


def m_emdash(rep: CourseReport) -> None:
    """Em-dashes in quiz text. Reported, never fatal -- house style lives elsewhere."""
    hits = sum(len(EMDASH_RE.findall(t)) for q in rep.questions for t in q.texts())
    rep.stats["emdashes"] = hits
    if hits:
        rep.add(INFO, "em-dash", f"{hits} em/en-dash(es) in quiz text")


METRICS = (m_structure, m_sequence, m_repeat, m_marginal, m_module_seed,
           m_length_rank, m_heuristics, m_lexical, m_answer_leak, m_dedup, m_emdash)


def _sample(items: list[str], n: int = 4) -> str:
    head = ", ".join(items[:n])
    return head + (f", +{len(items) - n} more" if len(items) > n else "")


def analyse(course_dir: Path) -> CourseReport:
    rep = CourseReport(slug=course_dir.name, questions=load_course(course_dir))
    if not rep.questions:
        rep.add(WARN, "empty", "no quiz questions found")
        return rep
    for metric in METRICS:
        metric(rep)
    return rep


# --------------------------------------------------------------------------
# Output
# --------------------------------------------------------------------------

ICON = {ERROR: "FAIL", WARN: "warn", INFO: "info"}


def render(rep: CourseReport) -> str:
    s = rep.stats
    lines = [f"── {rep.slug} " + "─" * max(0, 66 - len(rep.slug))]
    counts = ", ".join(f"{k}-option x{v}" for k, v in sorted(s.get("option_counts", {}).items()))
    lines.append(f"   {s.get('questions', 0)} questions ({counts}); "
                 f"{s.get('single_select', 0)} single-select, {s.get('multiselect', 0)} multiSelect")
    if "acc1" in s:
        lines.append(f"   predictability   constant {s.get('acc0', 0):.1%} | "
                     f"order-1 {s['acc1']:.1%} (p={s.get('p_acc1', 1):.3f}) | "
                     f"order-2 {s.get('acc2', 0):.1%} (p={s.get('p_acc2', 1):.3f})")
    if "repeat_rate" in s:
        lines.append(f"   slot repeats     {s['repeat_rate']} (p={s.get('p_repeat', 1):.2g}, "
                     f"power vs never-repeats {s.get('repeat_power', 0):.0%})")
    if "longest_correct" in s:
        lines.append(f"   length tells     longest {s['longest_correct']:.1%} | "
                     f"shortest {s.get('shortest_correct', 0):.1%} | chance {s.get('chance', 0):.1%} | "
                     f"key/distractor {s.get('key_distractor_len_ratio', 0):.2f} | "
                     f"worst spread {s.get('worst_spread', 0):.1f}x")
    if s.get("length_rank"):
        rank = "; ".join(f"k={k}: {v}" for k, v in s["length_rank"].items())
        lines.append(f"   key length rank  {rank}   (shortest→longest)")
    if s.get("heuristics"):
        best = max(s["heuristics"].items(), key=lambda kv: kv[1]["rate"])
        lines.append(f"   content-blind    best strategy '{best[0]}' scores {best[1]['rate']:.1%} "
                     f"({best[1]['hits']}/{best[1]['n']}, p={best[1]['p']:.2g})")
    if "hedge_ratio" in s or "absolutes_ratio" in s:
        lines.append(f"   lexical tells    hedge {s.get('hedge_ratio', float('nan')):.2f}x | "
                     f"absolutes {s.get('absolutes_ratio', float('nan')):.2f}x")
    if s.get("position_distribution"):
        pos = "; ".join(f"k={k}: {v}" for k, v in s["position_distribution"].items())
        lines.append(f"   key positions    {pos}")

    order = {ERROR: 0, WARN: 1, INFO: 2}
    for f in sorted(rep.findings, key=lambda f: order[f.severity]):
        lines.append(f"   [{ICON[f.severity]}] {f.metric}: {f.message}")
    if rep.failed and not rep.enforced:
        verdict = "FAIL (not yet enforced — informational until this course goes through the wave)"
    else:
        verdict = "FAIL" if rep.failed else "pass"
    lines.append(f"   => {verdict}")
    return "\n".join(lines)


def _synth(scheme: str, lessons: int = 30, per_lesson: int = 3, k: int = 4) -> CourseReport:
    """Build a synthetic course with a known keying scheme.

    Labels are drawn from a fixed pool of equal-length strings so the length and
    lexical metrics stay silent and each fixture tests exactly one thing.
    """
    rng = random.Random(4242)
    pool = [f"an option of quite ordinary length, number {i:02d}" for i in range(k)]
    qs: list[Question] = []
    for li in range(lessons):
        for qi in range(per_lesson):
            idx = li * per_lesson + qi
            if scheme == "rotation":
                correct = (li + qi) % k          # the artifact this gate exists to catch
            elif scheme == "all-first":
                correct = 0
            elif scheme == "honest":
                correct = rng.randrange(k)
            elif scheme == "balanced-no-repeat":
                # a Latin-square style scheme: perfect marginal, never repeats
                correct = (idx * 1) % k if qi == 0 else (qs[-1].correct_idx[0] + 1 + rng.randrange(k - 1)) % k
            else:
                raise ValueError(scheme)
            qs.append(Question(
                course="synth", lesson_slug=f"l{li:02d}", lesson_title="synthetic lesson",
                module_index=li // 3, block_key="check", qid=f"l{li:02d}-q{qi}",
                prompt="A synthetic prompt with no overlap.", explanation="x",
                multi=False, labels=list(pool),
                correct_idx=[correct], feedback_present=[True] * k,
            ))
    rep = CourseReport(slug=f"synth-{scheme}", questions=qs)
    for metric in (m_sequence, m_repeat, m_marginal, m_module_seed):
        metric(rep)
    return rep


def selftest() -> int:
    """The regression test for this whole file: the schemes that shipped must
    fail, and an honest shuffle must pass. A gate that fails everything is not a
    gate, and neither is one that passes the artifact it was written for."""
    cases = [
        ("rotation", True, "the per-lesson a->b->c rotation that shipped in four courses"),
        ("all-first", True, "every key in slot one"),
        ("balanced-no-repeat", True, "perfect marginal, never repeats a slot"),
        ("honest", False, "an honest uniform shuffle"),
    ]
    ok = True
    for scheme, should_fail, why in cases:
        rep = _synth(scheme)
        failed = rep.failed
        verdict = "PASS" if failed == should_fail else "**WRONG**"
        if failed != should_fail:
            ok = False
        metrics = sorted({f.metric for f in rep.findings if f.severity == ERROR})
        print(f"  [{verdict}] {scheme:20} expected {'fail' if should_fail else 'pass'}, "
              f"got {'fail' if failed else 'pass'} — {why}")
        if metrics:
            print(f"           fired: {', '.join(metrics)}")
    print("selftest OK" if ok else "selftest FAILED")
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description="Statistical gate on a course's quiz layer.")
    if "--selftest" in sys.argv:
        return selftest()
    ap.add_argument("courses", nargs="*", type=Path, help="course directories (containing course.yaml)")
    ap.add_argument("--all", action="store_true", help="analyse every course under ./courses")
    ap.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    ap.add_argument("--review-queue", action="store_true",
                    help="list the questions two or more content-blind strategies already solve")
    args = ap.parse_args()

    targets = list(args.courses)
    if args.all:
        # `_template` and `_draft` are scaffolding, not shipped courses.
        targets += [p.parent for p in sorted(Path("courses").glob("*/course.yaml"))
                    if not p.parent.name.startswith("_")]
    if not targets:
        ap.error("give at least one course directory, or --all")

    # A course only fails the build once it has been through the assessment wave.
    # Everything else is measured and printed but not enforced, so landing the
    # gate does not red-line courses nobody has been funded to fix. Add a slug
    # here in the same PR that fixes it -- never before, never separately.
    enforced_file = Path(__file__).with_name("quiz_gate_enforced.txt")
    enforced: set[str] | None = None
    if enforced_file.is_file():
        enforced = {
            line.split("#", 1)[0].strip()
            for line in enforced_file.read_text(encoding="utf-8").splitlines()
            if line.split("#", 1)[0].strip()
        }

    reports, failed = [], False
    for target in targets:
        if not (target / "course.yaml").is_file():
            print(f"skipping {target}: no course.yaml", file=sys.stderr)
            continue
        rep = analyse(target)
        rep.enforced = enforced is None or rep.slug in enforced
        reports.append(rep)
        failed = failed or (rep.failed and rep.enforced)

    if args.review_queue:
        for rep in reports:
            queue = rep.stats.get("review_queue") or {}
            print(f"── {rep.slug}: {len(queue)} question(s) solved by 2+ content-blind strategies")
            for qid, strategies in queue.items():
                print(f"   {qid:44} {', '.join(strategies)}")
            if not queue:
                print("   (none — no question falls to two independent strategies at once)")
        return 0

    if args.json:
        print(json.dumps([{
            "course": r.slug,
            "failed": r.failed,
            "stats": r.stats,
            "findings": [{"severity": f.severity, "metric": f.metric, "message": f.message} for f in r.findings],
        } for r in reports], indent=2))
    else:
        for rep in reports:
            print(render(rep))
            print()
        n_err = sum(1 for r in reports for f in r.findings if f.severity == ERROR)
        print(f"{len(reports)} course(s), {n_err} error-severity finding(s).")

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
