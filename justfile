mod wk ".just/wk.just"
mod check ".just/check.just"

set dotenv-load := true

# List all recipes, including those in modules.
default:
    @just --list --list-submodules
