# Learning Paths

One YAML file per path. A path is an ordered shelf of courses shown on the platform (e.g. "Solana Core").

```yaml
id: path-zero-to-deployed           # path-<kebab>, ≤ 128 UTF-8 bytes
                                    # NOTE the `path-` prefix — it deliberately
                                    # does not match the type name "learningPath"
slug: zero-to-deployed
title: Zero to Deployed
description: Start here…            # optional
tag: Foundation                     # optional short label
order: 1                            # optional display order (lower first), default 0
difficulty: beginner                # beginner | intermediate | advanced
draft: false                        # optional, default false — see below
retired: false                      # optional, default false — see below
courses:                            # course ids, in display order
  - course-solana-fundamentals
  - course-building-first-program
```

## Empty shelves: `draft` vs `retired`

**A path with no courses must say why it is empty.** The app hides a shelf with zero courses — it reads neither flag — so the flags exist to record intent, and CI holds you to them.

| you mean | write | rule |
| --- | --- | --- |
| courses are on the way | `draft: true` | must actually list courses; `draft: true` + `courses: []` is a CI **error** |
| permanently emptied, id preserved | `retired: true` | `courses` must be empty; a retired path that still lists courses is a CI **error** |

Setting both is a warning — `retired` already means permanently hidden, so drop `draft`.

```yaml
id: path-example-shelf
title: Example Shelf
difficulty: intermediate
# its only course moved into another path; the id stays because this shelf
# shipped to learners and its id is public
retired: true
courses: []
```

**Un-retiring is allowed.** When a course exists for a retired shelf, drop `retired` and list the course in the same change.

**Retire vs delete.** `retired: true` is for a shelf that *shipped* — learners saw it, so the id stays reserved and the file stays in the repo. A shelf that never rendered to a learner should simply be **deleted**: there is no id to protect and no history to preserve. Owner ruling, 2026-07-31 — the seven empty paths carried over from the catalog migration (`path-ai-solana`, `path-defi`, `path-frontend`, `path-infrastructure`, `path-rust-programs`, `path-security`, `path-solana-core`) were deleted on those grounds, since the app never renders a shelf with zero courses. There are currently no retired paths in this repo; the flag stays in the schema for the first real sunset.

`path-completed` achievements target a path by id, and CI (gate 4a) errors if an achievement points at a missing **or empty** path — a learner can never complete an empty shelf, so retiring **or deleting** a path means re-homing any achievement that referenced it.

## Rules

- `courses` entries must be real course ids. (CI checks course→module→lesson and path→course references and errors on a missing course.)
- `id` is permanent. Never rename a live path.
