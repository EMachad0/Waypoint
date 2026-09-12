# Landing

Waypoint's destination is landed work. This file says what landed means in this repo.

## Landed means merged

Work is landed when its pull request is merged into `main` with the pipeline green. This repo
builds a binary that goes nowhere else: no tag, no release artifact, no deployment. The merge is
the last event that changes what anyone gets from a clone.

Revisit this file the first time `wayfinder_solver` is published anywhere.

## Landing tickets

Merging the last implementation Ticket is not the end of a Map. The landing Ticket reads the Route
so far and asks whether any decision the effort ended on clears the bar in `docs/adr/README.md`.
Most efforts answer no, and no is the expected answer: promotion is optional, and a record nobody
needed is noise in the one place that has to stay high signal.

When a decision does clear the bar, it gets one record, however many Tickets fed it, written for a
reader who never saw the effort, and each Ticket it drew from gains a `Recorded as:` line.
Decisions superseded along the way stay where they are, unpromoted.

The landing Ticket merges like any other, and that merge is where the Map reaches its destination.
