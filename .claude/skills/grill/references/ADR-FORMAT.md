# ADR format

ADRs live in `docs/adr/` and are numbered in sequence: `0001-slug.md`, `0002-slug.md`, and so on.
The directory already exists and is tracked, so a new ADR is just a new file in it. None have been
written yet, so the first one is `0001-`.

## Template

```md
# {Short title of the decision}

{1-3 sentences: the context, what we decided, and why.}
```

That is the whole thing. An ADR can be a single paragraph. The value is in recording that a decision
was made and why, not in filling out sections.

## Optional sections

Include these only when they earn their place. Most ADRs will not need them.

- Status frontmatter (`proposed | accepted | deprecated | superseded by ADR-NNNN`), useful when
  decisions get revisited
- Considered options, only when the rejected alternatives are worth remembering
- Consequences, only when non-obvious downstream effects need calling out

## Numbering

Scan `docs/adr/` for the highest existing number and add one.

## When to offer an ADR

All three of these must be true:

1. Hard to reverse. Changing your mind later costs something real.
2. Surprising without context. A future reader will look at the code and wonder why on earth it was
   done this way.
3. The result of a real trade-off. There were genuine alternatives and you picked one for specific
   reasons.

If a decision is easy to reverse, skip it. You will just reverse it. If it is not surprising, nobody
will wonder why. If there was no real alternative, there is nothing to record beyond "we did the
obvious thing."

### What qualifies

- Architectural shape. "We're using a monorepo." "The write model is event-sourced, the read model
  is projected into Postgres."
- Integration patterns between contexts. "Ordering and Billing communicate via domain events, not
  synchronous HTTP."
- Technology choices that carry lock-in. Database, message bus, auth provider, deployment target.
  Not every library, only the ones that would take a quarter to swap out.
- Boundary and scope decisions. "Customer data is owned by the Customer context; other contexts
  reference it by ID only." The explicit noes are as valuable as the yeses.
- Deliberate deviations from the obvious path. "We're using manual SQL instead of an ORM because X."
  Anything a reasonable reader would assume the opposite of. These stop the next engineer from
  "fixing" something that was deliberate.
- Constraints not visible in the code. "We can't use AWS because of compliance requirements."
  "Response times must be under 200ms because of the partner API contract."
- Rejected alternatives when the rejection is non-obvious. If you considered GraphQL and picked REST
  for subtle reasons, record it. Otherwise someone will suggest GraphQL again in six months.

## Referencing ADRs

Docs may cite an ADR by path. Code comments may not. Per CLAUDE.md, a comment states its own why
inline, so it stays correct if an ADR is renumbered, superseded, or deleted.
