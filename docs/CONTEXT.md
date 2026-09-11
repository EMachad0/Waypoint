# Wayfinder Solver

The context of reading wayfinding work: the charts drawn while finding a way to a destination, and
the decisions taken along the way.

Format rules live in the `domain-modeling` skill, in `CONTEXT-FORMAT.md`.

## Language

**Map**:
The chart of one effort's route from a loose idea to its destination.
_Avoid_: plan, board, graph, roadmap

**Ticket**:
One decision a **Map** waits on.
_Avoid_: task, card

**Issue**:
One unit of work filed under `docs/issues/`, deleted from the tree when the work lands.
_Avoid_: story, work item

## Relationships

- A **Map** indexes many **Tickets**
- A **Ticket** resolves exactly one decision
- Every **Ticket** is filed as an **Issue**, but most **Issues** carry work rather than a decision

## Example dialogue

> **Dev:** "When a **Map** is opened, does it carry the decisions or the **Tickets**?"
> **Domain expert:** "Both, and they are not the same thing. The **Map** gists a decision in one
> line and points at the **Ticket** that holds it. A **Ticket** still open is a decision nobody has
> made yet."

## Flagged ambiguities

- "map" was used for both the artifact and its rendering on screen. Resolved: a **Map** is the
  artifact, and the rendering is never called a map.
- "issue" sat on **Ticket**'s avoid list while naming the tracker's own directory. Resolved: the
  two are different things, and **Issue** now carries its own definition.
