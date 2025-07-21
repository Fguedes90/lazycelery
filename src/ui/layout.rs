use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::app::{App, Tab};

/// Draw the header section with tab navigation
pub fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Workers", "Queues", "Tasks"];
    let selected = match app.selected_tab {
        Tab::Workers => 0,
        Tab::Queues => 1,
        Tab::Tasks => 2,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" LazyCelery v0.4.0 "),
        )
        .select(selected)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    f.render_widget(tabs, area);
}

/// Draw the status bar with information and key hints
pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left side - general info or status message
    let status_left = if !app.status_message.is_empty() {
        app.status_message.clone()
    } else if app.is_searching {
        format!("Search: {}_", app.search_query)
    } else {
        format!(
            "Workers: {} | Tasks: {} | Queues: {}",
            app.workers.len(),
            app.tasks.len(),
            app.queues.len()
        )
    };

    let status_left_widget = Block::default()
        .borders(Borders::ALL)
        .title(Span::raw(status_left));
    f.render_widget(status_left_widget, status_chunks[0]);

    // Right side - key hints
    let key_hints = get_key_hints(app);

    let status_right_widget = Block::default()
        .borders(Borders::ALL)
        .title(Span::raw(key_hints));
    f.render_widget(status_right_widget, status_chunks[1]);
}

/// Get appropriate key hints based on current application state
fn get_key_hints(app: &App) -> &'static str {
    if app.show_confirmation {
        "[y/Enter] Confirm | [n/Esc] Cancel"
    } else if app.show_task_details {
        "[Any key] Close details"
    } else if app.is_searching {
        "[Enter] Confirm | [Esc] Cancel"
    } else {
        match app.selected_tab {
            Tab::Queues => "[Tab] Switch | [↑↓] Navigate | [p] Purge | [/] Search | [?] Help | [q] Quit",
            Tab::Tasks => "[Tab] Switch | [↑↓] Navigate | [Enter/d] Details | [r] Retry | [x] Revoke | [/] Search | [?] Help | [q] Quit",
            _ => "[Tab] Switch | [↑↓] Navigate | [/] Search | [?] Help | [q] Quit",
        }
    }
}

/// Create the main application layout with header, content, and status bar
pub fn create_main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(area)
        .to_vec()
}

/// Create a centered rectangle for modal dialogs
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
