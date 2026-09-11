---
name: triage-issue
description: Triage a bug or issue by exploring the codebase to find root cause, then create a GitHub issue with a TDD-based fix plan. Use when user reports a bug, wants to file an issue, mentions "triage", or wants to investigate and plan a fix for a problem.
---

# Triage issue

Investigate a reported problem, find its root cause, and create a GitHub issue with a TDD fix plan.
This workflow is mostly hands-off. Keep questions to the user at a minimum.

## Process

### 1. Capture the problem

Get a brief description of the issue from the user. If they have not given one, ask ONE question:
"What's the problem you're seeing?"

Do NOT ask follow-up questions yet. Start investigating immediately.

### 2. Explore and diagnose

Use the Agent tool with subagent_type=Explore to investigate the codebase. You are looking for:

- Where the bug shows up (entry points, UI, API responses)
- What code path is involved (trace the flow)
- Why it fails (the root cause, not the symptom)
- What related code exists (similar patterns, tests, adjacent modules)

Look at:

- Related source files and their dependencies
- Existing tests: what is covered, what is missing
- Recent changes to affected files (`git log` on the relevant files)
- Error handling along the code path
- Similar patterns elsewhere in the codebase that work correctly

### 3. Identify the fix approach

From your investigation, determine:

- The minimal change that fixes the root cause
- Which modules and interfaces are affected
- Which behaviors need tests
- Whether this is a regression, a missing feature, or a design flaw

### 4. Design the TDD fix plan

Write a concrete, ordered list of RED-GREEN cycles. Each cycle is one vertical slice:

- RED: describe a specific test that captures the broken or missing behavior
- GREEN: describe the minimal code change that makes that test pass

Rules:

- Tests verify behavior through public interfaces, not implementation details
- One test at a time, in vertical slices. NOT all tests first, then all code
- Each test should survive an internal refactor
- Include a final refactor step if one is needed
- Durability: only suggest fixes that survive radical codebase changes. Describe behaviors and
  contracts, not internal structure. Tests assert on observable outcomes such as API responses, UI
  state, and user-visible effects, not on internal state. A good suggestion reads like a spec. A bad
  one reads like a diff.

### 5. Create the GitHub issue

Create a GitHub issue with `gh issue create` using the template below. Do NOT ask the user to review
it first. Create it and share the URL.

<issue-template>

## Problem

A clear description of the bug, including:

- What happens (actual behavior)
- What should happen (expected behavior)
- How to reproduce, if applicable

## Root cause analysis

What you found during the investigation:

- The code path involved
- Why the current code fails
- Any contributing factors

Do NOT include specific file paths, line numbers, or implementation details tied to the current code
layout. Describe modules, behaviors, and contracts instead, so the issue stays useful after a major
refactor.

## TDD fix plan

A numbered list of RED-GREEN cycles:

1. RED: write a test that [describes expected behavior]
   GREEN: [minimal change to make it pass]

2. RED: write a test that [describes next behavior]
   GREEN: [minimal change to make it pass]

...

REFACTOR: [any cleanup needed after all tests pass]

## Acceptance criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] All new tests pass
- [ ] Existing tests still pass

</issue-template>

After creating the issue, print the issue URL and a one-line summary of the root cause.
