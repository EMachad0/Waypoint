# Landing

A Waypoint effort's destination is landed work. This file says what landed means in this repo.

## Landed means merged

Work is landed when its pull request is merged into `main` with the pipeline green. This repo
builds a binary that goes nowhere else: no tag, no release artifact, no deployment. The merge is
the last event that changes what anyone gets from a clone.

Revisit this file the first time `waypoint` is published anywhere.

## Landing tickets

A Map does not end at the merge of its last implementation Ticket. The landing Ticket reads the
Route so far and asks whether any decision the effort ended on clears the bar in
`docs/adr/README.md`. Most efforts answer no.

When one does clear it, each Ticket the record draws from gains a `Recorded as:` line. Decisions
superseded along the way stay where they are, unpromoted.

The landing Ticket merges like any other, and that merge is where the Map reaches its destination.
