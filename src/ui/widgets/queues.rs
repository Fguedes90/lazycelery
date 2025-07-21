use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Gauge, List, ListItem, Paragraph},
    Frame,
};

use super::base::{helpers, Widget};
use crate::app::App;

pub struct QueueWidget;

impl Widget for QueueWidget {
    fn draw(f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Draw queue list on the left
        Self::draw_list(f, app, chunks[0]);

        // Draw queue details on the right
        Self::draw_details(f, app, chunks[1]);
    }

    fn draw_list(f: &mut Frame, app: &App, area: Rect) {
        let queues: Vec<ListItem> = app
            .queues
            .iter()
            .enumerate()
            .map(|(idx, queue)| {
                let status_color = if queue.length > 100 {
                    Color::Red
                } else if queue.length > 50 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                let content = Line::from(vec![
                    Span::raw(&queue.name),
                    Span::raw("   "),
                    Span::styled(queue.length.to_string(), Style::default().fg(status_color)),
                ]);

                if idx == app.selected_queue {
                    ListItem::new(content).style(helpers::selection_style())
                } else {
                    ListItem::new(content)
                }
            })
            .collect();

        let title = format!("Queues ({})", app.queues.len());
        let queues_list = List::new(queues)
            .block(helpers::titled_block(&title))
            .highlight_style(helpers::selection_style());

        f.render_widget(queues_list, area);
    }

    fn draw_details(f: &mut Frame, app: &App, area: Rect) {
        if app.queues.is_empty() {
            f.render_widget(helpers::no_data_message("queues"), area);
            return;
        }

        if let Some(queue) = app.queues.get(app.selected_queue) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(area);

            // Queue info
            let info_lines = vec![
                helpers::highlighted_field_line("Queue Name", &queue.name, Color::Cyan),
                helpers::status_line(
                    "Messages",
                    &queue.length.to_string(),
                    if queue.length > 100 {
                        Color::Red
                    } else if queue.length > 50 {
                        Color::Yellow
                    } else {
                        Color::Green
                    },
                ),
                helpers::field_line("Consumers", &queue.consumers.to_string()),
                helpers::status_line(
                    "Status",
                    if queue.has_consumers() {
                        "Active"
                    } else if queue.is_empty() {
                        "Empty"
                    } else {
                        "No consumers"
                    },
                    if queue.has_consumers() {
                        Color::Green
                    } else if queue.is_empty() {
                        Color::Gray
                    } else {
                        Color::Yellow
                    },
                ),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "[p] Purge queue (requires confirmation)",
                    Style::default().fg(Color::DarkGray),
                )]),
            ];

            let info = Paragraph::new(info_lines).block(helpers::titled_block("Queue Details"));
            f.render_widget(info, chunks[0]);

            // Queue fill gauge
            let max_queue_size = 1000; // Configurable max for visualization
            let ratio = (queue.length as f64 / max_queue_size as f64).min(1.0);
            let gauge = Gauge::default()
                .block(helpers::titled_block("Queue Fill"))
                .gauge_style(Style::default().fg(if queue.length > 100 {
                    Color::Red
                } else if queue.length > 50 {
                    Color::Yellow
                } else {
                    Color::Green
                }))
                .ratio(ratio)
                .label(format!("{}/{}", queue.length, max_queue_size));
            f.render_widget(gauge, chunks[1]);

            // Additional info or actions
            let actions = Paragraph::new(vec![
                Line::from("Available Actions:"),
                Line::from(""),
                Line::from("- View messages (coming soon)"),
                Line::from("- Purge queue (coming soon)"),
                Line::from("- Export messages (coming soon)"),
            ])
            .block(helpers::titled_block("Actions"));
            f.render_widget(actions, chunks[2]);
        }
    }
}
