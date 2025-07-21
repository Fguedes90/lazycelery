use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Row, Table},
    Frame,
};

use super::base::{helpers, Widget};
use crate::app::App;
use crate::models::WorkerStatus;

pub struct WorkerWidget;

impl Widget for WorkerWidget {
    fn draw(f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Draw worker list on the left
        Self::draw_list(f, app, chunks[0]);

        // Draw worker details on the right
        Self::draw_details(f, app, chunks[1]);
    }

    fn draw_list(f: &mut Frame, app: &App, area: Rect) {
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
                    ListItem::new(content).style(helpers::selection_style())
                } else {
                    ListItem::new(content)
                }
            })
            .collect();

        let title = format!("Workers ({})", app.workers.len());
        let workers_list = List::new(workers)
            .block(helpers::titled_block(&title))
            .highlight_style(helpers::selection_style());

        f.render_widget(workers_list, area);
    }

    fn draw_details(f: &mut Frame, app: &App, area: Rect) {
        if app.workers.is_empty() {
            f.render_widget(helpers::no_data_message("workers"), area);
            return;
        }

        if let Some(worker) = app.workers.get(app.selected_worker) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(10), Constraint::Min(0)])
                .split(area);

            // Worker info section
            let info_lines = vec![
                helpers::highlighted_field_line("Hostname", &worker.hostname, Color::Cyan),
                helpers::status_line(
                    "Status",
                    match worker.status {
                        WorkerStatus::Online => "Online",
                        WorkerStatus::Offline => "Offline",
                    },
                    match worker.status {
                        WorkerStatus::Online => Color::Green,
                        WorkerStatus::Offline => Color::Red,
                    },
                ),
                helpers::field_line("Concurrency", &worker.concurrency.to_string()),
                helpers::field_line(
                    "Active Tasks",
                    &format!("{}/{}", worker.active_tasks.len(), worker.concurrency),
                ),
                helpers::field_line("Utilization", &format!("{:.1}%", worker.utilization())),
                helpers::highlighted_field_line(
                    "Processed",
                    &worker.processed.to_string(),
                    Color::Green,
                ),
                helpers::highlighted_field_line("Failed", &worker.failed.to_string(), Color::Red),
                helpers::field_line("Queues", &worker.queues.join(", ")),
            ];

            let info = Paragraph::new(info_lines).block(helpers::titled_block("Worker Details"));
            f.render_widget(info, chunks[0]);

            // Active tasks section
            if !worker.active_tasks.is_empty() {
                let task_rows: Vec<Row> = worker
                    .active_tasks
                    .iter()
                    .map(|task_id| Row::new(vec![task_id.clone()]))
                    .collect();

                let tasks_table = Table::new(task_rows, [Constraint::Percentage(100)])
                    .block(helpers::titled_block("Active Tasks"))
                    .header(
                        Row::new(vec!["Task ID"])
                            .style(Style::default().fg(Color::Yellow))
                            .bottom_margin(1),
                    );

                f.render_widget(tasks_table, chunks[1]);
            } else {
                let no_tasks =
                    Paragraph::new("No active tasks").block(helpers::titled_block("Active Tasks"));
                f.render_widget(no_tasks, chunks[1]);
            }
        }
    }
}
