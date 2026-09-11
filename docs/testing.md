# Testing

## When to use

Read this before writing or changing any test, and before designing code that has to be testable.

The workspace has no crates yet, so this doc holds the conventions only. Add the worked examples
(fakes, feature flags, harness setup) once there is real code to show.

## Two layers, kept in separate places

Unit tests live in the module they cover, inside a `#[cfg(test)] mod tests` block. They may reach
into the module's internals directly (`pub(crate)` items, private helpers). Anything only reachable
through a crate-internal item is tested here.

End-to-end tests live in `crates/<crate>/tests/` and drive the crate through its public API, never
crate internals. They prove a consumer can use the crate, which catches the case where the
internals work but the crate is unusable from outside.

## Conventions

- Start every colocated `mod tests` with `use super::*;` so the module's items are in scope without
  repeating imports in each test.
- Name tests after the behavior, in `snake_case`, as a sentence. The name reads like a spec line:
  `connecting_resolves_to_connected_when_the_build_lands`, not `test_connect`.
- One behavior per test. Splitting them means a failure points at exactly one thing.
- Cover happy and unhappy paths. For each piece of behavior, test the success path and the failure
  and edge paths: errors, caps, overflow, empty input. Tests pair up, one per path.
- Assertion messages carry the why. When the values alone do not show an assertion's intent, add a
  message that states the expected behavior. It documents intent and makes the failure
  self-explaining:

  ```rust
  assert_eq!(
      probe.connects(),
      1,
      "Connecting must suppress auto-reconnect, no second build may be kicked in flight",
  );
  ```

- Test behavior, not implementation. Assert on observable results through the interface, not on
  internal call counts or private state. A test should survive an internal refactor that preserves
  behavior.
- Design for testability. Accept dependencies instead of constructing them internally, so a fake
  can be injected, and prefer returning results over hidden side effects.

## Running tests

- `just check::test` runs the workspace test suite.
- `cargo test --workspace` is the underlying command.
- `just check::all` runs fmt, clippy, and test.

## References

- Domain vocabulary for test and interface names lives in `docs/CONTEXT.md`.
