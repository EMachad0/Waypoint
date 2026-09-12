---
name: waypoint
description: Carry an idea too big for one agent session from rough shape to landed work. The effort is a shared map of tickets on your issue tracker, resolved one at a time through grilling, prototyping, implementation, and review.
disable-model-invocation: true
---

A loose idea has arrived, too big for one agent session, and wrapped in fog: the way from here to the **destination** isn't visible yet. Waypoint is about finding that way and then walking it, not charging at the destination. This skill charts the way as a **shared map** on the repo's issue tracker, then works its **tickets** (a decision to settle, an artifact to make, a change to build) one at a time until the work lands.

The destination is **landed work**: the idea, built and shipped. What shipped means belongs to the repo. Read `docs/agents/landing.md` before charting and name the destination in its terms, whether that is a merged pull request, a tagged release, or a synced cluster. Naming it is the first act of charting, since it fixes the scope of every ticket after it.

## Plan and do

The map runs the whole effort, from the first grilling to the landed change. Planning and building interleave: grill, implement, grill again with what the implementation taught, implement again. That loop is the normal shape of the work, not evidence the planning failed. A map that settles everything before building anything is a waterfall with tickets.

So no decision is final until the work lands. Until then any of them can be amended or superseded, by what the build revealed, by a colleague's pushback, by a better idea arriving late (see [Superseding a decision](#superseding-a-decision)). Scope moves the same way: a map can grow new tickets after its first pull request merges.

## Refer by name

