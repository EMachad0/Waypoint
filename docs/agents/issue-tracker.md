# Issue tracker: local markdown

Issues and specs for this repo live as tracked markdown files under `docs/issues/`. There is no
external tracker. Everything is greppable from the worktree you are standing in.

## Layout

One effort per directory, flat inside it:

```
docs/issues/<effort-slug>/
├── spec.md          the spec, when one exists
├── map.md           the Map, when the effort came through /waypoint
├── 01-<slug>.md
└── 02-<slug>.md
```

Issues are numbered from `01` within their effort directory. Never collect them into one combined
file.

## Issue file

A `Status:` line near the top carries the triage role, using the strings in
`docs/agents/triage-labels.md`. A `Type:` line carries the ticket type when the effort is a Map
(`research`, `prototype`, `grilling`, `spec`, `task`, `implementation`, `landing`). A
`Blocked by: 01, 04` line lists the issues that must land first. Conversation appends to the bottom
under a `## Comments` heading.

## Lifecycle

Creating and closing both happen through git, so the queue moves with the code.

- **Publish to the issue tracker**: write a new file under `docs/issues/<effort-slug>/`, creating
  the directory if needed.
- **Fetch the relevant ticket**: read the file. The user normally passes the path or the number.
- **Close an issue**: delete the file in the pull request that implements it. The merge is what
  closes it, so an issue cannot read as done while its work sits unmerged, and reverting the PR
  brings it back. History keeps the body.

A Ticket belonging to a Map is the exception: it is never deleted. Its Map points at it by path,
and those pointers have to stay resolvable for as long as the effort runs, so a Ticket closes by
its `Status:` line alone.

`just wk::new` branches from a fresh `origin/main`, so a new worktree always starts from the
current queue.

## What does not survive a branch

`Status: claimed` is advisory, not a lock. A worktree cannot see a claim made on another branch, so
two agents working in parallel worktrees can pick up the same issue. Coordinate out of band when
that is a real risk.

A Map is the file most exposed to this, since every resolved ticket appends to its Route so far.
One effort at a time is fine. Two efforts touching one Map from separate worktrees will conflict in
the same section, repeatedly.

For anything hot enough that the branch boundary hurts, the `*.ignore.*` and `*.ignore/` gitignore
patterns keep a file in the tree and out of git. Unlike a dotted directory, those stay visible to
`rg` by default.

## Waypoint operations

Used by `/waypoint`. The Map is `map.md`; its children are the numbered issue files beside it.

- **Map**: `docs/issues/<effort-slug>/map.md`, holding the Notes, Route so far, and Fog.
- **Child ticket**: `docs/issues/<effort-slug>/NN-<slug>.md`, with the question in the body, a
  `Type:` line, and a `Status:` line of `claimed`, `resolved`, `superseded`, or `wontfix`.
- **Blocking**: the `Blocked by:` line. A ticket is unblocked once every issue it names is closed.
- **Frontier**: scan the effort directory for issues that are open, unblocked, and unclaimed. First
  by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append
  a one-line gist and link to the Map's Route so far.
- **Supersede**: file the new ticket, append a `Superseded by: NN` line to the old one, set its
  `Status: superseded`, and delete its line from the Map's Route so far.
- **Rule out of scope**: set `Status: wontfix` and leave one line in the Map's Out of scope.
