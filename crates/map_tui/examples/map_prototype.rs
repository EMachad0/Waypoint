//! Launcher for the throwaway Map visualization prototype.

fn main() -> std::io::Result<()> {
    if std::env::args().any(|arg| arg == "--list") {
        map_tui::prototype::list();
        return Ok(());
    }
    map_tui::prototype::run()
}
