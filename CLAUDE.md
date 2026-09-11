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

When writing new comments:

- Sound **professional** and follow the existing codebase's comment style.
- Keep them **concise**.
- Comment **what is not easily inferred from the code** — the *why*, a non-obvious constraint, a
  subtle edge case — not what the code already says plainly.
- **Never reference ADRs or `docs/CONTEXT.md`.** State the *why* inline instead. Those docs get
  moved, renumbered, superseded, and deleted; a comment that points at one then has to be chased
  and updated, and a comment that leans on one has stopped explaining itself. (This applies to
  *code comments* only: docs may reference each other freely.)

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

## Agent Rules

- **Never use the `AskUserQuestion` tool.** If you need clarification, state your assumption and proceed. If you need to present options, list them in plain text output instead.
