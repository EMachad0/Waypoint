# Five variants: theses and design notes

Throwaway prototype. The question: how should a Map render in ratatui?

Read [survey.md](survey.md) first for the crate and algorithm verdicts these notes lean on.

## What the fixtures actually look like

Measured over the 11 Maps in `docs/wayfinder` (the invocation said 12; there are 11).

| Property | Range | Consequence for layout |
|---|---|---|
| Tickets per Map | 6 to 16 | A whole Map fits on one screen only if a row is one line |
| DAG depth | 1 to 9 | Deep enough to be worth drawing |
| Layer widths | `[8,1,1,2,1,1]`, `[6,2,1,1,1,1,1]` | Bushy at the root, then a chain. A 2-D drawing is mostly empty |
| Maps with zero edges | 2 of 11 | Every topology variant needs a no-edge fallback |
| Ticket label length | 30 to 42 chars | Two label columns at 100 wide, four at 200 |
| `map.md` length | 85 to 192 lines | Prose never fits. Scrolling is mandatory |
| `Decisions so far` | 21 to 116 lines | The bulk of the Map is the decision log |
| `Destination` | 4 to 12 lines | Fits as a header |
| Ticket body | 15 to 467 lines, median 66 | A detail pane scrolls or truncates |
| Ticket links inside `map.md` | 3 to 22 | Enough to annotate the prose, too few to navigate by |

Two facts drive the whole set. The Ticket graph is a bushy root followed by a spine, so the
techniques that pin one axis beat the ones that solve two. And the prose outweighs the graph by an
order of magnitude, so a variant that renders only the graph is discarding most of the artifact.

## The five theses

| Key | Name | Thesis in one line |
|---|---|---|
| A | Canvas | A Map is a dependency graph, so place Tickets in two dimensions and draw the edges |
| B | Rail | A Map is one ordered column of Tickets with the topology in a narrow gutter |
| C | Board | A Map is a queue of decisions, so position by readiness and drop the edges |
| D | Document | A Map is a document, and the Tickets are annotations of a text you read top to bottom |
| E | Outline | A Map is an index, so half the screen lists Tickets and half shows the one you picked |

## A. Canvas

**Layout.** Sugiyama layered drawing on a pannable surface. `rataflow` owns rank assignment,
crossing reduction and coordinates; Tickets become nodes, `blocked_by` becomes edges. Zoom, fit and
minimap come with the crate.

**Prior art.** `rataflow` 0.1.0 (survey area 2), which is `rust-sugiyama` plus a renderer that
already solves the two hard parts: junctions merged through `Cell::merge_symbol` and braille strokes
for diagonals. Behind it, Graphviz `dot` and dagre.

**Why it suits Tickets under a Map.** `blocked_by` is the only structure the Map states explicitly,
and this is the only variant that shows all of it at once: which decisions the effort forked into at
the start, where they rejoin, what the critical chain to the destination is.

**Tradeoffs.**

- Auto-organisation: complete, and the only variant where it is. Nothing is hand-placed.
- Theming: `rataflow` palettes, plus per-node colour from status and a type glyph in the label.
- Selection and popup: node selection is built in. Detail goes in an overlay drawn over the canvas,
  since the surface is already full.
- Title: a header line above the canvas. The canvas cannot spare interior space for it.
- Status legibility: good per node (border colour, glyph), poor for the Map as a whole. Counting
  open Tickets means counting nodes.

**Disagreement.** It is the only variant that spends two dimensions on layout, so it is the only one
that can put two unrelated Tickets side by side, and the only one that has to pan. B fixes the row
order and refuses to move a Ticket off its line. C positions by state, not by topology. D and E have
no topology at all.

**Risk it exists to expose.** With layer widths like `[8,1,1,2,1,1]` this drawing may be a fat root
and a thin stick, with labels truncated to nothing. Better to learn that here.

## B. Rail

**Layout.** One row per Ticket in topological order. A gutter 4 to 10 columns wide carries the lanes,
forks and joins; the rest of the row is Ticket text at full width. `renderdag` returns a structured
row plan per row, so each glyph is styled individually rather than handed over as a finished string.

