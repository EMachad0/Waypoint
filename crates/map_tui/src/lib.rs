use std::io;

use ratatui::Frame;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::Block;

#[derive(Default)]
pub struct App {
    quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let ctrl_c =
            key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl_c || matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
            self.quit = true;
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(Block::bordered().title("map_tui"), frame.area());
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }
}

/// Runs the application on the terminal, restoring it on exit and on panic.
pub fn run() -> io::Result<()> {
    ratatui::run(|terminal| {
        let mut app = App::new();
        while !app.should_quit() {
            terminal.draw(|frame| app.draw(frame))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                app.handle_key(key);
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_asks_the_application_to_quit() {
        let mut app = App::new();

        app.handle_key(KeyEvent::from(KeyCode::Char('q')));

        assert!(app.should_quit());
    }

    #[test]
    fn esc_asks_the_application_to_quit() {
        let mut app = App::new();

        app.handle_key(KeyEvent::from(KeyCode::Esc));

        assert!(app.should_quit());
    }

    #[test]
    fn ctrl_c_asks_the_application_to_quit() {
        let mut app = App::new();

        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        assert!(app.should_quit());
    }

    #[test]
    fn an_unbound_key_leaves_the_application_running() {
        let mut app = App::new();

        app.handle_key(KeyEvent::from(KeyCode::Char('c')));

        assert!(
            !app.should_quit(),
            "Only q, Esc and Ctrl-C quit, a bare c must not",
        );
    }
}
