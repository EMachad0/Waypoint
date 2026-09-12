# Waypoint forks wayfinder to run the whole arc

The installed `wayfinder` skill plans and then stops: its map is done once the route is clear, and
someone else builds what it decided. That handoff throws away the context the map spent sessions
accumulating, so this repo forked it as `waypoint` in `.agents/skills/waypoint/`, extending one map
from the rough idea through prototyping, implementation, review, and landing.

## The name

A waypoint is a stopping point on the life route of the software. It names the next step and the
ones already behind it at the same time, and it is tangible: something that truly landed, not a
decision about what to land. The near-rhyme with wayfinder is deliberate, since the charting half
is inherited whole.

## Considered options

Using `wayfinder` unchanged and handing off at the route. Rejected: the handoff is the expensive
moment, and the next agent re-derives what the map already knew.

Keeping upstream installed and layering a doc of deltas over it. Rejected: two artifacts in open
disagreement, and `docs/skills-and-docs.md` already forbids a doc duplicating or overriding skill
content.

## Consequences

Decisions are fluid until work lands, so a map supersedes its own closed tickets as the build
teaches it things, and `docs/issues/` became a permanent record rather than a queue that empties.

Upstream improvements no longer arrive for free. `c18450b` vendors the unedited skill, so
`git diff c18450b -- .agents/skills/waypoint` is the fork's delta, and re-syncing means vendoring a
newer upstream onto a scratch branch and reconciling by hand.

The repo now ships a skill of its own, which `.claude/skills` symlinks so both harnesses load it.