**Prior art.** `git log --graph`, `tig`, `lazygit`, and `jj log`, which drives Meta's
`sapling-renderdag` through its `GraphLog` trait. Survey area 4: every one of these pins the row
axis and solves only lane assignment.

**Why it suits Tickets under a Map.** The fixtures are a bushy root and then a spine, which is the
exact shape the git graph was built for. Ticket ids are numbered and already close to topological
order, so the rail rarely has to reorder anything, and every Ticket keeps a full-width line for its
label, type, status and gist.

**Tradeoffs.**

- Auto-organisation: lane assignment only. Deterministic, no pan, no zoom, no layout to tune.
- Theming: per glyph, since the row plan is data. Lane colour can follow the branch, text style the
  status.
- Selection and popup: the selected row is a list highlight. Detail is a popup over the rail.
- Title: a header line, with the destination gist under it, since a row costs one line.
- Status legibility: the best of the five. Status is a glyph in the rail (filled for open, hollow for
  closed) and every Ticket is on screen at once for a Map of 16.

**Disagreement.** It takes A's topology and throws away one degree of freedom on purpose. Against C,
it keeps the edges and orders by dependency rather than by state. Against E, it refuses a detail
pane, spending the full width on the list. Against D, the prose is a header and the Tickets are the
body.

## C. Board

**Layout.** Columns by readiness: Ready, Blocked, Closed. Each column is a list of multi-line cards
built with `tui-widget-list`. No edges are drawn at all. `blocked_by` becomes a badge on the card,
and selecting a card highlights its blockers and its dependents in the other columns.

**Prior art.** k9s and lazydocker panel layouts, and the topological-columns family in survey area 3,
which never leaves the grid because it never tries to.

**Why it suits Tickets under a Map.** A Ticket is a decision nobody has made yet, so the question
this answers is which decision can be made now. `Map::is_ready` already computes it. This is the only
variant whose primary axis is a question the Map does not state and the reader has to work out.

**Tradeoffs.**

- Auto-organisation: none, and that is the point. Bucketing is a sort, so it is correct at every
  terminal size and never overflows.
- Theming: per card, with the column carrying the state colour. The widest theming surface of the
  five.
- Selection and popup: card selection with cross-column highlight, which substitutes for the edges
  it drops. Detail can expand the card in place rather than pop over.
- Title: a full header band, since columns start below it anyway.
- Status legibility: the best at the Map level. Column heights are the progress bar. Per Ticket it is
  the worst, because position no longer says anything about where a Ticket sits in the route.

**Disagreement.** The only variant that discards the graph while still being about Tickets. A and B
sort by dependency; C sorts by state and shows dependency only on demand. D and E do not sort at all.
It is also the only one that works identically at 100x30 and 200x50, which is the claim to test.

## D. Document

**Layout.** The Map as continuous prose, scrolled. `tui-markdown` turns each section into styled
`Text`, `tui-scrollview` moves through it. Ticket links inside the prose are decorated with the
status and type of the Ticket they point at, and the shared `split_ticket_links` already cuts the
line for it. The selected link expands inline to show the Ticket's question and resolution. Tickets
the prose never links get a generated section at the end, since one Map links only 3 of its 10.

**Prior art.** `glow` and other terminal markdown readers, `tui-markdown` (survey area 2). The
inline-expansion move comes from literate document readers rather than from any graph tool.

**Why it suits Tickets under a Map.** The Map is 85 to 192 lines of prose against a 16 node graph,
and `Decisions so far` is the largest section in almost every Map. Only this variant shows Notes and
Out of scope at all, which is where the reasons live. The prose already points at Tickets with real
links, so the document carries a hand-written ordering that no layout algorithm can recover.

**Tradeoffs.**

- Auto-organisation: none. The author's section order is the layout, and that is the thesis.
- Theming: markdown styling plus status colour on link text. Constrained by what `tui-markdown` emits.
- Selection and popup: selection is the link under the cursor, and expansion happens inline. A popup
  would cover the text the reader is here for.
