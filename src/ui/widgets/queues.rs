use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub struct QueueWidget;

impl QueueWidget {
    pub fn draw(f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Draw queue list on the left
        Self::draw_queue_list(f, app, chunks[0]);

        // Draw queue details on the right
        Self::draw_queue_details(f, app, chunks[1]);
    }

    fn draw_queue_list(f: &mut Frame, app: &App, area: Rect) {
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
                    ListItem::new(content).style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(content)
                }
            })
            .collect();

        let queues_list = List::new(queues)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Queues ({}) ", app.queues.len())),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(queues_list, area);
    }

    fn draw_queue_details(f: &mut Frame, app: &App, area: Rect) {
        if app.queues.is_empty() {
            let no_queues = Paragraph::new("No queues found").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Queue Details "),
            );
            f.render_widget(no_queues, area);
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
                Line::from(vec![
                    Span::raw("Queue Name: "),
                    Span::styled(&queue.name, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("Messages: "),
                    Span::styled(
                        queue.length.to_string(),
                        Style::default().fg(if queue.length > 100 {
                            Color::Red
                        } else if queue.length > 50 {
                            Color::Yellow
                        } else {
                            Color::Green
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Consumers: "),
                    Span::raw(queue.consumers.to_string()),
                ]),
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        if queue.has_consumers() {
                            "Active"
                        } else if queue.is_empty() {
                            "Empty"
                        } else {
                            "No consumers"
                        },
                        Style::default().fg(if queue.has_consumers() {
                            Color::Green
                        } else if queue.is_empty() {
                            Color::Gray
                        } else {
                            Color::Yellow
                        }),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "[p] Purge queue (requires confirmation)",
                    Style::default().fg(Color::DarkGray),
                )]),
            ];

            let info = Paragraph::new(info_lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Queue Details "),
            );
            f.render_widget(info, chunks[0]);

            // Queue fill gauge
            let max_queue_size = 1000; // Configurable max for visualization
            let ratio = (queue.length as f64 / max_queue_size as f64).min(1.0);
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Queue Fill "))
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
            .block(Block::default().borders(Borders::ALL).title(" Actions "));
            f.render_widget(actions, chunks[2]);
        }
    }
}
