# Waypoint forks wayfinder to carry work through to landing

The installed `wayfinder` skill plans and then stops. Its map is done once the route is clear, and
someone else builds what it decided. That handoff throws away the context the map spent sessions
accumulating, so this repo forked it as `waypoint` in `.agents/skills/waypoint/`, extending one map
from the rough idea through prototyping, implementation, review, and landing.

## Considered options

Using `wayfinder` unchanged and handing off at the route. Rejected: the handoff is the expensive
moment, and the next agent re-derives what the map already knew.

Keeping upstream installed and layering a doc of deltas over it. Rejected: two artifacts in open
disagreement, and `docs/skills-and-docs.md` already forbids a doc duplicating or overriding skill
content.

## Consequences

Decisions are fluid until work lands, so a map supersedes its own closed tickets as the build
teaches it things, and `docs/issues/` became a permanent record.

Upstream improvements no longer arrive for free. The first commit of pull request 6 vendors the
unedited skill. This repo squash merges, so that commit never lands on `main`, but GitHub keeps the
branch reachable and the baseline with it:

```sh
git fetch origin refs/pull/6/head
git diff "$(git rev-list --reverse FETCH_HEAD ^origin/main | head -1)" -- .agents/skills/waypoint
```

That diff is the fork's delta. Re-syncing means vendoring a newer upstream onto a scratch branch
and reconciling by hand.

The repo now ships a skill of its own, which `.claude/skills` symlinks so Claude Code and pi both
load it.
