# Contributing

## Quick start

1. Read [courses/CATALOG.md](./courses/CATALOG.md) — the approved catalog, each course's syllabus, the lesson shapes, and the exact APIs to teach. This document is *what* to write; the rest of this file is *how*.
2. Fork this repo and create a branch.
3. Copy `courses/_template/` to `courses/<your-slug>/` and edit it.
4. Open a pull request. CI runs the full validator.

Your editor will autocomplete and validate as you type — `.vscode/settings.json` maps every file to its JSON Schema in `schema/`. In VSCode / Cursor this needs the **[Red Hat YAML extension](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml)** (`redhat.vscode-yaml`); the schema files themselves are generated, so don't hand-edit `schema/`.

Each folder has its own `README.md` with the fields, the controlled vocabulary, and a worked example — read the one for what you're editing ([courses](./courses/README.md), [achievements](./achievements/README.md), [quests](./quests/README.md), [paths](./paths/README.md)).

## Running the validator locally

The linter lives in the app repo, which is public:

```bash
git clone https://github.com/solanabr/superteam-academy
cd superteam-academy && pnpm install
pnpm --filter @superteam-lms/content-lint exec tsx src/cli.ts /path/to/courses-academy
```

Exit code 0 means zero errors. `notice` and `warning` lines never fail the build.

## The rules CI enforces

**Ids are permanent.** `course-…`, `lesson-…`, `achievement-…`, `path-…`, `quest-…`, all `kebab-case`. Course and achievement ids are capped at **32 characters**. Renaming an id after a course ships is rejected — it would lose the progress of everyone who's taken it.

**`slots.lock.json` is generated, not edited.** Never change it by hand; the checks regenerate and compare it.

**XP has a ceiling.** `xpPerLesson × lessonCount ≤ 10000` — go over it and the course can't be completed. `xpPerLesson` is 1–100.

**Challenges are executed.** For a TypeScript challenge, the checks run your `solution.ts` against `tests.json` — it must pass every case — and run your `starter.ts`, which **must fail** at least one. A starter that already passes has nothing to solve. Rust challenges get the same treatment from this repo's own `verify-code` workflow — failing to compile counts as the starter failing — and each graded file must expose exactly one entry `fn`: see [Graded Rust: the one-fn rule](#graded-rust-the-one-fn-rule).

**Code-bearing lessons carry a `versionStamp`.** A top-level `checkedAt` (YYYY-MM-DD) plus a `packages` map of **exact** semver pins — no ranges (`^`, `~`) — recording what the lesson's code was verified against. CATALOG §6 requires one on every code-bearing lesson; content-lint rejects range pins, and review enforces presence. The format lives in [courses/README.md](./courses/README.md#lessons-are-ordered-blocks).

**Every course needs a creator wallet.** `course.creator` must be a real Solana address — that's how the author gets credited (and earns creator XP). It becomes `Course.creator` on-chain and **cannot be changed after the course is created**, so use the wallet you actually want paid.

**Capabilities are ordered.** A block that `consumes: [deployed-program]` must appear after a block that produces it, anywhere earlier in the course — **or in a course listed earlier on a learning path this course sits on** (`paths/*.yaml`), which is how a course declares that it consumes the previous course's output. Only a `wallet-funding` block can produce `funded-wallet`; only a `deployable` `code` block can produce `deployed-program`. Producing across the boundary is a path-order fact only: it does not set `prerequisiteCourse`, which is written on-chain and gates enrollment.

