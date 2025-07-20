use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::models::WorkerStatus;

pub struct WorkerWidget;

impl WorkerWidget {
    pub fn draw(f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Draw worker list on the left
        Self::draw_worker_list(f, app, chunks[0]);

        // Draw worker details on the right
        Self::draw_worker_details(f, app, chunks[1]);
    }

    fn draw_worker_list(f: &mut Frame, app: &App, area: Rect) {
        let workers: Vec<ListItem> = app
            .workers
            .iter()
            .enumerate()
            .map(|(idx, worker)| {
                let status_symbol = match worker.status {
                    WorkerStatus::Online => "●",
                    WorkerStatus::Offline => "○",
                };
                let status_color = match worker.status {
                    WorkerStatus::Online => Color::Green,
                    WorkerStatus::Offline => Color::Red,
                };

                let content = Line::from(vec![
                    Span::styled(status_symbol, Style::default().fg(status_color)),
                    Span::raw(" "),
                    Span::raw(&worker.hostname),
                ]);

                if idx == app.selected_worker {
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

        let workers_list = List::new(workers)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Workers ({}) ", app.workers.len())),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(workers_list, area);
    }

    fn draw_worker_details(f: &mut Frame, app: &App, area: Rect) {
        if app.workers.is_empty() {
            let no_workers = Paragraph::new("No workers found")
                .block(Block::default().borders(Borders::ALL).title(" Worker Details "));
            f.render_widget(no_workers, area);
            return;
        }

        if let Some(worker) = app.workers.get(app.selected_worker) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(10), Constraint::Min(0)])
                .split(area);

            // Worker info section
            let info_lines = vec![
                Line::from(vec![
                    Span::raw("Hostname: "),
                    Span::styled(&worker.hostname, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        match worker.status {
                            WorkerStatus::Online => "Online",
                            WorkerStatus::Offline => "Offline",
                        },
                        Style::default().fg(match worker.status {
                            WorkerStatus::Online => Color::Green,
                            WorkerStatus::Offline => Color::Red,
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Concurrency: "),
                    Span::raw(worker.concurrency.to_string()),
                ]),
                Line::from(vec![
                    Span::raw("Active Tasks: "),
                    Span::raw(format!("{}/{}", worker.active_tasks.len(), worker.concurrency)),
                ]),
                Line::from(vec![
                    Span::raw("Utilization: "),
                    Span::raw(format!("{:.1}%", worker.utilization())),
                ]),
                Line::from(vec![
                    Span::raw("Processed: "),
                    Span::styled(
                        worker.processed.to_string(),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Failed: "),
                    Span::styled(
                        worker.failed.to_string(),
                        Style::default().fg(Color::Red),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Queues: "),
                    Span::raw(worker.queues.join(", ")),
                ]),
            ];

            let info = Paragraph::new(info_lines)
                .block(Block::default().borders(Borders::ALL).title(" Worker Details "));
            f.render_widget(info, chunks[0]);

            // Active tasks section
            if !worker.active_tasks.is_empty() {
                let task_rows: Vec<Row> = worker
                    .active_tasks
                    .iter()
                    .map(|task_id| {
                        Row::new(vec![task_id.clone()])
                    })
                    .collect();

                let tasks_table = Table::new(
                    task_rows,
                    [Constraint::Percentage(100)],
                )
                .block(Block::default().borders(Borders::ALL).title(" Active Tasks "))
                .header(
                    Row::new(vec!["Task ID"])
                        .style(Style::default().fg(Color::Yellow))
                        .bottom_margin(1),
                );

                f.render_widget(tasks_table, chunks[1]);
            } else {
                let no_tasks = Paragraph::new("No active tasks")
                    .block(Block::default().borders(Borders::ALL).title(" Active Tasks "));
                f.render_widget(no_tasks, chunks[1]);
            }
        }
    }
}
