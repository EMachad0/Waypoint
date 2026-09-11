# CONTEXT.md format

The glossary lives at `docs/CONTEXT.md`. This repo is a single context spanning the whole
workspace, so there is exactly one glossary and no context map.

## Structure

```md
# {Context Name}

{One or two sentences on what this context is and why it exists.}

## Language

**Order**:
{A concise description of the term}
_Avoid_: Purchase, transaction

**Invoice**:
A request for payment sent to a customer after delivery.
_Avoid_: Bill, payment request

**Customer**:
A person or organization that places orders.
_Avoid_: Client, buyer, account

## Relationships

- An **Order** produces one or more **Invoices**
- An **Invoice** belongs to exactly one **Customer**

## Example dialogue

> **Dev:** "When a **Customer** places an **Order**, do we create the **Invoice** immediately?"
> **Domain expert:** "No. An **Invoice** is only generated once a **Fulfillment** is confirmed."

## Flagged ambiguities

- "account" was used to mean both **Customer** and **User**. Resolved: these are distinct concepts.
```

## Rules

- Be opinionated. When several words exist for the same concept, pick the best one and list the rest
  as aliases to avoid.
- Flag conflicts explicitly. If a term is used ambiguously, call it out under "Flagged ambiguities"
  with a clear resolution.
- Keep definitions tight. One sentence max. Define what it IS, not what it does.
- Show relationships. Use bold term names and express cardinality where it is obvious.
- Only include terms specific to this project's context. General programming concepts such as
  timeouts, error types, and utility patterns do not belong, however heavily the project uses them.
  Before adding a term, ask whether it is unique to this context or a general programming concept.
  Only the first belongs.
- Group terms under subheadings when natural clusters emerge. If all terms belong to one cohesive
  area, a flat list is fine.
- Write an example dialogue. A conversation between a dev and a domain expert that shows how the
  terms interact naturally and clarifies the boundaries between related concepts.
