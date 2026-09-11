---
name: to-issues
description: Break a plan, spec, or PRD into independently-grabbable GitHub issues using tracer-bullet vertical slices. Use when user wants to convert a plan into issues, create implementation tickets, or break down work into issues.
---

# To issues

Break a plan into independently grabbable GitHub issues using vertical slices, one tracer bullet
per issue.

## Process

### 1. Gather context

Work from whatever is already in the conversation context. If the user passes a GitHub issue number
or URL as an argument, fetch it with `gh issue view <number>`, including comments.

### 2. Explore the codebase (optional)

If you have not explored the codebase yet, do it now to understand the current state of the code.

### 3. Draft vertical slices

Break the plan into tracer-bullet issues. Each issue is a thin vertical slice that cuts through ALL
integration layers end to end, NOT a horizontal slice of one layer.

A slice is either HITL or AFK. HITL slices need a human in the loop, for an architectural decision
or a design review. AFK slices can be implemented and merged without human interaction. Prefer AFK
where possible.

<vertical-slice-rules>
- Each slice delivers a narrow but COMPLETE path through every layer (schema, API, UI, tests)
- A completed slice is demoable or verifiable on its own
- Prefer many thin slices over few thick ones
</vertical-slice-rules>

### 4. Quiz the user

Present the proposed breakdown as a numbered list. For each slice, show:

- Title: short descriptive name
- Type: HITL or AFK
- Blocked by: which other slices, if any, must complete first
- User stories covered: which user stories this addresses, if the source material has them

Ask the user:

- Does the granularity feel right, too coarse, or too fine?
- Are the dependency relationships correct?
- Should any slices be merged or split further?
- Are the right slices marked HITL and AFK?

Iterate until the user approves the breakdown.

### 5. Create the GitHub issues

For each approved slice, create a GitHub issue with `gh issue create`. Use the body template below.

Create issues in dependency order, blockers first, so you can reference real issue numbers in the
"Blocked by" field.

<issue-template>
## Parent

#<parent-issue-number> (if the source was a GitHub issue, otherwise omit this section)

## What to build

A concise description of this vertical slice. Describe the end-to-end behavior, not a layer-by-layer
implementation.

## Acceptance criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Blocked by

- Blocked by #<issue-number> (if any)

Or "None, can start immediately" if there are no blockers.

</issue-template>

Do NOT close or modify any parent issue.
