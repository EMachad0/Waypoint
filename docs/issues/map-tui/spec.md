# Terminal shell for the map viewer

Status: ready-for-agent

## Problem statement

The workspace has no terminal UI and no place to put one. `crates/` does not exist, the root package
is a `cargo new` hello world, and nothing proves a terminal can be opened, drawn to, and restored
without wrecking the user's shell. Every later question about viewing a Map sits behind that.

## Solution

A library crate `map_tui` holding the terminal application, and a root binary that does nothing but
start it. Running the binary clears to the alternate screen, draws one bordered block, and quits on
`q`, `Esc` or `Ctrl-C`, restoring the terminal on the way out and on panic. No Map is read or drawn
yet.

## Implementation decisions

### Workspace

The root `Cargo.toml` keeps the `wayfinder_solver` package and gains `[workspace] members =
["crates/*"]`. `docs/adr/0001-workspace-shape.md` records why the root is both.

### The crate

`crates/map_tui`, package name `map_tui`, a library. It is named for the Map it will eventually
render, per the glossary. It renders; it never fetches, parses or writes a Map.

### Dependencies

ratatui 0.30.2, declared once in `[workspace.dependencies]` at the root so a second crate cannot
drift, with `map_tui` inheriting it. `cargo add` has no flag that writes into
`[workspace.dependencies]` (checked against cargo 1.96), so write the root entry by hand, then run
`cargo add -p map_tui ratatui`, which picks up the workspace entry and writes `workspace = true`.

Default features. No direct `crossterm` dependency: use `ratatui::crossterm`, so the backend and the
event types can never resolve to different crossterm versions.

### Public interface of `map_tui`

```rust
pub struct App;

impl App {
    pub fn new() -> Self;
    pub fn handle_key(&mut self, key: KeyEvent);
    pub fn draw(&self, frame: &mut Frame);
    pub fn should_quit(&self) -> bool;
}

pub fn run() -> io::Result<()>;
```

`App` is state, event handling and drawing, with no terminal in it, which is what makes it testable.
`run` owns the terminal: `ratatui::run` wraps the loop so raw mode and the alternate screen are
restored on early return and on panic, not only on the happy path.

ratatui types appear in these signatures on purpose. The crate exists to draw with ratatui, and a
local `Key` or `Action` enum with one implementation behind it would be indirection for its own
sake. Wrapping the dependency is planned, but later, when a second consumer or a second backend
makes it pay.

### Behavior

The loop blocks on `ratatui::crossterm::event::read`, so the process is idle between keystrokes.
There is no tick and no polling timeout; add one when something animates.

`q`, `Esc` and `Ctrl-C` set the quit flag. Every other key is ignored. The frame is a single
bordered block filling the area, titled `map_tui`.

### Errors

`std::io::Result` throughout. Everything that can fail here is terminal I/O. color-eyre and thiserror
are coming, but adding an error framework before there is a second kind of failure is a decision
made too early.

### The binary

`src/main.rs` becomes `fn main() -> io::Result<()> { map_tui::run() }`.

### Docs

`CLAUDE.md` currently reads `TODO` under Tech stack and claims `crates/` is empty. Both become
accurate in the implementation commit: the workspace layout, ratatui as the UI library, and what
`map_tui` is for. Nothing aspirational.

## Testing decisions

Four tests. `docs/testing.md` governs the mechanics.

Unit tests in `crates/map_tui`, in a `#[cfg(test)] mod tests` block opening with `use super::*`:

1. `q` sets the quit flag.
2. `Esc` sets the quit flag.
3. `Ctrl-C` sets the quit flag.
4. An unrelated key leaves it unset.

One end-to-end test in `crates/map_tui/tests/`, driving `App` through the public interface with
`ratatui::backend::TestBackend` at 20x5 and asserting the whole rendered buffer. While the content
is a placeholder, an exact buffer is the clearest statement of what a frame looks like. Once the
layout grows, swap it for property assertions rather than maintaining a wall of glyphs.

`run` is not tested. It needs a real terminal, and what is left in it after `App` is extracted is
lifecycle handled by ratatui.

## Out of scope

- Reading, parsing or rendering a real Map, or anything that touches `docs/issues/`.
- Layout beyond the single block: panes, scrolling, mouse, colors, themes.
- color-eyre, thiserror, logging, CLI arguments, async.
- Wrapping ratatui behind local types.
- Creating tracker labels or a `/wayfinder` Map.

## Further notes

Implement with the `tdd` skill, red then green then refactor. Run `just check::all` and confirm it
passes. Review with `/code-review` against `origin/main`.

Then commit on `tui_test_one`, delete this file in the same pull request (the merge is what closes
it), and open the PR into `main` following the order in `CLAUDE.md`: fetch, rebase, stage, commit,
push, create.

Facts checked while writing this, worth re-checking if it sits in the queue for long: ratatui 0.30.2
has an MSRV of 1.88 against the pinned 1.96 toolchain, re-exports crossterm as `ratatui::crossterm`,
exposes `TestBackend` from `ratatui::backend` under default features, and provides
`ratatui::run`, `init` and `restore` with a panic hook that restores the terminal.
