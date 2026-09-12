# Throwaway Map visualization prototype. Recipes run from the repo root.
set working-directory := '..'

[doc("Run the prototype. Optional argument: the directory holding the Maps")]
map *DIR:
    cargo run --example map_prototype -p map_tui -- {{ DIR }}

[doc("Print what the fixture loader parsed, without starting the terminal")]
list *DIR:
    cargo run --example map_prototype -p map_tui -- --list {{ DIR }}
