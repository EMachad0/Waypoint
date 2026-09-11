# Quality gates for the Rust workspace. Recipes run from the repo root.
set working-directory := '..'

[doc("Format check, lint, and test")]
all: fmt lint test

[doc("Check formatting")]
fmt:
    cargo fmt --all --check

[doc("Lint with clippy (warnings are errors)")]
lint:
    cargo clippy --workspace -- -D warnings

[doc("Run tests")]
test:
    cargo test --workspace
