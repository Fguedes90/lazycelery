pub mod events;
pub mod layout;
pub mod modals;
pub mod widgets;

use ratatui::Frame;

use crate::app::{App, Tab};
use crate::ui::layout::{create_main_layout, draw_header, draw_status_bar};
use crate::ui::modals::{draw_confirmation_dialog, draw_help, draw_task_details_modal};
use crate::ui::widgets::{QueueWidget, TaskWidget, Widget, WorkerWidget};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = create_main_layout(f.size());

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

    // Draw task details modal if active
    if app.show_task_details {
        draw_task_details_modal(f, app);
    }
}
