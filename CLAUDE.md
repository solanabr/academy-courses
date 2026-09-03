# courses-academy — agent guide

Source of truth for all Superteam Academy content. This repo is data, not code: plain YAML, Markdown, and source files.

**How it reaches learners:** merged content here *stages*; the app repo (`solanabr/superteam-academy`) compiles this tree into a committed bundle and activates it by bumping `content.lock`. Courses are then deployed on-chain from the admin panel. There is no CMS in the path.

**What to write** — the approved catalog, per-course syllabi, lesson shapes, and the exact APIs/versions to teach — is [courses/CATALOG.md](./courses/CATALOG.md). Read it before authoring or restructuring a course.

**When editing a folder, read that folder's `README.md` and its `schema/<type>.schema.json` first** — they hold the fields, the controlled vocabulary, and the worked examples:

| Editing… | Read |
|---|---|
| a course / lesson / slots | [courses/README.md](./courses/README.md) + `schema/course.schema.json`, `schema/lesson.schema.json` |
| a translation (`courses/*/l10n/`) | [courses/README.md § Translations](./courses/README.md#translations) + the worked example at `courses/_template/l10n/pt-BR/` |
| an achievement | [achievements/README.md](./achievements/README.md) + `schema/achievement.schema.json` |
| a quest | [quests/README.md](./quests/README.md) + `schema/quest.schema.json` |
| a learning path | [paths/README.md](./paths/README.md) + `schema/path.schema.json` |

## The rules that actually bite

- **Never hand-edit `courses/*/slots.lock.json`.** It pins on-chain bitmap positions; CI regenerates it and fails on any diff. A wrong slot corrupts real learner progress.
- **Ids are immutable and some are PDA seeds.** Never strip a prefix or rename. `course-*` / `achievement-*` ≤ 32 UTF-8 bytes; the rest ≤ 128.
- **`xpPerLesson × lessonCount ≤ 10000`**, or `finalize_course` reverts forever and nobody can complete the course.
- **A `code` block's `solution` must pass `tests.json`; its `starter` must fail.** CI executes TypeScript blocks; rust and buildable are graded at runtime (fail-closed), not in CI or at sync.
- **Answer keys are public by design.** Grading is by sandboxed execution, not secrecy.
- **`openEnded` never mints XP.** It's a reflection: one learner message, one AI reply.
- **`course.creator` is the author's wallet and is immutable.** It maps straight to `Course.creator` on-chain (there is no `instructor` indirection — the `instructors/` folder was removed). Must be on-curve. Changing it after creation costs a full close-and-recreate, so mainnet courses must be created with the final wallet.
- **`trackId` / `trackLevel` are equally immutable.** They order the catalog; get them right before a course is created on-chain.
- **A translation is an overlay, never a second course.** `courses/<slug>/l10n/<locale>/` holds display strings and images only — never ids, slugs, `skills`, block keys or order, quiz `correct` flags, XP/creator/track fields, code `starter`/`solution`, or test `input`/`expectedOutput`. Duplicating a course to translate it forks its PDA, enrolment, slot bitmap and XP ledger, and two `course_id`s can never be merged.
- **Never name a file `course.yaml`, `lesson.yaml` or `slots.lock.json` inside `l10n/`.** The compiler matches those names by suffix at *any* depth and would ship the overlay as a duplicate document. Content-lint escalates the two `.yaml` names to an error; **it cannot see `slots.lock.json`** — that scan only walks `.yaml`, a gap the app repo documents in `repo-paths.ts`. Structured strings go in `strings.yaml`.
- **`sourceLocale` is the language a course is written in, and is set once at creation.** Fallback is always *requested → sourceLocale*, never → `en`; most live courses are PT-BR originals with no English version at all.

## Validate locally

```bash
git clone https://github.com/solanabr/superteam-academy
cd superteam-academy && pnpm install
pnpm --filter @superteam-lms/content-lint exec tsx src/cli.ts /path/to/courses-academy
```

Exit 0 = zero errors. `notice`/`warning` never fail.

## Provenance

This tree was originally extracted from the CMS that the platform used before content moved to git. That CMS is gone — this repo is now the only source. Do not look for an upstream to re-sync from.

## Not yet wired end-to-end

`course.creator` flows to `Course.creator` on-chain, but the `wallet → platform user` linkage (`profiles.wallet_address`) is what makes a creator's name and profile render in the app — a course whose creator wallet has no linked profile still works, it just shows the raw address.

**The creator wallet IS the teacher pointer** (owner ruling, 2026-07-28). There is no teacher registry file and no `githubId` indirection: `course.creator` → `profiles.wallet_address` → the teacher's platform account is the whole model. The one requirement it puts on you is real: a course must be created on-chain with the instructor's **actual** wallet the first time, because `creator` is immutable afterwards.

**Translations are authorable but inert.** `sourceLocale` is stripped by the app's non-strict `Course` Zod, and the compiler does not read `l10n/` at all, so an overlay stages without shipping. Expect a content-lint `warning` on any real course's `l10n/<locale>/strings.yaml` (*unclassified content file*) — it is benign, and the fix is app-side (`classify()` needs the path), never moving the file.

**This repo picked Candidate B; the app spec recommends A.** `docs/superpowers/specs/2026-07-27-content-i18n-mechanism.md` in the app repo is design-only, pending owner decision D-5, and recommends per-lesson overlays (`lessons/<dir>/l10n/<locale>.yaml` plus `intro.<locale>.md` siblings). We shipped its Candidate B — a course-level mirrored subtree — named `l10n/` rather than `i18n/`. The reason is `gate5-orphans.ts`: it errors on any file in a lesson directory that no block references, and its allowlist is `lesson.yaml` and `*.quiz.yaml` only, so Candidate A is two gate-5 errors per translated lesson while Candidate B needs no gate change. Do not "correct" this tree toward the spec without reopening D-5.
