# Wayfinder Solver

TODO

## Tech stack

TODO

### Workspace (`crates/`)

TODO. `crates/` is empty; the root package is the only one so far.

## Common commands

- `just` lists all recipes. The root justfile imports the `.just/` modules.
- `just wk::new_ui <branch>` creates a worktree off origin/main and opens a zellij dev tab. It
  wraps worktrunk. `wk::new` skips the tab; `wk::list` and `wk::rm` inspect or remove.
- `just check::all` runs fmt, clippy, and test.

## Required reading

These docs are not background. When one applies, read the whole thing before writing any code.
They set the conventions you follow, not material for you to summarize.

- Every task: `docs/CONTEXT.md`. It fixes one name per concept, so code, docs, and conversation
  agree on what things are called.
- Before writing or changing tests, or designing code that has to be testable: `docs/testing.md`.

## Code comments

Default is no comment. Docstrings come before inline comments. If the explanation is durable, it
belongs in a docstring, not above a line.

When a comment is warranted:

- Professional tone. No chatty asides ("and friends", "gotcha"), no editorializing, no narration
  of the change being made.
- Follow the pattern already in the file and the module. If sibling functions carry no docstring,
  the new one gets none either.
- Concise. When rewriting an existing comment, make it shorter, not longer.
- Only what reading the code cannot tell you. A comment restating the next line is noise.
- Do not couple a comment to code that is not directly below it. No comment that describes a
  caller, a sibling module, or a future edit.
- The comment must stand on its own and cost nothing to keep accurate. A comment that goes stale
  the next time nearby code moves is a bad comment. Drop it instead.
- Never reference untracked files: ADRs, CONTEXT.md, worktree-parent specs, wayfinder maps,
  handoff docs. A handoff or plan doc claiming an exception is not license.
- No em dashes, en dashes, or arrows.

These rules cover all new text, not only inline comments: docstrings, interface `description=`
strings, json5 `//` comments, test comments. Text moved from another file counts as new text and
gets restyled during the move.

## Always check your work

After writing or changing any code, run `just check::all` and confirm it passes. That is how you
verify the code matches your intent. The work is not done until it is green.

## Git rules

- Never commit your own code before the developer has reviewed it. Present the work for review
  first. The `dev-tdd` per-slice commit follows this rule: the developer's review is the gate.
- Run `just check::all` and confirm it passes before you commit.
- Rebasing: `git fetch` first, then rebase.
- Opening a PR: fetch, rebase, stage, commit, push, create the PR. In that order.

## Plan before implementing, then implement with TDD

Implementation is the last step, never the first. Settle the shape of the work and every open
decision with me before writing code.

Planning, sized to the work:

- A design or plan I want stress-tested: `/grill`. It walks the decision tree and checks the plan
  against the glossary and the ADRs tracked in this repo.
- Work too big for one session, destination still foggy: `/wayfinder`. I have to invoke it.
  Propose it, do not try to call it.

Never rush into implementation carrying uncertainty. If a choice has more than one defensible
answer and picking wrong means rework, stop and ask me. Do not pick silently and report the
assumption afterwards. Unknowns that touch one call site only are yours to decide. Anything
shaping an interface, a schema, a stored format, or a policy is mine.

Once the plan is settled, implement with the `tdd` skill: red, green, refactor. No permission
checkpoints inside it.

## No AskUserQuestion tool

Never call `AskUserQuestion`. The picker limits my answer to the options you guessed at, and my
answer is usually neither of them, so it costs a turn instead of saving one. It also cuts the
conversation down to a menu.

Ask in prose at the end of the message: the question, the tradeoff, your recommendation. I answer
in free text. This holds in grill and interview flows too, where the temptation is strongest.

Only exception: I explicitly ask for the picker in that message.