**No orphan files.** Every file in a lesson directory must be referenced by a block (`src`, `starter`, `solution`, `tests`, `idl`) or linked from a prose `.md`. This is why translations live in a course-level `l10n/` folder rather than beside the lessons they translate — see [Translations](#translations) below.

**A translation is an overlay, not a copy.** Never duplicate a course to publish it in another language. `course_id` is a PDA seed, so a duplicate forks the course's enrolment, progress bitmap and XP ledger permanently, and the two can never be merged.

**An empty learning path must say why it is empty.** A path with no courses is hidden by the app either way, so it has to declare intent: `draft: true` means courses are on the way (and it must list them — `draft: true` with `courses: []` is an error), `retired: true` means permanently emptied with the id preserved (and its `courses` must stay empty). A path that never shipped to learners is deleted outright rather than retired — there is no id to reserve. See [paths/README.md](./paths/README.md).

## Quizzes

**Correctness is keyed to a stable option `id`, never to array position** — reordering options can't silently change the answer.

```yaml
- key: check
  type: quiz
  questions:
    - id: q1
      prompt: Where does on-chain state live?
      multiSelect: false
      options:
        - { id: a, label: Data accounts, correct: true, feedback: "Yes — and a program account holds its executable bytes." }
        - { id: b, label: Instructions, correct: false, feedback: "Inputs, not accounts." }
        - { id: c, label: Signatures, correct: false, feedback: "They authorize writes; they don't store state." }
        - { id: d, label: RPC nodes, correct: false, feedback: "They serve reads; the state they serve lives in accounts." }
        - { id: e, label: The program binary, correct: false, feedback: "Programs are stateless and can't write to themselves." }
```

With `multiSelect: false` exactly one option may be correct; with `true` the schema floor is at least one — and the authoring bar below raises it to between two and *k*−2.

**Four options minimum, five when the options are short.** A three-option question hands away a third of the answer. Where the options are short strings — an identifier, a flag, a version, a port — a fifth costs the learner almost nothing to read and is required: the rule is a mean option label of 70 characters or less.

**Every option carries `feedback`, including the correct one.** The app shows feedback for every option the learner *selected*, so a right answer with no feedback is silence at the exact moment they were paying attention.

**Use `multiSelect` wherever the honest answer is a set** — which constraints fire, which extensions compose, which failures share a cause. Never convert a single-answer question to multiSelect just to make it harder: grading is set-equality with no partial credit, so a multiSelect question a learner is one member off on is a wall, not a hint. If two careful reviewers could disagree about one member, it isn't multiSelect. A multiSelect question needs at least five options with between two and *k*−2 correct; all-but-one-correct is "find the single wrong one" wearing extra clicks.

**Don't author the option order.** Run `scripts/quiz_permute.py apply <course>`, which derives it from a hash of the question and option ids. This is not a style preference. The wave-2 courses were generated by a model that had been *asked* to vary the answer position, and it complied by rotating a→b→c per lesson: the position distribution came out almost perfectly balanced while the answer key predicted itself, and a learner who noticed could clear the entire graded layer without reading. A statistical property you ask for is a scheme; compute it instead. `scripts/quiz_stats.py` is the gate, and it measures how well a content-blind strategy scores rather than any single statistic.

**Option ids are immutable.** They're what a translation binds to, so renaming one is a hard compile error against any `l10n/` overlay. Append new options as new ids; never renumber to make them alphabetical after a reorder. Ids carry no meaning and are never rendered — the learner sees labels only.

Two things worth knowing before you write distractors: a distractor exists to be chosen by someone holding a *specific* wrong model the lesson addresses, so if you can't name the misconception it encodes, it's decoration. And the justification belongs in `feedback`, never in the label — an option is a choice, not a lesson. Options that differ in length, register or hedging give the answer away for free; the only difference between them should be truth.

## Graded Rust: the one-fn rule

A graded Rust file — the `starter.rs` and `solution.rs` of a `language: rust` challenge — contains **exactly one bare column-0 `fn`**: unindented, no `pub`, no `const`, not named `main`. That `fn` is the entry point the grader calls, through a `main()` it appends itself. Helpers are `const fn` or nested — inside the entry `fn` or a `mod` — which keeps them out of the count. (A starter may define *none*, when writing the entry `fn` is the exercise — but never more than one.) The template shows the shape: [`courses/_template/lessons/exercise/rs/solution.rs`](./courses/_template/lessons/exercise/rs/solution.rs) is a lone bare `fn add`.

The rule is mechanical, not stylistic. The production grader finds the entry point with a line-anchored regex, `/^fn\s+(\w+)\s*\(/`, which `pub fn` and `const fn` do not match:

- **Zero matches** — say the only candidate is a `pub fn` — and the grader has no entry point, so every test fails **against the reference solution itself**: the lesson ships un-completable. The platform grades Rust at runtime, fail-closed, so the first person to notice would be a learner. The `verify-code` workflow exists to fail your PR instead.
- **Two or more matches**, and which one is called depends on whether the executor takes the first or the last — an app-repo implementation detail this repo can't read. With exactly one, first and last are the same match.

Helpers often *can't* be `const fn` on stable — `for` loops, `&str` matching and `String` allocation are all rejected in const context — so nesting the helper inside the entry function is the usual fallback. All of this applies to `buildType: standard` (the default); a `buildType: buildable` block is a whole crate, compiled rather than called, and shapes its functions however it needs. Reproduce the check locally with `python3 scripts/verify_code_blocks.py courses/<your-slug>` (needs `cargo` and `pyyaml`).

## Reflections are never graded

An `openEnded` block asks the learner to write, and the AI replies once with feedback. It awards no XP and gates nothing. Don't use it to test knowledge — use a `quiz` or a `code` block.

## Answer keys are public

`solution` files and all test cases live in this public repo, by design. Grading is by **execution in a sandbox**, not by secrecy. Don't write a lesson whose value depends on the learner not seeing the answer.

## Translations

Every course declares the language it was written in — `sourceLocale: en | pt-BR | es` in `course.yaml` — and gains other languages through an optional `courses/<slug>/l10n/<locale>/` overlay holding a `strings.yaml` and any prose you translated. You don't have to translate everything: coverage is per string and per file, and anything you leave out renders in the course's source language. A translation is its own pull request, per course per language, and it touches nothing outside `l10n/`.

The format, the rules that keep a translation away from ids and answer keys, and a worked example are in [courses/README.md § Translations](./courses/README.md#translations) and `courses/_template/l10n/pt-BR/`. Read them before starting — the overlay is not yet rendered by the app, and the constraints are currently enforced by review rather than by CI.
