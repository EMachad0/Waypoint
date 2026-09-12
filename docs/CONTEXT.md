# Wayfinder Solver

The context of reading wayfinding work: the charts drawn while finding a way to a destination, the
decisions settled and the changes built along the way.

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
One unit of work filed under `docs/issues/`, deleted from the tree when the work lands, unless a
**Map** points at it.
_Avoid_: story, work item

**Landing**:
Reaching the destination: the point where an effort's work counts as shipped, which
`docs/agents/landing.md` fixes for this repo.
_Avoid_: merging, shipping, done

## Relationships

- A **Map** indexes many **Tickets**
- A **Ticket** resolves exactly one question, whether that question is a decision or a change
- A **Map** is finished when its destination is reached, which is **Landing**
- Every **Ticket** is filed as an **Issue**, but an **Issue** outside a **Map** sits on no route

## Example dialogue

> **Dev:** "When a **Map** is opened, does it carry the decisions or the **Tickets**?"
> **Domain expert:** "Both, and they are not the same thing. The **Map** gists a decision in one
> line and points at the **Ticket** that holds it. A **Ticket** still open is a decision nobody has
> made yet."

> **Dev:** "The last **Ticket** merged, so the **Map** is done?"
> **Domain expert:** "Merged, not landed. **Landing** is whatever this repo counts as shipped, and
> it is the merge only because nothing here publishes anywhere yet."

## Flagged ambiguities

- "map" was used for both the artifact and its rendering on screen. Resolved: a **Map** is the
  artifact, and the rendering is never called a map.
- "issue" sat on **Ticket**'s avoid list while naming the tracker's own directory. Resolved: the
  two are different things, and **Issue** now carries its own definition.
- "landed" named both a merged pull request and a finished effort. Resolved: **Landing** is the
  destination, per repo, and a merge reaches it only where `docs/agents/landing.md` says so.
