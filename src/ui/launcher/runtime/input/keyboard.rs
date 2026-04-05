use super::super::super::{Launcher, Message};
use super::key_candidates::{matches_alt_action_key, pressed_key_candidates};
use iced::Task;
use iced::keyboard::{self, Key, Modifiers};

impl Launcher {
    pub(in crate::ui::launcher) fn handle_launcher_key_press(
        &mut self,
        key: Key,
        modifiers: Modifiers,
        physical_key: keyboard::key::Physical,
    ) -> Task<Message> {
        if let Some(task) = self.handle_alt_action_key_press(&key, modifiers, physical_key) {
            return task;
        }

        if self.matches_shortcut(
            &self.app_preferences.shortcuts.close_launcher,
            &key,
            modifiers,
            physical_key,
        ) {
            return self.hide_launcher();
        }

        if self.matches_shortcut(
            &self.app_preferences.shortcuts.expand,
            &key,
            modifiers,
            physical_key,
        ) && self.normalized_query.is_empty()
            && self.results_target == 0.0
        {
            return self.expand_results();
        }

        if self.matches_shortcut(
            &self.app_preferences.shortcuts.move_down,
            &key,
            modifiers,
            physical_key,
        ) {
            let previous_rank = self.selected_rank;
            self.bump_selection_revision();
            self.move_selection(1);
            self.reveal_results_scrollbar_for_keyboard_end(previous_rank);
            return self.scroll_to_selected(previous_rank, false);
        }

        if self.matches_shortcut(
            &self.app_preferences.shortcuts.move_up,
            &key,
            modifiers,
            physical_key,
        ) {
            let previous_rank = self.selected_rank;
            self.bump_selection_revision();
            self.move_selection(-1);
            return self.scroll_to_selected(previous_rank, false);
        }

        if self.matches_shortcut(
            &self.app_preferences.shortcuts.launch_selected,
            &key,
            modifiers,
            physical_key,
        ) {
            if let Some(index) = self.selected_result_index() {
                return self.launch(index);
            }
        }

        Task::none()
    }

    fn handle_alt_action_key_press(
        &mut self,
        key: &Key,
        modifiers: Modifiers,
        physical_key: keyboard::key::Physical,
    ) -> Option<Task<Message>> {
        if !modifiers.alt() || !self.should_show_action_overlay() {
            return None;
        }

        if matches_alt_action_key(key, physical_key, '1') {
            self.suppress_alt_actions_until_release();
            self.suppress_next_query_change();

            return Some(
                self.selected_application_index()
                    .map_or_else(Task::none, |index| self.launch(index)),
            );
        }

        if matches_alt_action_key(key, physical_key, '2') {
            self.suppress_alt_actions_until_release();
            self.suppress_next_query_change();

            return Some(
                self.selected_application_index()
                    .map_or_else(Task::none, |index| self.open_location(index)),
            );
        }

        None
    }

    fn matches_shortcut(
        &self,
        binding: &crate::core::preferences::ShortcutBinding,
        key: &Key,
        modifiers: Modifiers,
        physical_key: keyboard::key::Physical,
    ) -> bool {
        if binding.ctrl != modifiers.control()
            || binding.alt != modifiers.alt()
            || binding.shift != modifiers.shift()
            || binding.super_key != modifiers.logo()
        {
            return false;
        }

        let expected = binding.normalized_key();

        pressed_key_candidates(key, physical_key)
            .into_iter()
            .any(|pressed| pressed == expected)
    }
}
