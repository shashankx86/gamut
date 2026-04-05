use super::super::channel::{AppCommandReceiverHandle, IpcReceiverHandle};
use super::super::Message;
use super::Launcher;
use crate::core::desktop::{IconResolveRequest, refresh_app_cache, resolve_icon_requests, save_cached_apps};
use iced::Task;
use log::warn;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const ICON_RESOLVE_BATCH_SIZE: usize = 24;
const APP_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

impl Launcher {
    pub(in crate::ui::launcher) fn request_icon_resolution_for_visible(&mut self) -> Task<Message> {
        if !self.is_visible || self.icon_resolve_in_flight || self.calculation_preview().is_some() {
            return Task::none();
        }

        let requests: Vec<IconResolveRequest> = self
            .filtered_indices
            .iter()
            .copied()
            .filter_map(|index| {
                self.apps
                    .get(index)
                    .filter(|app| app.icon_path.is_none())
                    .map(|app| app.icon_request(index))
            })
            .take(ICON_RESOLVE_BATCH_SIZE)
            .collect();

        if requests.is_empty() {
            return Task::none();
        }

        self.icon_resolve_in_flight = true;
        Task::perform(
            async move { resolve_icon_requests(requests) },
            Message::IconsResolved,
        )
    }

    pub(in crate::ui::launcher) fn request_app_refresh(&mut self, force: bool) -> Task<Message> {
        if self.app_refresh_in_flight {
            return Task::none();
        }

        if !force
            && self
                .last_app_refresh_at
                .is_some_and(|at| at.elapsed() < APP_REFRESH_INTERVAL)
        {
            return Task::none();
        }

        self.app_refresh_in_flight = true;
        self.app_refresh_started_at = Some(Instant::now());
        Task::perform(async { refresh_app_cache() }, Message::AppsLoaded)
    }

    pub(in crate::ui::launcher) fn apply_resolved_icons(&mut self, updates: Vec<(usize, Option<PathBuf>)>) {
        let mut changed = false;

        for (index, icon_path) in updates {
            if let Some(path) = icon_path
                && let Some(app) = self.apps.get_mut(index)
                && app.icon_path.is_none()
            {
                app.icon_path = Some(path);
                changed = true;
            }
        }

        if changed && let Err(error) = save_cached_apps(&self.apps) {
            warn!("failed to persist resolved icon paths: {error}");
        }

        self.icon_resolve_in_flight = false;
    }

    pub(in crate::ui::launcher) fn finish_app_refresh(&mut self) {
        self.app_refresh_in_flight = false;
        self.app_refresh_started_at = None;
        self.last_app_refresh_at = Some(Instant::now());
    }

    pub(in crate::ui::launcher) fn ipc_handle(&self) -> IpcReceiverHandle {
        self.ipc_handle.clone()
    }

    pub(in crate::ui::launcher) fn app_command_handle(&self) -> AppCommandReceiverHandle {
        self.app_command_handle.clone()
    }
}
