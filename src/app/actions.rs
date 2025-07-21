use crate::app::state::{AppState, PendingAction, Tab};
use crate::error::AppError;

impl AppState {
    /// Refresh all data from the broker
    pub async fn refresh_data(&mut self) -> Result<(), AppError> {
        let (workers_result, tasks_result, queues_result) = {
            let broker = self.broker.lock().await;

            // Fetch all data in parallel
            tokio::join!(
                broker.get_workers(),
                broker.get_tasks(),
                broker.get_queues()
            )
        };

        self.workers = workers_result?;
        self.tasks = tasks_result?;
        self.queues = queues_result?;

        // Validate selections after data refresh
        self.validate_selections();

        Ok(())
    }

    /// Execute the pending action (purge queue, retry task, or revoke task)
    pub async fn execute_pending_action(&mut self) -> Result<(), AppError> {
        if let Some(action) = self.pending_action.take() {
            let message = {
                let broker = self.broker.lock().await;

                match &action {
                    PendingAction::PurgeQueue(queue_name) => {
                        match broker.purge_queue(queue_name).await {
                            Ok(count) => {
                                format!("Purged {count} messages from queue '{queue_name}'")
                            }
                            Err(e) => format!("Failed to purge queue '{queue_name}': {e}"),
                        }
                    }
                    PendingAction::RetryTask(task_id) => match broker.retry_task(task_id).await {
                        Ok(_) => format!("Task '{task_id}' marked for retry"),
                        Err(e) => format!("Failed to retry task '{task_id}': {e}"),
                    },
                    PendingAction::RevokeTask(task_id) => match broker.revoke_task(task_id).await {
                        Ok(_) => format!("Task '{task_id}' revoked"),
                        Err(e) => format!("Failed to revoke task '{task_id}': {e}"),
                    },
                }
            };

            self.set_status_message(message);
        }

        self.hide_confirmation_dialog();
        Ok(())
    }

    /// Initiate queue purge action with confirmation dialog
    pub fn initiate_purge_queue(&mut self) {
        if !self.queues.is_empty() && self.selected_tab == Tab::Queues {
            let queue = &self.queues[self.selected_queue];
            let message = format!(
                "Are you sure you want to purge all {} messages from queue '{}'?",
                queue.length, queue.name
            );
            self.show_confirmation_dialog(message, PendingAction::PurgeQueue(queue.name.clone()));
        }
    }

    /// Initiate task retry action with confirmation dialog
    pub fn initiate_retry_task(&mut self) {
        if !self.tasks.is_empty() && self.selected_tab == Tab::Tasks {
            let filtered_tasks = self.get_filtered_tasks();
            if self.selected_task < filtered_tasks.len() {
                let task = filtered_tasks[self.selected_task];
                let message = format!("Are you sure you want to retry task '{}'?", task.id);
                self.show_confirmation_dialog(message, PendingAction::RetryTask(task.id.clone()));
            }
        }
    }

    /// Initiate task revoke action with confirmation dialog
    pub fn initiate_revoke_task(&mut self) {
        if !self.tasks.is_empty() && self.selected_tab == Tab::Tasks {
            let filtered_tasks = self.get_filtered_tasks();
            if self.selected_task < filtered_tasks.len() {
                let task = filtered_tasks[self.selected_task];
                let message = format!("Are you sure you want to revoke task '{}'?", task.id);
                self.show_confirmation_dialog(message, PendingAction::RevokeTask(task.id.clone()));
            }
        }
    }
}
