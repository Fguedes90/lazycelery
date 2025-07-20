use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::TaskStatus;
use chrono::Utc;

pub struct TaskWidget;

impl TaskWidget {
    pub fn draw(f: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Draw task list
        Self::draw_task_list(f, app, chunks[0]);

        // Draw task details
        Self::draw_task_details(f, app, chunks[1]);
    }

    fn draw_task_list(f: &mut Frame, app: &App, area: Rect) {
        let filtered_tasks = app.get_filtered_tasks();
        
        let header = Row::new(vec!["ID", "Name", "Status", "Worker", "Duration"])
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(1);

        // Calculate viewport
        let height = area.height.saturating_sub(4) as usize; // Account for borders and header
        
        if filtered_tasks.is_empty() {
            let no_tasks = Row::new(vec![
                Cell::from(""),
                Cell::from("No tasks found"),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ]).style(Style::default().fg(Color::DarkGray));
            
            let table = Table::new(
                vec![no_tasks],
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(30),
                    Constraint::Percentage(15),
                    Constraint::Percentage(20),
                    Constraint::Percentage(15),
                ],
            )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(" Tasks (0) "));
            
            f.render_widget(table, area);
            return;
        }
        
        let selected = app.selected_task.min(filtered_tasks.len().saturating_sub(1));
        
        // Calculate the start of the viewport to ensure selected item is visible
        let start = if selected >= height && height > 0 {
            selected.saturating_sub(height / 2)
        } else {
            0
        };
        
        let end = (start + height).min(filtered_tasks.len());
        let visible_tasks = &filtered_tasks[start..end];

        let rows: Vec<Row> = visible_tasks
            .iter()
            .enumerate()
            .map(|(idx, task)| {
                let actual_idx = start + idx;
                let status_color = match task.status {
                    TaskStatus::Success => Color::Green,
                    TaskStatus::Failure => Color::Red,
                    TaskStatus::Active => Color::Yellow,
                    TaskStatus::Pending => Color::Gray,
                    TaskStatus::Retry => Color::Magenta,
                    TaskStatus::Revoked => Color::DarkGray,
                };

                let duration = task.duration_since(Utc::now());
                let duration_str = format!(
                    "{:02}:{:02}:{:02}",
                    duration.num_hours(),
                    duration.num_minutes() % 60,
                    duration.num_seconds() % 60
                );

                let row = Row::new(vec![
                    Cell::from(task.id.clone()),
                    Cell::from(task.name.clone()),
                    Cell::from(format!("{:?}", task.status))
                        .style(Style::default().fg(status_color)),
                    Cell::from(task.worker.as_deref().unwrap_or("-")),
                    Cell::from(duration_str),
                ]);

                if actual_idx == app.selected_task {
                    row.style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    row
                }
            })
            .collect();

        // Add scroll indicator to title
        let scroll_info = if filtered_tasks.len() > height {
            format!(" [{}/{}]", app.selected_task + 1, filtered_tasks.len())
        } else {
            String::new()
        };
        
        let title = if app.is_searching {
            format!(" Tasks (filtered: {}/{}){} ", filtered_tasks.len(), app.tasks.len(), scroll_info)
        } else {
            format!(" Tasks ({}){} ", app.tasks.len(), scroll_info)
        };

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(30),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(15),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        f.render_widget(table, area);
    }

    fn draw_task_details(f: &mut Frame, app: &App, area: Rect) {
        let filtered_tasks = app.get_filtered_tasks();
        
        if filtered_tasks.is_empty() {
            let no_tasks = Paragraph::new("No tasks found")
                .block(Block::default().borders(Borders::ALL).title(" Task Details "));
            f.render_widget(no_tasks, area);
            return;
        }

        let selected = app.selected_task.min(filtered_tasks.len().saturating_sub(1));
        if let Some(task) = filtered_tasks.get(selected) {
            let mut lines = vec![
                Line::from(vec![
                    Span::raw("ID: "),
                    Span::styled(&task.id, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("Name: "),
                    Span::styled(&task.name, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        format!("{:?}", task.status),
                        Style::default().fg(match task.status {
                            TaskStatus::Success => Color::Green,
                            TaskStatus::Failure => Color::Red,
                            TaskStatus::Active => Color::Yellow,
                            TaskStatus::Pending => Color::Gray,
                            TaskStatus::Retry => Color::Magenta,
                            TaskStatus::Revoked => Color::DarkGray,
                        }),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Worker: "),
                    Span::raw(task.worker.as_deref().unwrap_or("None")),
                ]),
                Line::from(vec![
                    Span::raw("Timestamp: "),
                    Span::raw(task.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
                ]),
            ];

            if !task.args.is_empty() && task.args != "[]" {
                lines.push(Line::from(vec![
                    Span::raw("Args: "),
                    Span::raw(&task.args),
                ]));
            }

            if !task.kwargs.is_empty() && task.kwargs != "{}" {
                lines.push(Line::from(vec![
                    Span::raw("Kwargs: "),
                    Span::raw(&task.kwargs),
                ]));
            }

            if let Some(result) = &task.result {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::raw("Result: "),
                    Span::styled(result, Style::default().fg(Color::Green)),
                ]));
            }

            if let Some(traceback) = &task.traceback {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Traceback:",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]));
                for line in traceback.lines() {
                    lines.push(Line::from(vec![Span::styled(
                        line,
                        Style::default().fg(Color::Red),
                    )]));
                }
            }

            let details = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(" Task Details "))
                .wrap(Wrap { trim: false });

            f.render_widget(details, area);
        }
    }
}
