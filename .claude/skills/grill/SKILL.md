---
name: grill
description: Grilling session that challenges your plan against this repo's tracked domain model (docs/CONTEXT.md) and decisions (docs/adr/), updating both inline as decisions crystallise. Use when stress-testing a plan against the project's language and documented decisions.
---

<what-to-do>

Interview me relentlessly about every aspect of this plan until we reach a shared understanding.
Walk down each branch of the design tree, resolving dependencies between decisions one by one. For
each question, give your recommended answer.

Ask the questions one at a time. Wait for feedback on each one before continuing.

If exploring the codebase can answer a question, explore the codebase instead of asking.

</what-to-do>

<supporting-info>

## Use this skill, not `grill-with-docs`

The user-level `grill-with-docs` skill keeps the glossary and the ADRs in the parent folder of the
repo, outside any working tree. Wayfinder Solver tracks both inside the repo, so that skill's paths
and its "never write these files inside the repo" instruction are wrong here. In this repo, always
use `/grill`.

## Domain awareness

While exploring the codebase, look for the existing documentation too.

### File structure

The glossary and the ADRs are tracked files in the repo:

```
docs/
├── CONTEXT.md          the glossary
└── adr/
    ├── 0001-<slug>.md
    ├── 0002-<slug>.md
    └── ...
```

Tracking them in the repo versions them alongside the code they describe, puts them in the same
pull request as the change that motivated them, and keeps them on the current branch instead of
shared by every worktree at once.

Both already exist, so append to them. The glossary is still empty, so add terms one at a time as
they get resolved rather than seeding it. Never create a second glossary elsewhere in the tree,
and never split `docs/CONTEXT.md` into per-directory glossaries. This repo is one context spanning
the whole workspace.

### Writing to them is writing to the branch

Anything this skill writes lands in the working tree, so the project's usual rules apply:

- It shows up in `git status` and belongs in the pull request that motivated the decision.
- Per CLAUDE.md, it is not committed until the developer has reviewed it.
- Other worktrees see it only once the branch merges.

## During the session

### Challenge against the glossary

When the user uses a term that conflicts with the language already in `docs/CONTEXT.md`, call it
out immediately. "Your glossary defines 'cancellation' as X, but you seem to mean Y. Which is it?"

### Sharpen fuzzy language

When the user uses a vague or overloaded term, propose a precise canonical one. "You're saying
'account'. Do you mean the Customer or the User? Those are different things."

### Discuss concrete scenarios

When the conversation turns to domain relationships, stress-test them with specific scenarios.
Invent scenarios that probe edge cases and force the user to be precise about where one concept
ends and the next begins.

### Cross-reference with code

When the user states how something works, check whether the code agrees. If you find a
contradiction, surface it: "Your code cancels entire Orders, but you just said partial cancellation
is possible. Which is right?"

### Update docs/CONTEXT.md inline

When a term is resolved, update `docs/CONTEXT.md` right there. Do not batch these up. Capture them
as they happen. Use the format in [CONTEXT-FORMAT.md](./references/CONTEXT-FORMAT.md).

`docs/CONTEXT.md` holds no implementation details. It is not a spec, a scratch pad, or a place to
park implementation decisions. It is a glossary and nothing else.

### Offer ADRs sparingly

Only offer to create an ADR when all three are true:

1. Hard to reverse. Changing your mind later costs something real.
2. Surprising without context. A future reader will wonder why it was done this way.
3. The result of a real trade-off. There were genuine alternatives and you picked one for specific
   reasons.

If any of the three is missing, skip the ADR. Use the format in
[ADR-FORMAT.md](./references/ADR-FORMAT.md).

</supporting-info>
