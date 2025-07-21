use crate::app::App;
use ratatui::{layout::Rect, Frame};

/// Common trait for all UI widgets that display data with list and details sections
pub trait Widget {
    /// Draw the complete widget with its list and details sections
    fn draw(f: &mut Frame, app: &App, area: Rect);

    /// Draw the list section (left or top panel)
    fn draw_list(f: &mut Frame, app: &App, area: Rect);

    /// Draw the details section (right or bottom panel)
    fn draw_details(f: &mut Frame, app: &App, area: Rect);
}

/// Common helper functions for widget styling and layout
pub mod helpers {
    use ratatui::{
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Paragraph},
    };

    /// Create a standard selection style for highlighted items
    pub fn selection_style() -> Style {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    }

    /// Create a standard block with borders and title
    pub fn titled_block(title: &str) -> Block {
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {title} "))
    }

    /// Create a standard "no data" message
    pub fn no_data_message(item_type: &str) -> Paragraph {
        let message = format!("No {item_type} found");
        let title = format!("{item_type} Details");
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .border_type(BorderType::Rounded)
            .title(format!(" {title} "));
        Paragraph::new(message).block(block)
    }

    /// Create a colored status indicator line
    pub fn status_line(label: &str, value: &str, color: Color) -> Line<'static> {
        Line::from(vec![
            Span::raw(format!("{label}: ")),
            Span::styled(value.to_string(), Style::default().fg(color)),
        ])
    }

    /// Create a field line with label and value
    pub fn field_line(label: &str, value: &str) -> Line<'static> {
        Line::from(vec![
            Span::raw(format!("{label}: ")),
            Span::raw(value.to_string()),
        ])
    }

    /// Create a highlighted field line (for important values)
    pub fn highlighted_field_line(label: &str, value: &str, color: Color) -> Line<'static> {
        Line::from(vec![
            Span::raw(format!("{label}: ")),
            Span::styled(value.to_string(), Style::default().fg(color)),
        ])
    }
}
