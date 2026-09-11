---
name: dev-tdd
description: Collaborative, pair-style test-driven development. The agent writes the tests, the developer writes the implementation, and they move through the task one vertical slice at a time. Use when building a feature or fixing a bug together with the developer, test-first.
---

# Collaborative TDD

A pair-style variant of TDD. The roles are fixed:

- The agent writes the tests. Behavior-focused. Never writes production code.
- The developer writes the implementation. Owns all production code, including the real interface.

You move through the task one vertical slice at a time, and every piece of work is reviewed before
you move on.

For how to write tests in this repo (layers, fakes, conventions, running them), read
`docs/testing.md`. This skill governs the workflow; that doc governs the mechanics.

## What a slice is

A slice is one code piece: a function, a method, or a cohesive cluster of behavior. It is the
vertical unit of work and the red-green unit, not the individual test. A single slice usually has
several tests covering happy and unhappy paths.

The task is built up slice by slice, in tracer-bullet order. The first slice is the thinnest
vertical that proves the path end to end. Later slices thicken it.

## Why this diverges from stock TDD

Stock TDD forbids "all tests, then all code" because batched tests describe imagined behavior. That
risk is gone within a slice: a prior grill already fixed the design (see Planning), so a slice's
tests describe known behavior rather than guesses. Batching a slice's tests is fine.

The risk only bites across slices, so the one hard rule is:

> Never pre-write tests for a future slice. Finish slice N completely, meaning tests,
> implementation, green, review, and commit, before touching slice N+1's tests.

## Planning (before any slice)

1. Assume a prior grill settled the design. If it did not, because the behavior is fuzzy or
   undecided, say so and recommend running `/grill` first. Do not slice blind.
2. Propose the slice breakdown: an ordered list of code pieces, with the tracer bullet first.
3. Wait for the developer to approve or reorder the breakdown before starting.

## The per-slice loop

For each slice, in order:

1. Propose the test list. State the behavior each test will pin, listing happy and unhappy paths
   explicitly so unhappy paths cannot be silently dropped. Wait for the developer to approve the
   list.
2. Write the tests. Tests only. They are behavior-focused and may not compile yet. The interface
   they assume is a proposal, not binding. Wait for the developer to review the tests.
3. The developer implements all production code and decides the real interface. It may diverge from
   what the tests assumed.
4. Reconcile the tests to the developer's actual interface. The developer edits them, or asks you
   to adapt them, until they compile and pass.
5. Review the developer's code once green:
   - Bug found: write a new test pinning the bug. If no test can express it, tell the developer in
     plain text.
   - Refactor, rename, or typo ideas, however small: tell the developer. The developer decides
     whether to apply them. Do not apply them yourself.
6. Close the slice once you fully agree that every raised issue is resolved, or the developer says
   to proceed. Commit the slice, then move to the next one.

## Rules

- Write tests, never production code. Refactors are suggestions for the developer to apply.
- One behavior per test. No cramming multiple behaviors into one test.
- Every slice's test list covers happy and unhappy paths explicitly.
- Stop and wait at each review checkpoint (test list, tests, post-green review). Proceed only on
  the developer's go.
- Never pre-write tests for a future slice.
- Commit only after the slice has been reviewed. See the git rules in CLAUDE.md.

## Checklist per slice

```
[ ] Test list proposed (happy + unhappy) and approved
[ ] Tests written, behavior-focused, one behavior each
[ ] Developer implemented; tests reconciled to the real interface
[ ] Tests green
[ ] Code reviewed; bugs pinned with tests, refactors offered to the developer
[ ] Slice committed; on to the next
```
