mod wk ".just/wk.just"
mod check ".just/check.just"
mod proto ".just/proto.just"

# List all recipes, including those in modules.
default:
    @just --list --list-submodules
