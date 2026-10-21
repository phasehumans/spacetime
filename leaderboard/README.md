# Leaderboard submissions

Every row on [spacetime.trydecember.com/leaderboard](https://spacetime.trydecember.com/leaderboard)
comes from a JSON file in this directory. Nothing is posted to an API: a submission
is inert data until CI validates it and a human merges the pull request.

## Layout

```
leaderboard/submissions/<taskset-id>/<harness>__<model>.json
```

For example `leaderboard/submissions/v1-50-1a2b3c4d/claude-code__claude-sonnet-4-5.json`.
The taskset directory keeps runs from different task suites from ever being ranked
against each other.

## Producing one

```bash
spacetime eval-all
spacetime submit --report results/spacetime_results_*.json --user your-handle
```

`submit` validates the report, recomputes the score, and writes the file. Commit it
and open a pull request.

## What CI enforces

| Check | Why |
| --- | --- |
| `passed` / `total` / `pass_rate` recomputed from `run.results[]` | hand-edited headline numbers are rejected |
| `taskset_hash` matches the current `tasks/` | a run only competes against runs on the same suite |
| `run.results` length equals the taskset size | partial runs cannot claim a full-suite score |
| `task_id` values are unique and belong to the suite | no duplicated or invented tasks |
| one row per harness + model + taskset | ranking stays deterministic |
| `schema_version`, `tier`, `submitted_by` | provenance is never optional |

Run the same checks locally:

```bash
npm --prefix website run validate
```

## Tiers

- `community` — submitted by the runner, not reproduced by a maintainer. Rendered
  with a badge.
- `verified` — a maintainer or CI re-ran the suite on a pinned commit and container
  image. The badge is the only thing that distinguishes it from a self-reported run.

Changing a task's `prompt.txt`, `setup.sh`, `test.sh` or `meta.sh` changes the taskset
digest, which retires every existing run to a historical group. That is intentional:
the numbers stopped being comparable.
