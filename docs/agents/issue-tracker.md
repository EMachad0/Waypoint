# Issue tracker: local markdown

Issues for this repo live as tracked markdown files under `docs/issues/`. There is no external
tracker. Everything is greppable from the worktree you are standing in.

Nothing here is ever deleted. An issue closes by its `Status:` line, so `docs/issues/` is the
record of what this repo decided and built, not a queue that empties.

## Layout

One effort per directory:

```
docs/issues/<effort-slug>/
├── map.md                     the Map, when the effort came through /waypoint
├── tickets/
│   ├── 01-<slug>.md
│   └── 02-<slug>.md
└── resources/
    ├── <name>_spec.md
    ├── <name>_research.md
    └── sample_payload.json
```

Tickets are numbered from `01` within their effort. Never collect them into one combined file.

## Resources

Whatever an effort needs to carry between sessions: writeups, images, samples, probe scripts,
examples. The list stays flat until it hurts, and each file is named for what it is. The `_spec`,
`_research`, and `_prototype` suffixes are a suggestion for markdown writeups, not a rule.

The ticket that produced a resource links it. That link runs one way, so a resource carries no
back-pointer to its ticket.

Resources are tracked git files, so a secret never goes in one. Committing a credential puts it in
every clone permanently, and no later commit takes it back. Record where a credential lives, the
secret manager path or the env var name, never its value. Anything that has to sit on disk
unversioned goes in `resources/<name>.ignore.<ext>`. Fixtures carry synthetic values, never a copy
of production data.

## Issue file

A `Status:` line near the top carries the triage role or the lifecycle state, using the strings in
`docs/agents/triage-labels.md`. A `Type:` line carries the ticket type when the effort has a Map
(`research`, `prototype`, `grilling`, `spec`, `task`, `implementation`, `landing`). A
`Blocked by: 01, 04` line lists the tickets that must close first. Conversation appends to the
bottom under a `## Comments` heading.

## Lifecycle

Creating and closing both happen through git, so the record moves with the code.

- **Publish to the issue tracker**: write a new file under `docs/issues/<effort-slug>/`, creating
  the directory if needed.
- **Fetch the relevant ticket**: read the file. The user normally passes the path or the number.
- **Close an issue**: record the answer on it and set its `Status:`, in the pull request that
  implements it. The merge is what closes it, so an issue cannot read as done while its work sits
  unmerged, and reverting the PR reopens it.

Files are never deleted. A Map points at its tickets by path and an ADR points back at the ticket
it came from, so those paths stay resolvable long after the effort ends.

`just wk::new` branches from a fresh `origin/main`, so a new worktree always starts from the
current record.

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

Used by `/waypoint`. The Map is `map.md`; its tickets are the numbered files under `tickets/`.

- **Map**: `docs/issues/<effort-slug>/map.md`, holding the Notes, Route so far, and Fog.
- **Child ticket**: `docs/issues/<effort-slug>/tickets/NN-<slug>.md`, with the question in the
  body, a `Type:` line, and a `Status:` line.
- **Blocking**: the `Blocked by:` line. A ticket is unblocked once every ticket it names is closed.
- **Frontier**: scan `tickets/` for tickets that are open, unblocked, and unclaimed. First by
  number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append
  a one-line gist and link to the Map's Route so far.
- **Supersede**: file the new ticket, append a `Superseded by: NN` line to the old one, set its
  `Status: superseded`, and delete its line from the Map's Route so far.
- **Rule out of scope**: set `Status: wontfix` and leave one line in the Map's Out of scope.
- **Promote a decision**: optional, at landing. A decision clearing the bar in
  `docs/adr/README.md` gets one record in `docs/adr/`, however many tickets fed it, and each ticket
  it came from gains a `Recorded as: adr/NNNN-<slug>.md` line. Most efforts promote nothing.

An ADR is authoritative for what is true now. A ticket's answer is dated history, so it is never
edited to match a later ADR.
