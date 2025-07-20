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

    if app.show_confirmation {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                // Confirmation dialog will be handled in main loop
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.hide_confirmation_dialog();
            }
            _ => {}
        }
        return;
    }

    if app.show_help {
        app.toggle_help();
        return;
    }

    if app.show_task_details {
        app.hide_task_details();
        return;
    }

    // Clear status message on any key press (except actions that set new status)
    match key.code {
        KeyCode::Char('p')
        | KeyCode::Char('r')
        | KeyCode::Char('x')
        | KeyCode::Enter
        | KeyCode::Char('d') => {
            // These will set their own status messages or open modals
        }
        _ => {
            app.clear_status_message();
        }
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Tab => app.next_tab(),
        KeyCode::BackTab => app.previous_tab(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Char('/') => app.start_search(),
        KeyCode::Char('p') => app.initiate_purge_queue(),
        KeyCode::Char('r') => app.initiate_retry_task(),
        KeyCode::Char('x') => app.initiate_revoke_task(),
        KeyCode::Enter | KeyCode::Char('d') => app.show_task_details(),
        _ => {}
    }
}