Every map and ticket is an issue, so it has a **name**: its title. In everything the human reads (narration, the map's Route-so-far), refer to it by that name, never by a bare id, number, or slug. A wall of `#42, #43, #44` is illegible; names read at a glance. The id and URL don't vanish; a name wraps its link, but they ride _inside_ the name, never stand in for it.

## The Map

The map is a single issue on this repo's issue tracker, labelled `waypoint:map`, the canonical artifact. Its tickets are child issues of the map.

The map is an **index**, not a store. It lists what each closed ticket resolved and points at the ticket that holds the detail; that detail lives in exactly one place, its ticket, so the map never restates it, only gists it and links.

**Where the map, its child tickets, blocking, and frontier queries physically live is tracker-specific.** This repo's tracker is `docs/agents/issue-tracker.md`. Its "Waypoint operations" section says how each of those is expressed here.

### The map body

The whole map at low resolution, loaded once per session. Open tickets are **not** listed: they are open child issues, found by query.

```markdown
## Destination

<the landed work this effort ends at, in the terms `docs/agents/landing.md` sets. One or two lines; every session orients to it before choosing a ticket.>

## Notes

<domain; skills every session should consult; standing preferences for this effort>

## Route so far

<!-- the index: one line per closed ticket, enough to judge relevance, then zoom the link for the detail the ticket holds. Live entries only: a superseded decision leaves this list -->

- [<closed ticket title>](link): <one-line gist of the answer, or of what was built>

## Not yet specified

<!-- see "Fog of war": in-scope fog you can't ticket yet; graduates as the frontier advances -->

## Out of scope

<!-- see "Out of scope": work ruled beyond the destination; closed, never graduates -->
```

### Tickets

Each ticket is a **child issue** of the map; the tracker's issue id is its identity. Its body is the question, sized to one 100K token agent session:

```markdown
## Question

<the decision, investigation, or change this ticket resolves>
```

Each ticket carries a `waypoint:<type>` label, one of `research`, `prototype`, `grilling`, `spec`, `task`, `implementation`, `landing` (see [Ticket Types](#ticket-types)).

A session **claims** a ticket by assigning it to the dev driving the map, **first**, before any work, so concurrent sessions skip it. That assignee _is_ the claim: an open, unassigned ticket is unclaimed.

Blocking uses the tracker's **native** dependency relationship: essential because it renders the frontier _visually_ in the tracker's own UI, so the human sees what's takeable without opening the map. Only a tracker that lacks native blocking falls back to a body convention. A ticket is **unblocked** when every ticket blocking it is closed; the **frontier** is the open, unblocked, unclaimed children, the edge of the known. Decisions and implementations sit on that frontier undifferentiated: both wait on a human, and neither outranks the other. What is takeable is whatever is unblocked.

The answer isn't part of the body; it's recorded on resolution (see [Work through the map](#work-through-the-map)). A **resource** created while resolving a ticket is linked from the issue, not pasted in; the tracker doc says where resources live.

## Ticket Types

Every ticket is either **HITL** (human in the loop, worked _with_ a human who speaks for themselves) or **AFK**, driven by the agent alone. A HITL ticket only resolves through that live exchange; the agent never stands in for the human's side of it (a grilling agent that answers its own questions has broken this).

- **Research** (AFK): Reading documentation, third-party APIs, or knowledge bases to surface a fact a decision waits on. Resolved by a subagent that calls the Skill tool with "research" and writes its findings as a resource. Use when knowledge outside the current working directory is required.
- **Prototype** (HITL): Raise the fidelity of the discussion by making a cheap, rough, concrete artifact to react to (an outline, a rough take, a stub, or UI/logic code) by calling the Skill tool with "prototype". Links the prototype as a resource. Use when "how should it look" or "how should it behave" is the key question.
- **Grilling** (HITL): Conversation. The default case. Always call the Skill tool twice, for "grilling" and "domain-modeling".
- **Spec** (HITL): Gather what the grilling and the prototypes settled into one statement of what gets built, by calling the Skill tool with "to-spec". Links the spec as a resource. Optional and rare. It earns its place when a change carries enough arbitrary decisions that every implementation ticket downstream would otherwise re-derive them.
- **Task** (HITL or AFK): Manual work that must happen before a _decision_ can be made: nothing to decide, prototype, or research, but the discussion is blocked until it's done. Signing up for a service so its API can be judged, provisioning access, moving data so its shape can be seen. It earns its place by unblocking a decision, which is what separates it from an implementation ticket. The agent drives it alone where it can (AFK); otherwise it hands the human a precise checklist (HITL). Resolved when the work is done; the answer records what was done and any resulting facts (credentials location, new URLs, row counts) later tickets depend on.
- **Implementation** (HITL): Build the change and get it reviewed, looping until the pull request merges. One ticket, one branch, one pull request; a change too big for one session is too big for one ticket, so split it. Call the Skill tool with "tdd" to build it and with "code-review" before handing it to the human. Review rounds are neither separate tickets nor separate states: the ticket carries the whole loop and resolves when the branch merges. Links the pull request from the ticket; the answer records whatever the build taught that other tickets now depend on.
- **Landing** (HITL or AFK): Take merged work the last stretch to landed, the way `docs/agents/landing.md` says: tagging, publishing, syncing, whatever this repo counts as shipped. It also weighs the decisions the effort ended on against the repo's bar for a decision record, and writes one for any that clears it. Most maps end on one of these, and some want one partway through. Resolved once the destination is reached; the answer records what shipped and where to see it.

## Fog of war

The map is _deliberately_ incomplete: don't chart what you can't yet see. Beyond the live tickets lies the **fog of war**: the dim view of decisions and investigations you can tell are coming but can't yet pin down, because they hang on questions still open. Resolving a ticket clears the fog ahead of it, graduating whatever's now specifiable into fresh tickets, one at a time, until the way to the destination is clear and no tickets remain.

The map's **Not yet specified** section is where that dim view is written down: the suspected question, the area to revisit later. It's the undiscovered frontier _toward_ the destination: everything here is in scope, just not sharp enough to ticket. Write as loosely or as fully as the view allows; it doubles as a signpost for collaborators reading where the effort is headed.

**Fog or ticket?** The test is whether you can state the question precisely now, _not_ whether you can answer it now.

- **Ticket when** the question is already sharp, even if it's blocked and you can't act on it yet.
- **Not yet specified when** you can't yet phrase it that sharply. Don't pre-slice the fog into ticket-sized pieces: it's coarser than a ticket, and one patch may graduate into several tickets, or none, once the frontier reaches it.

**Not yet specified** excludes what's already settled (Route so far), what's already a live ticket, and what's out of scope (the section after next).

Fog also re-forms behind you. Building a thing turns up what planning could not have known, and that discovery either graduates into new tickets or overturns a decision already closed. Both are ordinary. Record what the work taught, then make the map match it.

## Superseding a decision

A closed ticket's answer is what was true when it closed. When later work overturns it, don't edit that answer and don't reopen the ticket: **supersede** it with a new ticket that settles the question again.

- The new ticket's answer states what it supersedes and why the earlier answer no longer holds.
- The superseded ticket keeps its own answer untouched, gaining only a pointer to the ticket that replaced it.
- The superseded ticket's line leaves **Route so far**. The index carries live entries only, so the map still reads as the current state of the effort in a single pass, however many times it changed its mind.

Dropping the line loses nothing: the superseded ticket is still answered and still linked from its replacement, and the map's own history lives in the tracker.

A ticket is never deleted. Every pointer on the map has to stay resolvable, so a ticket that turns out wrong gets superseded and one that turns out to sit past the destination gets ruled out of scope. Both leave it in place, closed and readable.

## Out of scope

Fog only ever gathers _toward_ the destination. The destination fixes the scope, so work beyond it is **out of scope**: it isn't fog, and it doesn't belong in **Not yet specified**. It gets its own **Out of scope** section on the map: work you've consciously ruled out of _this_ effort. Scope, not sharpness, lands it here.

Out-of-scope work never graduates (the frontier stops at the destination), so it returns only if the destination is redrawn, and then as a fresh effort, not a resumption.

Ruling something out of scope is a scoping act, not a step on the route. When a ticket that already exists turns out to sit past the destination (mis-scoped in while charting, or exposed by a resolution), **close it** (a closed ticket is unambiguously off the frontier) and leave one line in the **Out of scope** section: the gist plus why it's out of scope, linking the closed ticket. It stays out of **Route so far**, which records the route actually walked; a scope boundary isn't a step on it.

## Invocation

Two modes. Either way, **never resolve more than one ticket per session**, with the exception of research tickets. That holds through a review round too: a session waiting on a reviewer waits rather than taking another ticket.

### Chart the map

User invokes with a loose idea.

1. **Name the destination.** Call the Skill tool twice, for "grilling" and "domain-modeling", to pin down the landed work this map ends at, in the terms `docs/agents/landing.md` sets. The destination fixes the scope, so it's settled first.
2. **Map the frontier.** Grill again, **breadth-first** this time: fan out across the whole space rather than deep on any one thread, surfacing the open decisions and the first steps takeable now. **If this surfaces no fog** (the way to the destination is already clear, the whole journey small enough for one session), you don't need a map. Stop and ask the user how they'd like to proceed.
3. **Create the map** (label `waypoint:map`): Destination and Notes filled in, Route-so-far empty, the fog sketched into **Not yet specified**.
4. **Create the tickets you can specify now** as child issues of the map, then wire blocking edges in a **second pass** (issues need ids before they can reference each other). Wiring sorts them into the frontier and the blocked; everything you can't yet specify stays in the fog: the **Not yet specified** section.
5. **Fire the research subagents.** For each `research` ticket you just created, spin up a subagent that calls the Skill tool with "research" to resolve it in parallel, writing its findings as a resource with a context pointer from the ticket.
6. Stop: charting is one session's work; it hand-resolves nothing.

### Work through the map

User invokes with a map (URL or number). A ticket is **optional**: without one, you pick the next ticket, not the user.

1. Load the **map**: the low-res view, not every ticket body.
2. Choose the ticket. If the user named one, use it. Otherwise take the first frontier ticket in order. **Claim it**: assign it to yourself before any work.
3. Resolve it. **Zoom as needed**: fetch the full body of any related or closed ticket on demand; call the Skill tool for whichever skills the `## Notes` block names. If in doubt, call the Skill tool twice, for "grilling" and "domain-modeling".
4. Record the resolution: post the answer as a **resolution comment**, **close** the issue, and **append a context pointer** to the map's Route so far.
5. Add newly-surfaced tickets (create-then-wire); graduate any fog the answer has made specifiable, clearing each graduated patch from **Not yet specified** so it lives only as its new ticket. If the answer reveals that a ticket (this one or another) sits beyond the destination, **rule it out of scope** rather than resolving it on the route. If it overturns decisions already closed, **supersede** them, and re-specify whichever tickets rested on them.

The user may run unblocked tickets in parallel, so expect other sessions to be editing the tracker concurrently.