- Title: the H1 in the flow of the document, where the author put it.
- Status legibility: per Ticket only where the prose mentions it, and a Map-level summary is
  impossible without adding chrome the thesis rejects.

**Disagreement.** The only variant where the Tickets are not the subject. The other four treat
`map.md` as metadata and the `tickets/` directory as content; this one inverts that. It is also the
only one with no second panel, no columns and no gutter.

## E. Outline

**Layout.** Two panes. Left is every Ticket, indented by depth in the dependency graph, showing
status and type. Right is the selected Ticket rendered in full and scrolled independently. A Ticket
with two blockers is indented under the first and carries a marker naming the others, because a tree
cannot hold a join.

**Prior art.** File explorers, mail readers, lazygit's panel model, `tui-tree-widget` (survey area 2,
which states the join problem plainly: it renders a tree, not a DAG).

**Why it suits Tickets under a Map.** Ticket bodies run to a median of 66 lines and a maximum of 467,
so they are the only part of a Map that genuinely needs a pane of its own. Indentation conveys the
spine of the route at a glance without any drawing, and the reader never loses their place in the
list while reading a body.

**Tradeoffs.**

- Auto-organisation: a depth sort and a spanning tree. Cheap, and it lies about joins by
  construction.
- Theming: two panes to theme, with the risk that the body pane outshouts the list.
- Selection and popup: selection drives the right pane, so no popup is needed anywhere. The only
  variant where detail has a permanent home.
- Title: a border title on the left pane, with the Ticket id on the right pane's border.
- Status legibility: good per Ticket, with status in the gutter of every row. The Map-level view
  costs a summary line because half the width is gone.

**Disagreement.** The only variant that shows a Ticket body in full without hiding the Map. Against
B, which is also one column of rows, it trades the whole right half away and replaces drawn edges
with indentation, so it cannot show joins at all. Against A it flattens two dimensions into one.
Against C it sorts by dependency. Against D the pane content is a Ticket, not the Map.

## The frozen interface

Built and green in `crates/map_tui/src/prototype/`. Phase 4 agents read it and do not change it.

```rust
pub trait Variant {
    fn name(&self) -> &'static str;
    fn draw(&mut self, frame: &mut Frame, area: Rect, map: &Map, selection: &Selection);
    fn handle_key(&mut self, key: KeyEvent, map: &Map, selection: &mut Selection) {}
}
```

- `Map`: `id`, `title`, `sections`, `tickets`, `source`, plus `ticket`, `index_of`, `section`,
  `destination`, `blocks`, `is_ready`, `open_tickets`, `edges`.
- `Ticket`: `id`, `label`, `kind`, `status`, `blocked_by`, `assignee`, `sections`, `source`, plus
  `is_open`, `question`, `resolution`, `gist`.
- `Selection`: `ticket_id`, `is`, `select`, `index_in`, `step`. The switcher reconciles it on every
  variant and Map change, falling back to the Map's first Ticket.
- `split_ticket_links` cuts a markdown line at its `(tickets/<id>.md)` links.

The switcher claims `Tab`, `BackTab`, `[`, `]`, `q` and `Ctrl-C`. Every other key reaches the
variant. The bottom bar is drawn by the switcher and takes one line; a variant gets the rest.

Run it with `just proto::map`, or `just proto::list` to print what the loader parsed. The fixture
directory comes from the first argument, else `WAYFINDER_MAPS_DIR`, else the wayfinder charts in the
monorepo.

## Dependencies

Added once, with `cargo add`, before the parallel phase, so five agents never race on the manifest:
`rataflow` for A, `renderdag` for B, `tui-widget-list` for C, `tui-markdown` and `tui-scrollview` for
D, `tui-tree-widget` for E, `tui-popup` for whoever needs an overlay. All are on the ratatui 0.30
line and all were verified on crates.io at the versions the survey lists. Whether any of them becomes
a real dependency is a later decision.
