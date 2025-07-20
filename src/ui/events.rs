use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

#[allow(dead_code)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Refresh,
}

pub async fn next_event(tick_rate: Duration) -> Result<AppEvent, std::io::Error> {
    if event::poll(tick_rate)? {
        match event::read()? {
            Event::Key(key) => Ok(AppEvent::Key(key)),
            _ => Ok(AppEvent::Tick),
        }
    } else {
        Ok(AppEvent::Tick)
    }
}

pub fn handle_key_event(key: KeyEvent, app: &mut crate::app::App) {
    if app.is_searching {
        match key.code {
            KeyCode::Esc => app.stop_search(),
            KeyCode::Enter => app.stop_search(),
            KeyCode::Char(c) => app.search_query.push(c),
            KeyCode::Backspace => {
                app.search_query.pop();
            }
            _ => {}
        }
        return;
    }

    if app.show_help {
        app.toggle_help();
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Tab => app.next_tab(),
        KeyCode::BackTab => app.previous_tab(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Char('/') => app.start_search(),
        _ => {}
    }
}
