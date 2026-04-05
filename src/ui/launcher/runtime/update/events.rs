use super::super::super::{Launcher, Message};
use iced::Task;

impl Launcher {
    pub(in crate::ui::launcher) fn on_tick(&mut self) -> Task<Message> {
        self.tick_progress_indicator();
        let mut tasks = vec![self.animate_results()];

        if self.is_visible {
            tasks.push(self.request_app_refresh(false));
        }

        Task::batch(tasks)
    }
}
