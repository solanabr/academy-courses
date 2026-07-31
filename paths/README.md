# Learning Paths

One YAML file per path. A path is an ordered shelf of courses shown on the platform (e.g. "Solana Core").

```yaml
id: path-solana-core                # path-<kebab>, ≤ 128 UTF-8 bytes
                                    # NOTE the `path-` prefix — it deliberately
                                    # does not match the type name "learningPath"
slug: solana-core
title: Solana Core
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
id: path-frontend
title: Frontend
difficulty: intermediate
# its only course moved into another path; the id stays because path ids are
# permanent once live
retired: true
courses: []
```

**Un-retiring is allowed.** When a course exists for a retired shelf, drop `retired` and list the course in the same change. Retired paths stay in the repo rather than being deleted, because the id is permanent.

`path-completed` achievements target a path by id, and CI (gate 4a) errors if an achievement points at a missing **or empty** path — a learner can never complete an empty shelf, so retiring a path means re-homing any achievement that referenced it.

## Rules

- `courses` entries must be real course ids. (CI checks course→module→lesson and path→course references and errors on a missing course.)
- `id` is permanent. Never rename a live path.
