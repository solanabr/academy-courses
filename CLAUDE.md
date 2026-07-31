# courses-academy — agent guide

Source of truth for all Superteam Academy content. This repo is data, not code: plain YAML, Markdown, and source files.

**How it reaches learners:** merged content here *stages*; the app repo (`solanabr/superteam-academy`) compiles this tree into a committed bundle and activates it by bumping `content.lock`. Courses are then deployed on-chain from the admin panel. There is no CMS in the path.

**What to write** — the approved catalog, per-course syllabi, lesson shapes, and the exact APIs/versions to teach — is [courses/CATALOG.md](./courses/CATALOG.md). Read it before authoring or restructuring a course.

**When editing a folder, read that folder's `README.md` and its `schema/<type>.schema.json` first** — they hold the fields, the controlled vocabulary, and the worked examples:

| Editing… | Read |
|---|---|
| a course / lesson / slots | [courses/README.md](./courses/README.md) + `schema/course.schema.json`, `schema/lesson.schema.json` |
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
