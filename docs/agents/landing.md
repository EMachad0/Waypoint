# Landing

Waypoint's destination is landed work. This file says what landed means in this repo.

## Landed means merged

Work is landed when its pull request is merged into `main` with the pipeline green. This repo
builds a binary that goes nowhere else: no tag, no release artifact, no deployment. The merge is
the last event that changes what anyone gets from a clone.

Revisit this file the first time `wayfinder_solver` is published anywhere. Until then, a Map whose
last implementation Ticket merged has reached its destination.

## Landing tickets

With nothing to release, a landing Ticket carries no release procedure here. Give a Map one only
when reaching the destination takes work after the last merge, and say in the Ticket what that work
is.
