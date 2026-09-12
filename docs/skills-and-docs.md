# Skills and docs

## When to use

Read this when you are:

- Creating a new skill or documentation file
- Deciding whether a piece of knowledge belongs in a skill or a doc
- Getting oriented in where this project keeps its agent-facing knowledge

## Kinds of knowledge

This project keeps agent-facing knowledge in four places. Docs and skills are mechanisms. The
glossary and the decision records are single artifacts, each with one owner.

`docs/issues/` is not one of them. It holds the work queue, not knowledge: entries describe what is
not true yet, and each one is deleted when its work lands. Never read an issue as a statement of
current convention. `docs/agents/issue-tracker.md` governs it.

### Docs (`docs/`)

Static best-practice documents. Agents read them automatically, triggered by a CLAUDE.md directive
("Before creating or modifying X, read `docs/Y.md`"). Nobody has to invoke them.

- Location: `docs/<name>.md`
- Loaded when: CLAUDE.md tells the agent to read them, based on task type
- Content: conventions, patterns, rules, and anti-patterns to follow during implementation
- Examples: `docs/testing.md`, this doc

### Skills (`.agents/skills/` and `~/.agents/skills/`)

Interactive workflows invoked as `/skill-name`. A skill performs work: it runs commands, asks
questions, and writes files.

- Location: `~/.agents/skills/<skill-name>/SKILL.md` when the workflow is shared by every repo on
  the machine, `.agents/skills/<skill-name>/SKILL.md` when it belongs to this one. Claude Code
  reads each through a `skills` symlink, `~/.claude/skills` and `.claude/skills`
- Loaded when: the user invokes `/skill-name`, or the agent matches the description
- Content: instructions for an interactive workflow, not static reference
- Examples: `/grill-with-docs` interviews the user about a plan and updates the glossary and ADRs
  inline, `/to-spec` writes a spec, `/to-tickets` splits work into tickets

### Glossary (`docs/CONTEXT.md`)

The project's ubiquitous language: one canonical name per concept, plus the aliases to avoid.
CLAUDE.md marks it as the only always-read doc, because naming comes up in every task.

- Location: `docs/CONTEXT.md`. One glossary only. This repo is a single context spanning the whole
  workspace, so it is never split per crate or per directory
- Loaded when: always, per the CLAUDE.md required reading
- Content: terms, relationships, flagged ambiguities. No implementation details, no decisions, no
  rules
- Written by: `/grill-with-docs`, inline, as terms get resolved. Format lives in the
  `domain-modeling` skill, in `CONTEXT-FORMAT.md`

### Decision records (`docs/adr/`)

Numbered records of decisions that were hard to reverse, surprising without context, and the result
of a real trade-off. An ADR captures why a choice was made at a point in time, not how the code
behaves today.

- Location: `docs/adr/NNNN-slug.md`, numbered in sequence
- Loaded when: an agent needs the reasoning behind an existing decision, or `/grill-with-docs` is
  checking whether a question is already settled
- Content: append-only history. Supersede an old ADR with a new one instead of rewriting it to
  match the present
- Written by: `/grill-with-docs`, on the rare occasion it offers one. Format lives in the
  `domain-modeling` skill, in `ADR-FORMAT.md`
- Docs may cite an ADR by path. Code comments may not, per the comment rules in CLAUDE.md

## When to use which

| Situation                                                                     | Use      |
| ----------------------------------------------------------------------------- | -------- |
| Convention that applies whenever code is written (naming, structure, patterns) | Doc      |
| Interactive workflow that runs commands or asks questions                      | Skill    |
| Decision tree an agent should follow automatically                             | Doc      |
| Tool the user invokes on demand                                                | Skill    |
| Reference material (API patterns, schema placement, test layout)               | Doc      |
| Canonical name for a domain concept, or an alias to stop using                 | Glossary |
| Hard-to-reverse choice with real alternatives a reader would question          | ADR      |

Rule of thumb: a rule to follow during work is a doc. A task to perform on demand is a skill. What
to call something is the glossary. Why a settled choice was made is an ADR.

## Creating a new doc

1. Create `docs/<name>.md`
2. Start with a `## When to use` section listing the trigger conditions
3. Write rules as numbered headings under `## Critical rules`
4. End with a `## References` section linking to relevant external docs
5. Add a CLAUDE.md directive: `- Before creating or modifying <topic>: read \`docs/<name>.md\``

## Creating a new skill

1. Create `SKILL.md` under `.agents/skills/<skill-name>/` for a workflow this repo owns, or under
   `~/.agents/skills/<skill-name>/` for one every repo should get
2. Add frontmatter:

```yaml
---
name: skill-name
description: One-line description of what the skill does and when to invoke it.
---
```

3. Write the skill instructions below the frontmatter
4. Optional: add `allowed-tools` and `metadata` fields to the frontmatter
5. Reference files can live alongside `SKILL.md` in the skill's own directory

## Docs are live documents

Docs describe the project's current conventions and patterns. They are not write-once artifacts.
When you use a doc during implementation and find it outdated, incomplete, or contradicted by the
code, update it. Keeping docs accurate is part of the implementation work, not a separate task.

## Code examples in docs

Keep code examples accurate against the current codebase. When writing or updating them:

- Use valid Rust, TOML, or shell syntax
- Match the project's formatting conventions (`cargo fmt` for Rust)
- Keep examples minimal but syntactically complete

## Consistency rules

- Docs must not contradict each other. Each concept has one owner doc. Other docs reference it with
  a one-line reminder.
- Docs must not contradict skills. When a skill and a doc disagree, the doc wins on conventions and
  the skill wins on its own workflow.
- Tooling conventions: `cargo` for the Rust workspace, `just` for task running, `mise` for
  installing tools. Never hand-edit generated code; regenerate it.
