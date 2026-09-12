# Domain docs

How the engineering skills consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- `docs/CONTEXT.md`: the glossary, one canonical name per concept. This repo is a single context
  spanning the whole workspace, so there is one glossary and no `CONTEXT-MAP.md`.
- `docs/adr/`: the ADRs that touch the area you are about to work in.

The glossary sits at `docs/CONTEXT.md`, not at the repo root where the skills default to looking.
`docs/skills-and-docs.md` fixes that location, and it wins. Never create a root `CONTEXT.md`, never
add a second glossary elsewhere in the tree, and never split this one per directory.

Do not propose writing an ADR upfront. The `domain-modeling` skill writes one when a decision
actually gets resolved, and a Waypoint effort may write one at landing. Neither is routine; see
`docs/adr/README.md` for the bar.

## File structure

```
/
├── docs/
│   ├── CONTEXT.md
│   └── adr/
│       ├── README.md
│       └── 0001-slug.md
└── src/
```

## Use the glossary's vocabulary

When your output names a domain concept, in an issue title, a refactor proposal, a hypothesis, or a
test name, use the term as `docs/CONTEXT.md` defines it. Do not drift to the synonyms the glossary
marks as aliases to avoid.

A concept missing from the glossary is a signal. Either you are inventing language the project does
not use, and should reconsider, or the gap is real and belongs to `domain-modeling`.

## Flag ADR conflicts

When your output contradicts an existing ADR, say so instead of overriding it silently:

> Contradicts ADR-0002 (single binary crate), but worth reopening because the TUI now needs a
> separate render target.
