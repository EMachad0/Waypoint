# Waypoint

The context of reading an effort's route: the charts drawn while finding a way to a destination,
and the decisions and changes made along it.

Format rules live in the `domain-modeling` skill, in `CONTEXT-FORMAT.md`.

## Language

**Map**:
The chart of one effort's route from a loose idea to its destination.
_Avoid_: plan, board, graph, roadmap

**Ticket**:
One unit of progress a **Map** waits on: a decision to settle, an artifact to make, or a change to
build.
_Avoid_: task, card

**Issue**:
One unit of work filed under `docs/issues/`, closed by its `Status:` line and kept in the tree
afterwards.
_Avoid_: story, work item

**Effort**:
One journey from a rough idea to landed work, charted by a **Map** and filed as one directory under
`docs/issues/`.
_Avoid_: project, initiative, epic

**Resource**:
An artifact a **Ticket** produced that later sessions need: a writeup, a sample, a script.
_Avoid_: asset, attachment

**Landing**:
The point where an effort's work counts as shipped. `docs/agents/landing.md` fixes that point for
this repo.
_Avoid_: merging, done

## Relationships

- An **Effort** has exactly one **Map**
- A **Map** indexes many **Tickets**
- A **Ticket** resolves exactly one question, settled by a decision or by a change
- A **Map** is finished at **Landing**, when its destination is reached
- Every **Ticket** is filed as an **Issue**, but an **Issue** outside a **Map** sits on no route

## Example dialogue

> **Dev:** "When a **Map** is opened, does it carry the decisions or the **Tickets**?"
> **Domain expert:** "Both, and they are not the same thing. The **Map** gists a decision in one
> line and points at the **Ticket** that holds it. A **Ticket** still open is work the **Map** is
> still waiting on."

> **Dev:** "The last **Ticket** merged, so the **Map** is done?"
> **Domain expert:** "A **Map** is done at **Landing**, and each repo fixes where that is. Here it
> is the merge, because nothing publishes anywhere yet."

## Flagged ambiguities

- "map" was used for both the artifact and its rendering on screen. Resolved: a **Map** is the
  artifact, and the rendering is never called a map.
- "issue" sat on **Ticket**'s avoid list while naming the tracker's own directory. Resolved: the
  two are different things, and **Issue** now carries its own definition.
- "landed" named both a merged pull request and a finished effort. Resolved: **Landing** is the
  destination, and each repo fixes where that is. A merge reaches it only where
  `docs/agents/landing.md` says so.
