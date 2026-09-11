---
name: to-prd
description: Turn the current conversation context into a PRD and submit it as a GitHub issue. Use when user wants to create a PRD from the current context.
---

This skill turns the current conversation context and your understanding of the codebase into a
PRD. Do NOT interview the user. Synthesize what you already know.

## Process

1. Explore the repo to understand the current state of the codebase, if you have not already.

2. Sketch the major modules you will build or modify to complete the implementation. Look for
   chances to extract deep modules that can be tested in isolation.

   A deep module, as opposed to a shallow one, packs a lot of functionality behind a simple,
   testable interface that rarely changes.

   Check with the user that these modules match their expectations, and which ones they want tests
   written for.

3. Write the PRD using the template below and submit it as a GitHub issue.

<prd-template>

## Problem statement

The problem the user is facing, from the user's perspective.

## Solution

The solution to the problem, from the user's perspective.

## User stories

A LONG, numbered list of user stories. Each one in the format:

1. As an <actor>, I want a <feature>, so that <benefit>

<user-story-example>
1. As a mobile bank customer, I want to see the balance on my accounts, so that I can make better informed decisions about my spending
</user-story-example>

Make this list extensive enough to cover every aspect of the feature.

## Implementation decisions

The implementation decisions that were made. This can include:

- The modules to be built or modified
- The interfaces of those modules that will change
- Technical clarifications from the developer
- Architectural decisions
- Schema changes
- API contracts
- Specific interactions

Do NOT include specific file paths or code snippets. They go out of date fast.

## Testing decisions

The testing decisions that were made. Include:

- What makes a good test here (test external behavior, not implementation details)
- Which modules will be tested
- Prior art for the tests, meaning similar types of tests already in the codebase

## Out of scope

What this PRD does not cover.

## Further notes

Anything else worth recording about the feature.

</prd-template>
