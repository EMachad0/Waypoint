# Survey: rendering a Map in a ratatui terminal UI

Research done 2026-09-12. Every version, date and download count below was read from the
crates.io API or the project's own repository on that day, not from memory. Download figures are
given as `all-time / last 90 days`, which is what crates.io reports.

The thing to render is one Map: a title, five or six prose sections of 85 to 190 lines total with
inline links, and a DAG of 6 to 16 Tickets carrying a type, a status and a `blocked_by` list. The
DAG is shallow and mostly linear with a few forks and joins. Target terminal sizes are 100x30 up
to 200x50.

## 1. ratatui 0.30 on the pinned 0.30.2 line

### Version facts

| Fact | Value |
|---|---|
| Pinned version | 0.30.2, published 2026-06-19 |
| 0.30 line | 0.30.0 on 2025-12-26, 0.30.1 on 2026-06-05, 0.30.2 on 2026-06-19 |
| Previous line | 0.29.0 on 2024-10-21 |
| Downloads | 50,401,206 / 17,933,830 |
| MSRV | 1.88.0 for 0.30.2 (0.30.0 was 1.86.0). Repo toolchain is 1.96.0, so no constraint |
| Edition | 2024 |
| Repo | [ratatui/ratatui](https://github.com/ratatui/ratatui), 22,554 stars, last push 2026-09-11 |
| License | MIT |

Sources: [0.30.0 highlights](https://ratatui.rs/highlights/v030/),
[BREAKING-CHANGES.md](https://github.com/ratatui/ratatui/blob/main/BREAKING-CHANGES.md),
[release notes](https://github.com/ratatui/ratatui/releases/tag/ratatui-v0.30.0).

### What 0.29 to 0.30 changed that a widget author must know

**The crate was split into a workspace.** `ratatui` 0.30.2 is now a facade over `ratatui-core`
0.1.2, `ratatui-widgets` 0.3.2, and one backend crate per backend (`ratatui-crossterm` 0.1.2,
`ratatui-termion`, `ratatui-termwiz`, `ratatui-termina` 0.1.0), plus `ratatui-macros` 0.7.2.
Applications keep depending on `ratatui`. Widget libraries are told to depend on `ratatui-core`
instead, for a more stable surface. This matters when reading third-party manifests: a crate that
declares `ratatui-core ^0.1` and `ratatui-widgets ^0.3` is 0.30-compatible even though the string
"0.30" never appears in its dependency list. That is how most of the maintained widget crates in
area 2 are declared.

**crossterm comes through ratatui.** `ratatui::crossterm` is a re-export of
`ratatui_crossterm::crossterm`, gated on the `crossterm` feature, which is on by default. The
resolved crossterm is 0.29.0 (published 2025-04-05). `crossterm_0_28` and `crossterm_0_29` feature
flags exist for pinning a specific major; when both are on, the newest wins. Depending on crossterm
directly is how you end up with two crossterm majors and event types that do not match the backend.

**`ratatui::run` replaces hand-rolled setup and teardown.** `ratatui::run(|terminal| { .. })` calls
`init()` before the closure and `restore()` after, returning the closure's result.
`init()` gives a `DefaultTerminal` on `CrosstermBackend<Stdout>` with raw mode, the alternate
screen, and a panic hook that restores the terminal. `init_with_options` takes a `Viewport` for
inline or fixed-region rendering. `Terminal::new` is now only for custom backends.
Docs: [`ratatui::init`](https://docs.rs/ratatui/latest/ratatui/init/index.html),
[`run`](https://docs.rs/ratatui/latest/ratatui/init/fn.run.html).

**Widget API churn, the parts that bite:**

- `WidgetRef` lost its blanket `Widget` impl, and the relation is reversed. You now write
  `impl Widget for &MyWidget` and get `WidgetRef` for free. Any widget that wants to be rendered
  without being consumed implements `Widget` for a reference.
- `Frame::render_widget_ref` and `render_stateful_widget_ref` moved behind the `FrameExt` trait and
  the `unstable-widget-ref` feature. Rendering `&widget` through `render_widget` works on stable
  because of the impl above; the `_ref` methods do not.
- `layout::Alignment` is renamed `HorizontalAlignment`. `VerticalAlignment` is new.
- `widgets::block::Title` no longer exists. `Block::title` takes `Into<Line>`,
  `block::Position` is now `widgets::TitlePosition`, `widgets::block` is not exported.
- `List::highlight_symbol` takes `Into<Line>` instead of `&str`, so it can be styled, and it can no
  longer be built in const context.
- `Backend` gained an associated `Error` type and a required `clear_region` method. Generic code
  over `B: Backend` returns `Result<_, B::Error>` rather than `io::Result<_>`.
- `From` conversions between ratatui and backend types were replaced by `FromCrossterm` /
  `IntoCrossterm` and siblings.
- `Flex::SpaceAround` changed meaning. The old behaviour is now `Flex::SpaceEvenly`.
- `Style` no longer implements `Styled`; the stylize shorthands are inherent on `Style` so they
  work in const context.
- Layout cache is opt-in in `ratatui-core` and on by default in `ratatui` through the
  `layout-cache` feature. Turning off default features without re-enabling it costs performance.

**New in 0.30 that is directly useful here:**

- `MergeStrategy` (`Replace`, `Exact`, `Fuzzy`) with `Cell::merge_symbol(symbol, strategy)` and
  `Block::merge_borders(strategy)`. This resolves overlapping box-drawing glyphs into the correct
  junction: `merge("┘", "┏")` gives `╆`. This is the single most important primitive for area 3 and
  4, because it removes the hand-written 200-entry junction table that every prior-art git graph
  renderer had to build. It lives on `Cell` in the `Buffer`, not on `Canvas`.
- Four new `Marker` variants beyond `Braille`: `Quadrant` (2x2), `Sextant` (2x3), `Octant` (2x4).
  `Octant` is the same resolution as `Braille` but with densely packed pixels and no visible gap
  between cells. It uses a much newer Unicode block, so font support is worse.
- `Rect::layout`, `try_layout`, `layout_vec`, `centered`, `centered_horizontally`,
  `centered_vertically`, `outer`. Splitting an area is now a method on the area.
- Dashed border sets (`LightDoubleDashed` through `HeavyQuadrupleDashed`) and `Block::shadow`.

### The built-in widgets

`ratatui-widgets` 0.3.2 ships: `barchart`, `block`, `borders`, `canvas`, `chart`, `clear`, `fill`,
`gauge`, `list`, `logo`, `mascot`, `paragraph`, `scrollbar`, `sparkline`, `table`, `tabs`, and
`calendar` behind a feature. There is no tree, no graph, no markdown and no popup in the box.

### What Canvas can and cannot do

Read from the 0.30.2 source of `ratatui-widgets/src/canvas.rs`, not from the prose docs.

**Coordinate space.** `x_bounds` and `y_bounds` are `[f64; 2]`. Y increases upward, which is the
opposite of buffer rows. The canvas builds a `Context` sized to the inner `Rect` in cells, and
`Painter::get_point(x, y)` maps world coordinates onto dot indices.

**Resolution per marker.**

| Marker | Grid | Dots per cell | Colour |
|---|---|---|---|
| `Dot`, `Block`, `Bar` | `CharGrid` | 1x1 | one fg per cell |
| `HalfBlock` | `HalfBlockGrid` | 1x2 | one fg and one bg per cell |
| `Quadrant` | `PatternGrid<2,2>` | 2x2 | one fg per cell |
| `Sextant` | `PatternGrid<2,3>` | 2x3 | one fg per cell |
| `Braille`, `Octant` | `PatternGrid<2,4>` | 2x4 | one fg per cell |

A 200x50 terminal at `Braille` is a 400x200 dot field. That sounds generous until you notice the
colour constraint: a pattern grid stores one foreground colour per character cell, so two edges of
different colours crossing inside one cell cannot both keep their colour. `HalfBlock` is the only
grid that gives you two colours per cell, at 1x2.

**Clipping.** `Line` runs Cohen-Sutherland against the bounds and then Bresenham, so lines that
leave the canvas are clipped, not dropped. `Painter::get_point` returns `None` outside bounds, so a
custom `Shape` that does not clip its own geometry will simply lose the out-of-bounds parts.

**Layers.** `Context::layer()` snapshots the grid and starts a new one. A blank pattern cell in an
upper layer lets the lower layer show through, so you can draw a `Block` marker background pass and
a `Braille` stroke pass and get the symbol from one and the background from the other.

**Text.** `Context::print(x, y, line)` queues a `Label`. Labels are drawn last, on top of every
layer, are filtered to the bounds, and land at **cell** resolution, not dot resolution, computed
against `(canvas_area.width - 1, canvas_area.height - 1)`. A label is truncated at the right edge of
the canvas. So text on a Canvas is exactly as coarse as text anywhere else; the sub-cell resolution
only ever applies to strokes.

**What Canvas cannot do.** It writes each cell with `set_char`, which overwrites. There is no
`merge_symbol` path inside `Canvas`, so two crossing box-drawing strokes clobber each other instead
of becoming `┼`. Crossing braille strokes do merge, because the pattern bits OR together within a
layer, but crossing braille strokes drawn in *different* layers do not merge either. If you want
merged box-drawing junctions you draw into the `Buffer` yourself, not through `Canvas`.

Docs: [`Canvas`](https://docs.rs/ratatui/latest/ratatui/widgets/canvas/struct.Canvas.html),
[`Marker`](https://docs.rs/ratatui/latest/ratatui/symbols/enum.Marker.html),
[canvas.rs source](https://docs.rs/ratatui-widgets/latest/src/ratatui_widgets/canvas.rs.html).

### What the built-in widgets cost when abused

- **`Paragraph` re-wraps its entire text every frame.** Rendering line 400 of a 500-line section
  costs the wrap of all 500 lines, every frame. With 190 lines per Map this is fine; it stops being
  fine if a variant builds one giant Paragraph of Map plus every Ticket body.
- **You cannot measure a wrapped `Paragraph` on stable.** `line_count(width)` and `line_width()`
  are gated behind the `unstable-rendered-line-info` feature. A variant that wants a scrollbar
  proportional to wrapped content has to either enable that feature, wrap the text itself, or fake
  the proportion from the unwrapped line count.
- **`Paragraph::scroll` is a plain `Position` offset into the laid-out lines.** There is no notion
  of "scroll to this Ticket"; you compute the offset.
- **`Table` rows are fixed height, `List` items can be multi-line `Text`.** A board of cards built
  out of `Table` will fight you; built out of `List` it will not, but then column alignment is
  yours to maintain.
- **`Canvas` allocates.** One `PatternCell` per cell per frame, plus a full `Layer` snapshot per
  `layer()` call. Three layers over a 200x50 area is 30,000 cell records per frame. Measurable, not
  fatal.
- **Braille and friends depend on the font.** Terminals or fonts without the Braille Patterns block
  render replacement characters. `Octant` and `Sextant` are newer blocks with worse coverage than
  Braille. lazygit hit exactly this problem and moved its commit graph off `⏣` and `◯` because they
  are missing from most monospace fonts
  ([commit](https://github.com/jesseduffield/lazygit/commit/35d3659ccee590b00f274e5c4d883e1176a4485b)).
  Any variant that leans on exotic glyphs needs an ASCII fallback, or it needs to accept that it
  only looks right on the developer's machine.

## 2. Third-party ratatui widget crates

### Verdict table

Downloads are all-time / last 90 days. "ratatui dep" is what the crate's own manifest declares;
`core 0.1 + widgets 0.3` means it targets the 0.30 line through the split crates.

| Crate | Latest | Released | Downloads | ratatui dep | Repo activity | License | Verdict |
|---|---|---|---|---|---|---|---|
| [rataflow](https://crates.io/crates/rataflow) | 0.1.0 | 2026-08-18 | 9,698 / 9,698 | `^0.30` | 162 stars, pushed 2026-09-11 | MIT | **USE** |
| [tui-nodes](https://crates.io/crates/tui-nodes) | 0.10.0 | 2026-01-07 | 21,190 / 957 | `^0.30` | sr.ht, last commit 8 months ago | MIT | **USE** |
| [tui-popup](https://crates.io/crates/tui-popup) | 0.7.6 | 2026-06-14 | 650,450 / 506,366 | core 0.1 + widgets 0.3 | ratatui org, pushed 2026-09-09 | MIT OR Apache-2.0 | **USE** |
| [tui-scrollview](https://crates.io/crates/tui-scrollview) | 0.6.7 | 2026-06-14 | 452,736 / 75,176 | core 0.1 + widgets 0.3 | ratatui org, pushed 2026-09-09 | MIT OR Apache-2.0 | **USE** |
| [tui-widget-list](https://crates.io/crates/tui-widget-list) | 0.15.3 | 2026-07-18 | 317,788 / 139,607 | core 0.1 + widgets 0.3 | 51 stars, pushed 2026-07-18 | MIT | **USE** |
| [tui-tree-widget](https://crates.io/crates/tui-tree-widget) | 0.24.1 | 2026-08-09 | 1,487,932 / 744,408 | core 0.1 + widgets 0.3 | 128 stars, pushed 2026-09-01 | MIT | **USE** |
| [tui-markdown](https://crates.io/crates/tui-markdown) | 0.3.9 | 2026-07-23 | 468,072 / 145,544 | core 0.1 | 117 stars, pushed 2026-09-07 | MIT OR Apache-2.0 | **USE** |
| [tui-big-text](https://crates.io/crates/tui-big-text) | 0.8.9 | 2026-08-24 | 495,782 / 85,519 | core 0.1 + widgets 0.3 | ratatui org, pushed 2026-09-09 | MIT OR Apache-2.0 | **USE**, narrow |
| [tui-overlay](https://crates.io/crates/tui-overlay) | 0.1.2 | 2026-04-07 | 2,617 / 1,770 | core 0.1 + widgets 0.3 | 50 stars, pushed 2026-04-07 | MIT OR Apache-2.0 | **RISKY** |
| [rat-popup](https://crates.io/crates/rat-popup) | 3.0.2 | 2026-03-08 | 25,042 / 2,857 | core 0.1 + crossterm 0.1 + widgets 0.3 | 64 stars, pushed 2026-07-03 | MIT/Apache-2.0 | **RISKY** |
| [ratatui-flow](https://crates.io/crates/ratatui-flow) | 0.1.1 | 2026-06-18 | 55 / 55 | `^0.30` | 0 stars, pushed 2026-06-18 | MIT | **RISKY** |
| [tui-treelistview](https://crates.io/crates/tui-treelistview) | 0.2.2 | 2026-07-15 | 361 / 319 | `^0.30.2` | 2 stars, pushed 2026-09-07 | MIT OR Apache-2.0 | **RISKY** |
| [diffler-graph](https://crates.io/crates/diffler-graph) | 0.2.1 | 2026-06-22 | 28 / 28 | `^0.30` | 8 stars, pushed 2026-09-10 | MIT OR Apache-2.0 | **RISKY** |
| [tachyonfx](https://crates.io/crates/tachyonfx) | 0.25.2 | 2026-09-06 | 383,233 / 223,973 | core 0.1.2 | ratatui org, pushed 2026-09-06 | MIT | **RISKY**, out of scope |
| [ratatui-image](https://crates.io/crates/ratatui-image) | 11.0.8 | 2026-09-04 | 828,177 / 360,400 | `^0.30.1` | ratatui org, pushed 2026-09-04 | MIT | **RISKY**, see note |
| [ratatui-graph](https://crates.io/crates/ratatui-graph) | 0.1.0 | 2024-08-11 | 1,242 / 13 | `^0.28` | repo 404s | MIT | **REJECT** |
| [ascii-petgraph](https://crates.io/crates/ascii-petgraph) | 0.2.0 | 2026-01-28 | 1,885 / 1,667 | `^0.29` | 6 stars, pushed 2026-01-28 | MIT | **REJECT** |
| [tuiflow](https://crates.io/crates/tuiflow) | 0.2.2 | 2026-03-28 | 415 / 200 | `^0.29` | 0 stars, pushed 2026-03-28 | MIT | **REJECT** |
| [ratslate](https://crates.io/crates/ratslate) | 0.1.2 | 2026-09-07 | 40 / 40 | `^0.30` | 6 stars, pushed 2026-09-10 | MIT OR Apache-2.0 | **REJECT** |
| [ratatui-markdown](https://crates.io/crates/ratatui-markdown) | 0.3.6 | 2026-05-21 | 4,714 / 3,931 | `^0.29` | celestia-island | MIT OR Apache-2.0 | **REJECT** |

### What each one buys us

**rataflow.** A full node-graph widget: nodes and edges generic over your content types, pan, zoom,
fit-view, mouse drag, multi-select, keyboard navigation, off-screen culling, minimap, `Controls` and
`Background` companion widgets, runtime theming with Dark/Light/custom palette, and serde snapshots.
Layout is Sugiyama via an optional `rust-sugiyama` dependency behind the default `sugiyama` feature,
with `set_node_positions` for any external algorithm. `Flow::from_edges(&[(&str, &str)], Sugiyama::vertical())`
gets a laid-out graph on screen in one call. Edge rendering already solves the two hard parts:
crossing edges are merged into junction glyphs through `Cell::merge_symbol(_, MergeStrategy::Fuzzy)`,
and braille strokes are OR-ed into the buffer for smooth diagonals. MSRV 1.88, edition 2024,
`ratatui = { version = "0.30", default-features = false }`, which unifies cleanly with the workspace
pin. The risk is age: first real release 2026-08-18, one author, 0.1.0. For a prototype that risk is
acceptable and the payoff is the largest of anything in this survey.

**tui-nodes.** The small, old answer to the same problem, and the one on the official widget
showcase. `NodeGraph::new(nodes, connections, width, height)`, then `calculate()`, then
`split(area)` returns a `Vec<Rect>`, one per node, that you render your own widgets into. The graph
itself is a `StatefulWidget` that draws the boxes and the connection lines. It bumped to ratatui
0.30 eight months ago and has not moved since. It buys a working node graph with a tiny API and no
mouse story. Its layout is its own, not Sugiyama.

**tui-popup.** `Popup` widget with body, title, borders, optional mouse-drag movement via
`PopupState`. Maintained inside the ratatui org's `tui-widgets` monorepo alongside tui-scrollview,
tui-big-text and tui-bar-graph. The default choice for a Ticket detail overlay.

**tui-scrollview.** `ScrollView::new(content_size)` plus `ScrollViewState`. You render widgets into
a virtual area larger than the screen and the view blits a window of it, with optional scrollbars.
The cost is explicit in the API: it allocates a `Buffer` of the full content size. A 200-wide by
2000-tall virtual canvas is 400,000 `Cell`s. The docs tell you to store the `ScrollView` and render
it by reference when the content is expensive, which is the mitigation.

**tui-widget-list.** A list whose items are arbitrary widgets with per-item heights, with
`ListBuildContext` telling each item whether it is selected and how much room it has. This is the
crate for a Ticket list where each row is a two-line card rather than a string.

**tui-tree-widget.** The most-downloaded third-party ratatui widget in this survey by a wide margin.
Nested `TreeItem`s with open/close state, selection, and flattening. It renders a tree, not a DAG, so
a Ticket with two blockers has to be duplicated or elided.

**tui-markdown.** `tui_markdown::from_str(&str)` returning a `Text`, on `pulldown-cmark` 0.13, with optional
`syntect` highlighting and `ansi-to-tui`. It converts markdown to styled `Text` and stops there:
no widget, no scrolling, no link handling. That is the right size for turning Map prose sections
into styled paragraphs.

**tui-big-text.** `font8x8` glyphs as a widget. Only relevant if a variant wants the Map title as a
banner. Everything else about it is overkill.

**tui-overlay, rat-popup.** Both work on 0.30. `tui-overlay` gives drawers, modals, popovers and
toasts from one primitive; it is four months old with one release burst and no activity since.
`rat-popup` is better established but drags in `rat-cursor`, `rat-event`, `rat-focus` and
`rat-reloc`, which is the rat-salsa event and focus model, not just a popup. Neither beats
`tui-popup` for this job.

**ratatui-flow.** Technically the closest match to the problem after rataflow: automatic DAG layout,
grid-search connection routing with per-connection line type and colour, content-fitted node sizing
through `unicode-width`, graceful degradation to `Diagnostic`s instead of panics on cycles and
oversmall canvases, and a one-shot offscreen canvas render with a `NodeGraphView` blit viewport so
panning does not recompute layout. The problem is entirely social: 55 downloads, zero stars, two
releases one day apart in June 2026 and silence since, and a Chinese-first README with the English
version secondary. Usable in a throwaway prototype, not something to build on.

**tui-treelistview.** Interactive tree list on `ratatui ^0.30.2`, actively pushed. But 0.2.0 and
0.2.1 were both yanked the same day 0.2.2 shipped, and total downloads are 361. That release
pattern is the warning.

**diffler-graph.** An orthogonal node-graph TUI component extracted from the `diffler` app. On
ratatui 0.30 and the parent repo is active, but 28 downloads total and no independent users.

**ratatui-image.** Not a graph widget. It is here because of what `serie` does in area 4: render the
graph as a raster image and push it through the kitty or iTerm2 graphics protocol. That buys
unlimited resolution and costs you every terminal that does not implement the protocol. Listed as
RISKY because the technique is real and available, not because the crate is shaky.

### Dead ends, stated plainly

- **`ratatui-graph` is abandoned and its repo is gone.** One release, 2024-08-11, on ratatui 0.28.
  `https://github.com/kdheepak/ratatui-graph` returns 404. Thirteen downloads in the last 90 days.
  It cannot be used with the 0.30 pin and there is nothing to fork.
- **`ascii-petgraph` is on ratatui 0.29.** It is the only crate in this survey that combines
  petgraph with a force-directed layout and an ASCII renderer, which sounds ideal. It would pull a
  second, incompatible ratatui into the tree, and its widget would not satisfy 0.30's `Widget`
  trait. Last push 2026-01-28, one day after first release. Dead for us.
- **`tuiflow` and `ratatui-markdown` are on ratatui 0.29**, same problem. `ratatui-markdown` also
  carries forty-odd optional `tree-sitter-*` grammars.
- **`ratslate` publishes no library.** crates.io reports `has_lib=false` and a single `ratslate`
  binary. It is a mouse-driven infinite-canvas diagram editor that pulls `yrs` (a CRDT), `schemars`
  and `ratatui-dnd`. Even if it had a library target, it is an authoring tool, not a renderer for a
  DAG you already have.
- **There is no `dagre` crate on crates.io that is both current and maintained for this.** See
  area 3.
- **The ratatui widget showcase has no graph, board or DAG widget.** The
  [official showcase](https://ratatui.rs/showcase/third-party-widgets/) lists ratatui-image,
  ratatui-textarea, throbber, tui-big-text, tui-checkbox, tui-logger, tui-menu, tui-nodes,
  tui-piechart, tui-scrollview, tui-term, tui-tree-widget and tui-widget-list. `tui-nodes` is the
  only graph entry. Everything else in this section came from
  [Awesome Ratatui](https://github.com/ratatui/awesome-ratatui) or from crates.io search.

## 3. DAG layout algorithms and their implementations

### The algorithm families, and which survive a character grid

**Sugiyama / layered drawing.** Four phases: break cycles, assign each node a layer (rank), reduce
crossings by ordering nodes within each layer, assign coordinates. It is the algorithm behind
Graphviz `dot`, dagre, mermaid and ELK. It is also the only family that naturally produces what a
terminal wants: discrete layers, which become rows or columns of cells, and an explicit within-layer
ordering, which becomes a lane index. The coordinate phase is the only part that produces continuous
values, and it is the only part you have to quantise. **Survives quantisation: yes, best in class.**

**Tree layouts (Reingold-Tilford).** Linear-time, tidy, well-defined. But it needs a tree. Ticket
DAGs have joins: two Tickets blocked by the same Ticket, or one Ticket blocked by two. A join forces
either a duplicated subtree or an edge that the tree layout does not know how to draw.
**Survives quantisation: yes, but only if you accept a spanning tree plus separately drawn cross
edges, or duplicate nodes.**

**Force-directed.** Continuous positions from a physics simulation. On a character grid every
position is rounded to a cell, which destroys the fine distinctions the simulation produced;
adjacent nodes collide, edges become diagonals across a grid with 2:1 cell aspect ratio, and the
result is non-deterministic between runs unless the seed is fixed. It also has no notion of flow
direction, which is the one thing a "route from idea to destination" wants to show.
**Survives quantisation: no. This is the trap of the survey.**

**Topological columns and swimlanes.** Not a graph layout at all. Sort topologically, bucket into
columns by depth or by status, render each bucket as a list. Edges are either dropped, or reduced to
badges ("blocked by 3"), or drawn only for the selected node. Deterministic, trivially scrollable,
never overflows, and correct at any terminal size. **Survives quantisation: trivially, because it
never leaves the grid.**

**Sankey-ish flows.** Width-proportional ribbons between stages. Needs a quantity to be proportional
to, and a Ticket has none. Would degenerate into a layered diagram with thicker lines.
**Survives quantisation: technically, but there is nothing here to encode in the width.**

**Lane assignment (the git-log family).** A special case of layered drawing where the layer axis is
fixed by the input order (one row per node) and only the cross-axis lane needs assigning. Scales to
one or two columns of glyphs per lane and is the single most battle-tested technique for drawing a
DAG in a terminal. Covered in area 4. **Survives quantisation: it was born quantised.**

### Crate verdicts

| Crate | Latest | Released | Downloads | Deps | Repo activity | License | Verdict |
|---|---|---|---|---|---|---|---|
| [petgraph](https://crates.io/crates/petgraph) | 0.8.3 | 2025-09-30 | 503,828,221 / 104,897,373 | none relevant | 4,017 stars, pushed 2026-09-06 | MIT OR Apache-2.0 | **USE**, data structure only |
| [rust-sugiyama](https://crates.io/crates/rust-sugiyama) | 0.4.0 | 2025-09-21 | 29,621 / 13,993 | petgraph 0.8, log | 28 stars, pushed 2025-09-21 | MIT | **USE** |
| [ascii-dag](https://crates.io/crates/ascii-dag) | 0.11.0 | 2026-09-06 | 79,521 / 71,165 | **zero** | 25 stars, pushed 2026-09-06 | MIT OR Apache-2.0 | **USE**, with a caveat |
| [renderdag](https://crates.io/crates/renderdag) | 0.4.0 | 2026-08-23 | 339 / 201 | none (serde optional) | 1 star, pushed 2026-08-23 | MIT | **USE** |
| [sapling-renderdag](https://crates.io/crates/sapling-renderdag) | 0.1.0 | 2024-11-12 | 682,023 / 209,455 | bitflags 2.6 | facebook/sapling, pushed 2026-09-11 | MIT (see note) | **USE** |
| [layout-rs](https://crates.io/crates/layout-rs) | 0.1.3 | 2025-04-24 | 694,350 / 204,252 | log (optional) | 740 stars, pushed 2025-05-22 | MIT | **RISKY** |
| [dugong](https://crates.io/crates/dugong) | 0.8.0-alpha.6 | 2026-09-02 | 71,104 / 65,707 | dugong-graphlib, rustc-hash, serde | part of Latias94/merman, 551 stars, pushed 2026-09-10 | MIT OR Apache-2.0 | **RISKY** |
| [reingold-tilford](https://crates.io/crates/reingold-tilford) | 1.0.0 | 2019-02-03 | 2,385 / 95 | none | last release 2019 | MIT | **RISKY** |
| [elkrs](https://crates.io/crates/elkrs) | 0.1.1 | 2026-06-17 | 295 / 295 | n/a | single release day | Apache-2.0 | **REJECT** |
| [dagre](https://crates.io/crates/dagre) | 0.1.1 | 2026-05-06 | 5,016 / 4,759 | log | repo **archived** | Apache-2.0 | **REJECT** |
| [gen-sugiyama](https://crates.io/crates/gen-sugiyama) | 0.2.1 | 2026-07-09 | 726 / 172 | petgraph **0.6** | vendored inside genhub-bio/gen | MIT | **REJECT** |
| [rust-sugiyama-fork](https://crates.io/crates/rust-sugiyama-fork) | 0.5.1 | 2026-08-22 | 51 / 51 | n/a | 0 stars, one day of commits | MIT | **REJECT** |
| [abstracttui-graph](https://crates.io/crates/abstracttui-graph) | 0.5.0 | 2026-08-25 | 151 / 151 | `abstracttui` 0.6 | 2 stars | MIT | **REJECT** |
| [forceatlas2](https://crates.io/crates/forceatlas2) | 0.8.0 | 2025-10-12 | 16,924 / 761 | n/a | framagit | **AGPL-3.0-only** | **REJECT** |
| [fdg-sim](https://crates.io/crates/fdg-sim) | 0.9.1 | 2022-12-17 | 38,031 / 5,821 | n/a | last release 2022 | MIT | **REJECT** |

### What each one buys us

**petgraph.** It is a graph *data structure* library and a graph *algorithms* library. It does not
do layout. Its modules are `algo`, `csr`, `data`, `dot`, `graph`, `graphmap`, `matrix_graph`,
`stable_graph`, `unionfind`, `visit`. The `dot` module emits Graphviz DOT *text*; it computes no
positions. What petgraph gives us that we want: `StableDiGraph` for the Ticket DAG,
`toposort`, `greedy_feedback_arc_set` for cycle breaking, and `NodeIndex` stability across
mutations. Anything that claims petgraph draws graphs is wrong.

**rust-sugiyama.** The reference Rust Sugiyama. Phases are named in the source tree:
`p0_cycle_removal`, `p1_layering`, `p2_reduce_crossings`, `p3_calculate_coordinates`. Cycle removal
uses petgraph's `greedy_feedback_arc_set`. Ranking follows Gansner et al. Crossing reduction offers
weighted-median or barycenter, with Barth/Jünger/Mutzel bilayer cross counting. Coordinates follow
Brandes and Köpf. Entry points: `from_edges(&[(u32, u32)])`, `from_vertices_and_edges`,
`from_graph(&StableDiGraph<V, E>)`, each returning `(layout, width, height)` per connected
component with `NodeIndex` preserved. Configuration via a `Config` builder or environment variables.
No releases for a year, but the algorithm is finished and the API is small. This is the crate
rataflow depends on.

**ascii-dag.** Zero dependencies, and it is both a Sugiyama layout engine and a text renderer.
`compute_layout()` returns positioned nodes and routed edges you can draw yourself with `Canvas` or
a custom widget; `render_string(&RenderOptions)` gives you finished box-drawing text you can drop
into a `Paragraph`. `RenderOptions` covers Unicode or ASCII charset, no colour / Ansi256 / TrueColor,
per-element style callbacks, a legend, and a label policy for edge labels that do not fit.
`LayoutConfig` covers four directions (TB, BT, LR, RL), node spacing and level spacing. It has
subgraph clusters, edge ports declaring which face an edge leaves and arrives on, `scene.hit_test(x, y)`
for mapping a click back to a node, and cycles are handled by reversing back edges and drawing them
dashed rather than refusing to render. `no_std` and no-alloc capable. Golden-file tests in the repo.

The caveat: it is one author, 25 stars, and the version churn is fast. 0.9 in March 2026, 0.10 in
August, 0.11 in September, with a migration guide per bump. The download figure is also misleading:
74,591 of the 79,521 all-time downloads sit on v0.9.1 alone while every other version is in the
double or triple digits, which is the shape of a crawler or a mirror, not adoption. Fine as a
prototype dependency. Not something to pin a product on without re-evaluating.

**renderdag and sapling-renderdag.** These are the git-log lane algorithm as a library.
`sapling-renderdag` 0.1.0 is Meta's, extracted from the Sapling SCM, and it is what `jj log` uses
through its `GraphLog` trait ([jj source](https://github.com/jj-vcs/jj/blob/9ff73e7f/cli/src/graphlog.rs)).
It exposes `GraphRowRenderer` plus `AsciiRenderer`, `AsciiLargeRenderer` and `BoxDrawingRenderer`:
you feed node descriptions with their parents and get back rendered rows. 682,023 downloads all-time
is real adoption, but the last release was 2024-11-12 and the crate is published out of a monorepo
whose root license is GPL-2.0 while the crate metadata says MIT. Confirm that licensing before it
goes anywhere near a product.

`renderdag` 0.4.0 is kdheepak's independent rewrite: edition 2024, MIT, zero runtime dependencies,
`GraphLayout::layout(&[T])` returning a `RowPlan` per node with `TrackCell`s, plus `GraphRenderer`
with a configurable `Glyphs` table and `render_if_changed`. Because it hands back a structured row
plan rather than a finished string, you can colour and style each glyph yourself in ratatui, which
is exactly what a variant would want. It has one star and 339 downloads, so it is unproven socially,
but it is 5,000 lines of tested single-purpose code with no dependency surface to rot.

**layout-rs.** This is the crate people mean when they say "the Rust dagre port", and that
description is not quite right. It is a Graphviz DOT parser plus a Sugiyama-family layout engine
plus rendering *backends*, and the only shipped backend is SVG. Its geometry is `Point`s in `f64`
with bezier-ish arrows sized for pixels (the example uses 100x100 nodes). There is a `RenderBackend`
trait you could implement for a character grid, but you would be reimplementing edge routing from
curve control points onto cells, which is the hard part. 740 stars and 694,350 downloads reflect its
use as an SVG generator, not as a terminal layout engine. Also: no release since 2025-04-24.

**dugong.** Genuinely a dagre port, and a serious one: it backs
[merman](https://github.com/Latias94/merman), a Rust mermaid implementation with 551 stars and daily
commits. But the current version is `0.8.0-alpha.6`, the last stable is 0.7.0, it has no text
renderer, and taking it means taking `dugong-graphlib` plus serde. It is the best dagre-compatible
option in Rust and it is still the wrong shape for this job.

**reingold-tilford.** 1.0.0 from 2019, MIT, no dependencies, 2,385 downloads all-time and 95 in the
last 90 days. Seven years without a release is normally a red flag; for a finished textbook
algorithm with no dependencies it is closer to "done". Only useful for a variant that deliberately
renders the Ticket DAG as a tree.

### Dead ends, stated plainly

- **`dagre` (kookyleo/dagre-rs) is archived.** 5,016 downloads and a fresh-looking 2026-05-06
  release, but the GitHub repo is archived. Do not build on it.
- **The `dagre` name is a swamp.** crates.io currently holds `dagre`, `dagre-rs`, `dagre_rust`,
  `dagre-dgl-rs`, `mermaid-dagre`, `rusty-mermaid-dagre` and `dugong`, all claiming to be dagre
  ports, all published between 2023 and 2026, most with single-digit stars. Only `dugong` has a real
  project behind it.
- **`gen-sugiyama` is on petgraph 0.6** while everything else in this survey is on 0.8. It is a
  component vendored out of a bioinformatics application, not a general library.
- **`rust-sugiyama-fork` is a personal fork** created and released on the same day in August 2026,
  zero stars, 51 downloads. Prefer the original.
- **`elkrs` claims a byte-exact ELK reimplementation** and shipped both its releases on 2026-06-17.
  Interesting, unproven, and ELK is far more machinery than a 16-node DAG needs.
- **`forceatlas2` is AGPL-3.0-only.** Even if force-directed were the right algorithm, which it is
  not, the license makes it unusable here.
- **`fdg-sim` has not released since 2022-12-17** and the `fdg` name itself is not on crates.io.
- **`abstracttui-graph` only works inside AbstractTUI**, its own TUI framework. It is not a ratatui
  crate.
- **`hascii`, `mmdflux` and friends shell out or convert files.** `hascii` (111 downloads total)
  requires a Graphviz binary on the machine. Not a library dependency, a subprocess.

## 4. Prior art: drawing graphs and boards in a terminal

### The git commit graph family

This is where the technique was proven, over twenty years and several independent implementations.

**`git log --graph`** ([graph.c](https://code.googlesource.com/git/+/HEAD/graph.c)). The original.
Keeps a column state machine: `new_columns`, `mapping`, `merge_layout`, and a
`graph_needs_truncation` cap on lane count. Emits an extra, separate row between commits whenever
lanes have to shift sideways, because it refuses to draw a diagonal inside a commit row. That
compromise (a dedicated "link row" between node rows) is what makes the whole thing tractable on a
grid, and every later implementation kept it.

**tig** ([graph.c](https://github.com/jonas/tig/blob/master/src/graph.c), 13,330 stars). Three-row
lookahead: `prev_row`, `row`, `next_row`, `parents`, and a `graph_generate_symbols` pass that picks
the glyph for each cell from the three rows. A `line-graphics` setting with `ascii`, `default`,
`utf-8` and `auto` values, where `auto` means utf-8 only if the locale says so. Its
[issue #527](https://github.com/jonas/tig/issues/527) is the cautionary tale: utf-8 line graphics in
a non-utf-8 locale renders as mojibake, so the charset is a runtime decision, not a compile-time one.

**lazygit**
([graph.go](https://github.com/jesseduffield/lazygit/blob/d167063b/pkg/gui/presentation/graph/graph.go),
82,248 stars). Models the graph as `Pipe`s with `fromPos` and `toPos` per row, computes `GetPipeSets`
for the whole commit list, then renders. Separating "which lanes exist and where do they go" from
"which character goes in this cell" is the cleanest version of this idea in any of these codebases.
And, as noted above, lazygit had to
[change its node glyphs](https://github.com/jesseduffield/lazygit/commit/35d3659ccee590b00f274e5c4d883e1176a4485b)
because `⏣` and `◯` are absent from most monospace fonts.

**jj / Sapling `renderdag`.** jj does not implement a graph renderer. It defines a `GraphLog` trait
and delegates to `renderdag`'s `GraphRowRenderer`, choosing between the ASCII, large-ASCII and
box-drawing renderers. The lesson is the interface: `add_node(id, edges, node_symbol, text)` plus
`width(id, edges)`. The renderer owns the glyph column, the caller owns everything to the right of
it, and `width()` is how the caller knows how much room it has left.

**Zed's git panel**
([git_graph.rs](https://github.com/zed-industries/zed/blob/be52d3b7/crates/git_ui/src/git_graph.rs))
uses a `LaneState` enum with `Empty` and `Active { child, parent, color, starting_row, starting_col,
destination_column, segments }`. Lanes as an explicit state machine with a colour attached, which is
how you get stable per-branch colours across scrolling.

**serie** (2,094 stars, ratatui `^0.30`). The outlier: it gives up on characters entirely and
renders the commit graph as a raster image through the iTerm2 or kitty graphics protocol, with
`--protocol auto|iterm|kitty|kitty-unicode`. The result is genuinely better looking than any
character-grid graph. The cost is in its own
[issue #84](https://github.com/lusingander/serie/issues/84): Alacritty is sixel-only and gets
nothing, and every user on an unsupported terminal sees a blank column. It also depends on
`tui-tree-widget` and `ansi-to-tui`, so the non-graph parts are ordinary ratatui.

**The key insight from all of them:** a general DAG layout is not needed when one axis is already
fixed. Git graphs fix the vertical axis to the commit order and solve only lane assignment on the
horizontal. A Map's Ticket DAG can do exactly the same: fix the vertical axis to topological order
and solve only lane assignment. That turns an NP-hard crossing-minimisation problem into a greedy
sweep with a handful of glyphs, and it is why the git graph family is the most reliable prior art in
this survey.

### Text diagram engines

**Graph::Easy** (Perl, [manual](http://bloodgate.com/perl/graph/manual/layouter.html)). The oldest
serious answer. Its layouter is explicitly a *grid* (Manhattan) layouter: nodes occupy cells,
edges run along cell boundaries, and `as_ascii()` / `as_boxart()` are two charsets over one layout.
Its own documentation admits the limits: two colours only, and arrows that cannot always be placed
where they belong. It is also what `dot-to-ascii` (510 stars) shells out to, because there is no
native Graphviz ASCII backend.

**There is no `dot -Tascii`.** Graphviz has never shipped an ASCII backend. Everything that claims
to be one shells out to Graph::Easy (`dot-to-ascii`), reimplements layout in another language
(`phart` in Python, `mermaid-ascii` in Go), or requires a Graphviz binary and post-processes its
output (`hascii`). The one native attempt,
[dotmatrix](https://forum.graphviz.org/t/dotmatrix-preview-layout-engine-for-monospaced-unicode-fonts/3376),
is a preview registered as a real Graphviz layout engine and renderer for monospaced Unicode fonts,
reporting roughly 170 of ~320 test graphs rendering perfectly. Watch it; do not depend on it.

**mermaid-ascii** (Go, 1,568 stars, active). Flowcharts with `graph LR` and `graph TD`, labelled
edges, subgraphs, colours, plus sequence diagrams. Proves that a full Sugiyama pipeline to
box-drawing characters is achievable and readable at terminal sizes.

**Diagon** (C++, 2,221 stars). Multiple diagram grammars to Unicode art, with
[diagonjs](https://github.com/elmouradiaminedev/diagonjs) as a wasm wrapper. Its value here is
stylistic: it demonstrates how much legibility comes from consistent glyph choice and spacing rather
than from clever routing.

**termaid** (Python) renders 18 mermaid diagram types and is "terminal-aware: auto-fits diagrams to
terminal width with progressive compaction". Progressive compaction is the important idea. When the
diagram does not fit, do not scroll it and do not scale it; simplify it, in stages.

**asciiflow** is a manual drawing tool, not a layout engine. It teaches nothing about automatic
layout and everything about which glyph set reads well by hand.

### Board and panel layouts

**k9s** (34,564 stars) and **lazydocker** (52,794 stars) both render structured data with zero
edges. Panels, tables, a breadcrumb of context, and a key hint bar. Relationships are shown by
selection and by adjacency: select a pod, the panel beside it shows what it relates to. This is the
strongest argument for a variant that draws no edges at all. At 100x30, a board with no edges and a
good selection model conveys a 16-node DAG faster than a drawn graph does.

**blessed-contrib** (15,770 stars, JavaScript). A 12x12 grid layout where each widget declares row,
col, rowSpan and colSpan, over `drawille` braille rendering. Two lessons: a coarse fixed grid makes
dashboards composable, and braille is how you get sub-character resolution in a terminal. ratatui's
`Marker::Braille` is the same technique with the same limits.

### What the prior art teaches about edges on a character grid

| Technique | Where it is proven | What it costs |
|---|---|---|
| Lane assignment with a fixed row order | git, tig, lazygit, jj, Zed | Only works when one axis is pinned. No free layout |
| A separate link row between node rows | git `--graph`, and everything after it | One extra row per rank transition. Buys you never drawing a diagonal |
| Glyph chosen from a 3-row window | tig | Needs lookahead. Removes ambiguity at junctions |
| Pipes as data, glyphs as a later pass | lazygit `GetPipeSets` | Two passes. Buys testability and reuse |
| Unicode box drawing plus a junction table | Graph::Easy, tig, mermaid-ascii | On ratatui 0.30 the table is free: `Cell::merge_symbol` with `MergeStrategy::Fuzzy` |
| Braille for smooth diagonals | blessed-contrib, rataflow | One foreground colour per cell. Font support is not universal |
| Charset chosen at runtime | tig `line-graphics`, ascii-dag `RenderOptions::ascii()` | Two glyph tables to maintain. Buys correctness on every terminal |
| Raster image over the graphics protocol | serie | Unlimited resolution, and nothing at all on terminals without it |
| Progressive compaction when it does not fit | termaid | You must define the simplification stages yourself |
| Give up on edges, use adjacency and selection | k9s, lazydocker | Loses global structure, gains legibility at every size |

## What this means for the five variants

### Genuinely available to us

- **`Cell::merge_symbol` with `MergeStrategy::Exact` or `Fuzzy`.** New in 0.30 and the reason
  hand-drawn box-drawing edges are now cheap. Crossings, corners and T-junctions resolve themselves.
  This is not available through `Canvas`; it is a `Buffer` operation, so a variant that wants merged
  junctions writes cells directly.
- **`Canvas` with `Marker::Braille` or `HalfBlock`** for smooth diagonal strokes at 2x4 or 1x2
  sub-cell resolution, with Cohen-Sutherland clipping already done for `Line`. Labels on a Canvas
  are cell-resolution and drawn on top, which is fine for short Ticket ids and wrong for prose.
- **`rataflow`** for a complete interactive node graph on 0.30 with Sugiyama layout, merged junction
  glyphs and braille strokes already solved.
- **`tui-nodes`** for a much smaller node graph where you own the layout and it gives you a `Rect`
  per node.
- **`rust-sugiyama`** for layered coordinates from a `petgraph::StableDiGraph`, with no rendering
  opinion at all.
- **`ascii-dag`** for a finished box-drawing rendering of a DAG in one call, or for
  `compute_layout()` if you want to draw it yourself. Four directions, clusters, hit testing, and an
  ASCII fallback charset.
- **`renderdag`** for a git-log-style lane plan per row, as structured data you style yourself.
- **`tui-markdown`** for Map prose sections as styled `Text`, **`tui-scrollview`** for scrolling a
  virtual area, **`tui-widget-list`** for multi-line Ticket cards, **`tui-tree-widget`** for a tree
  projection, **`tui-popup`** for a Ticket detail overlay.

### Traps

- **Force-directed layout.** `ascii-petgraph` is the crate that offers it and it is stuck on ratatui
  0.29; `forceatlas2` is AGPL; `fdg-sim` is four years stale. Even with a working implementation the
  output does not survive rounding to cells, and it cannot express direction of flow.
- **`layout-rs` as a terminal layout engine.** It is a DOT parser with an SVG backend. Writing a
  character-grid `RenderBackend` means reimplementing edge routing from curve geometry.
- **Anything on ratatui 0.29 or earlier.** `ascii-petgraph`, `tuiflow`, `ratatui-markdown`,
  `ratatui-graph`. They pull a second ratatui into the tree and their widgets do not satisfy 0.30's
  traits. There is no cheap workaround.
- **Exotic glyphs without a fallback.** `Marker::Octant` and `Marker::Sextant` are the newest
  Unicode blocks and the least supported. lazygit and tig both had to retreat on this.
- **`tui-scrollview` over a very tall virtual area.** It allocates the whole content-size `Buffer`.
  Fine for one Map; not fine for a 200x2000 canvas rebuilt every frame.
- **Measuring wrapped text on stable.** `Paragraph::line_count` is behind
  `unstable-rendered-line-info`. A variant that needs an accurate scrollbar must plan for this.
- **A general DAG layout when the DAG is 6 to 16 shallow nodes.** Every git tool in area 4 pins one
  axis and solves only the other. Reaching for full crossing minimisation on 16 nodes is effort
  spent where no user will see it.

### Combinations that are genuinely viable at 100x30 up to 200x50

1. **`rataflow` with its built-in Sugiyama.** The most capable node graph on 0.30, with panning,
   zooming, selection, minimap and theming already done. Highest ceiling, newest crate.
2. **`petgraph` plus `rust-sugiyama` plus hand-drawn cells with `merge_symbol`.** Layers become rows,
   within-layer order becomes columns, edges are drawn as orthogonal runs with junctions merged by
   ratatui. Total control, no rendering dependency, and the layout crate is replaceable.
3. **`renderdag` lanes down the left, Ticket text to the right.** The git-log model applied directly
   to a Map: topological order fixes the rows, lane assignment handles the forks and joins, and the
   remaining width is a normal list you can style per Ticket type and status. Works at 100 columns
   without compromise and degrades to pure ASCII trivially.
4. **`ascii-dag` rendered to a string and shown in a `Paragraph`, inside `tui-scrollview`.** The
   fastest path to a real drawn DAG, with an ASCII charset switch and four layout directions, at the
   cost of styling only through its own callbacks rather than ratatui's.
5. **Topological columns with no drawn edges at all, k9s style.** Buckets by depth or by status,
   each a `tui-widget-list` of Ticket cards, with `blocked_by` shown as a badge and the selected
   Ticket's edges highlighted in the other columns. Never overflows, never needs a fallback charset,
   and is the only option that is equally legible at 100x30 and 200x50.

`tui-nodes` and `ratatui-flow` are the fallback if rataflow disappoints; both are 0.30-current and
both hand you node rectangles, `ratatui-flow` with automatic layout and `tui-nodes` without. Anything
built on force-directed layout, on an SVG-first engine, or on a pre-0.30 ratatui should not be
attempted.
