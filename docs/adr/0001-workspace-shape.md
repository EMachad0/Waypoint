# Root manifest is both the binary package and the workspace root

The project ships one executable and needs libraries beside it, so `Cargo.toml` at the root keeps
the `wayfinder_solver` package and also declares `[workspace] members = ["crates/*"]`. Libraries
live under `crates/`, the binary stays at the root, and `cargo run` needs no `-p` or
`default-members`.

## Considered options

A virtual manifest at the root, with the binary moved to `crates/wayfinder_solver` beside the
libraries. Rejected: it buys uniformity we cannot spend, since there is exactly one executable and
no prospect of a second, and it costs a `default-members` entry to keep `cargo run` working.

## Consequences

The root manifest carries both `[dependencies]` for the binary and `[workspace.dependencies]` for
the members, which reads oddly until you know why. If a second executable ever appears, the root
package moves into `crates/` and this record is superseded rather than edited.
