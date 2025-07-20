pub mod events;
pub mod widgets;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::app::{App, Tab};
use crate::ui::widgets::{QueueWidget, TaskWidget, WorkerWidget};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Draw header with tabs
    draw_header(f, app, chunks[0]);

    // Draw main content based on selected tab
    match app.selected_tab {
        Tab::Workers => WorkerWidget::draw(f, app, chunks[1]),
        Tab::Tasks => TaskWidget::draw(f, app, chunks[1]),
        Tab::Queues => QueueWidget::draw(f, app, chunks[1]),
    }

    // Draw status bar
    draw_status_bar(f, app, chunks[2]);

    // Draw help overlay if active
    if app.show_help {
        draw_help(f);
    }

    // Draw confirmation dialog if active
    if app.show_confirmation {
        draw_confirmation_dialog(f, app);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
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
                .title(" LazyCelery v0.2.0 "),
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

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
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
    let key_hints = if app.show_confirmation {
        "[y/Enter] Confirm | [n/Esc] Cancel"
    } else if app.is_searching {
        "[Enter] Confirm | [Esc] Cancel"
    } else {
        match app.selected_tab {
            Tab::Queues => "[Tab] Switch | [↑↓] Navigate | [p] Purge | [/] Search | [?] Help | [q] Quit",
            Tab::Tasks => "[Tab] Switch | [↑↓] Navigate | [r] Retry | [x] Revoke | [/] Search | [?] Help | [q] Quit",
            _ => "[Tab] Switch | [↑↓] Navigate | [/] Search | [?] Help | [q] Quit",
        }
    };

    let status_right_widget = Block::default()
        .borders(Borders::ALL)
        .title(Span::raw(key_hints));
    f.render_widget(status_right_widget, status_chunks[1]);
}

fn draw_help(f: &mut Frame) {
    use ratatui::widgets::{Clear, Paragraph, Wrap};

    let area = centered_rect(60, 60, f.size());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from("LazyCelery - Keyboard Shortcuts"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  Tab       - Switch between tabs"),
        Line::from("  ↑/k       - Move up"),
        Line::from("  ↓/j       - Move down"),
        Line::from("  Enter     - View details"),
        Line::from("  Esc       - Go back"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  /         - Search"),
        Line::from("  p         - Purge queue (in Queues tab)"),
        Line::from("  r         - Retry task (in Tasks tab)"),
        Line::from("  x         - Revoke task (in Tasks tab)"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  ?         - Toggle this help"),
        Line::from("  q         - Quit application"),
        Line::from(""),
        Line::from("Press any key to close this help..."),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

fn draw_confirmation_dialog(f: &mut Frame, app: &App) {
    use ratatui::widgets::{Clear, Paragraph, Wrap};

    let area = centered_rect(50, 30, f.size());
    f.render_widget(Clear, area);

    let confirmation_text = vec![
        Line::from(""),
        Line::from(app.confirmation_message.clone()),
        Line::from(""),
        Line::from("Press [y/Enter] to confirm or [n/Esc] to cancel"),
    ];

    let confirmation = Paragraph::new(confirmation_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirmation ")
                .style(Style::default().bg(Color::Black).fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(confirmation, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
