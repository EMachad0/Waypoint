# Monster Master

TODO

## Tech Stack

TODO

### Workspace (`crates/`)

| Crate           | Role                                                                       |
| --------------- | -------------------------------------------------------------------------- |

Glossary of these terms lives in `docs/CONTEXT.md`.

## Common commands

- `just` — list all recipes (root justfile just imports `.just/` modules)
- `just wk::new_ui <branch>` — create a worktree off origin/main and open a zellij dev tab (worktrunk-backed; `wk::new` skips the tab, `wk::list` / `wk::rm` inspect or remove)
- `just check::all` — fmt + clippy + test (skips the wasm-only Module)

## Required Reading

These docs are **not** optional background. The glossary applies to every task; the rest are matched
by task type. When one applies, you **MUST read the whole doc before writing any code** — they
encode conventions you are expected to follow, not summarize.

- Always, whatever the task: read `docs/CONTEXT.md`. It fixes the project's ubiquitous language, so
  that code, docs, and conversation use the same word for the same concept.
- Before writing or changing **tests**, or designing code that must be testable: read `docs/testing.md`

## Comment Rules

## Code comments

Default is no comment. Docstrings come before inline comments. If the explanation is
durable, it belongs in a docstring, not above a line.

When a comment is warranted:

- Professional tone. No chatty asides ("and friends", "gotcha"), no editorializing, no
  narration of the change being made.
- Follow the pattern already in the file and the module. If sibling functions carry no
  docstring, the new one gets none either.
- Concise. When rewriting an existing comment, make it shorter, not longer.
- Only information that cannot be inferred by reading the code. A comment restating what
  the next line does is noise.
- Do not couple a comment to code that is not directly below it. No comment that describes
  a caller, a sibling module, or a future edit.
- The comment must stand on its own and cost nothing to keep accurate. A comment that goes
  stale the next time nearby code moves is a bad comment; drop it instead.
- Never reference untracked files: ADRs, CONTEXT.md, worktree-parent specs, wayfinder maps,
  handoff docs. A handoff or plan doc claiming an exception is not license.
- No em dashes, en dashes, or arrows.

These rules cover all new text, not just Python inline comments: docstrings, interface
`description=` strings, json5 `//` comments, test comments. Text moved from another file
counts as new text and gets restyled during the move.

## Always Check Your Work

- After writing or changing any code, run **`just check::all`**
  and confirm it passes. It is how you verify the code matches your intent — do not consider
  work done until it is green.

## Git Rules

- **Never commit your own code before the developer has reviewed it.** Present the work for review
  first; commit only after it has been reviewed (the `dev-tdd` per-slice commit follows this — the
  developer's review is the gate).
- **Check your work before committing** — run `just check::all` and confirm it passes first.
- **Rebase** → always `git fetch` first, then rebase.
- **Open a PR** → `git fetch` → rebase → stage → commit → push → create the PR, in that order.

## Plan before implementing, then implement with TDD

Implementation is the last step, never the first. Before writing code, the shape of the
work and every open decision must be settled with me.

Planning, sized to the work:

- A design or plan I want stress-tested: `grill-with-docs` (checks it against the domain
  model and the docs) or `grill-me` (plain interrogation of the decision tree).
- Work too big for one session, destination still foggy: `/wayfinder`. I have to invoke it;
  it is not model-invocable. Propose it, do not try to call it.

Never rush into implementation carrying uncertainty. If a choice has more than one
defensible answer and picking wrong means rework, stop and ask me. Do not pick silently and
report the assumption afterwards. Unknowns that only affect one call site are yours to
decide; anything shaping an interface, a schema, a stored format, or a policy is mine.

Once the plan is settled, implement with the `tdd` skill: red, green, refactor. That is the
one pass referenced above. No permission checkpoints inside it.

## No AskUserQuestion tool

Never call `AskUserQuestion`. The option picker constrains my answer to the choices you
guessed at, and my answer is usually neither of them, so it costs a turn instead of saving
one. It also cuts the conversation down to a menu.

Ask in prose at the end of the message: the question, the tradeoff, and your recommendation.
I answer in free text. This holds in grill and interview flows too, where the temptation is
strongest.

Only exception: I explicitly ask for the picker in that message.

