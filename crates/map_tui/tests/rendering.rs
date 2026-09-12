use map_tui::App;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn a_frame_is_one_bordered_block_filling_the_terminal() {
    let app = App::new();
    let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();

    terminal.draw(|frame| app.draw(frame)).unwrap();

    terminal.backend().assert_buffer_lines([
        "┌map_tui───────────┐",
        "│                  │",
        "│                  │",
        "│                  │",
        "└──────────────────┘",
    ]);
}
