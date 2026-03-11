mod general;
mod shortcuts;
mod tabs;
mod theme;
mod widgets;

use crate::core::ipc::{IpcCommand, send_command};
use crate::core::preferences::{AppPreferences, load_preferences, save_preferences};
use shortcuts::ShortcutEditor;
use tabs::PreferencesTab;

const WINDOW_WIDTH: f32 = 760.0;
const WINDOW_HEIGHT: f32 = 520.0;

struct PreferencesApp {
    preferences: AppPreferences,
    active_tab: PreferencesTab,
    shortcut_editor: ShortcutEditor,
}

impl PreferencesApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);

        let prefs = load_preferences();
        let editor = ShortcutEditor::from_shortcuts(&prefs.shortcuts);

        Self {
            preferences: prefs,
            active_tab: PreferencesTab::General,
            shortcut_editor: editor,
        }
    }

    fn persist(&self) {
        if let Err(error) = save_preferences(&self.preferences) {
            eprintln!("failed to save preferences: {error}");
            return;
        }

        let _ = send_command(IpcCommand::ReloadPreferences);
    }
}

impl eframe::App for PreferencesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sidebar
        egui::SidePanel::left("preferences_sidebar")
            .resizable(false)
            .exact_width(140.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::BASE)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ctx, |ui| {
                tabs::render_sidebar(ui, &mut self.active_tab);
            });

        // Main content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::SURFACE)
                    .inner_margin(egui::Margin {
                        left: 24,
                        right: 24,
                        top: 16,
                        bottom: 16,
                    })
                    .stroke(egui::Stroke::new(1.0, theme::SEPARATOR)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        match self.active_tab {
                            PreferencesTab::General => {
                                if general::render_general(ui, &mut self.preferences) {
                                    self.persist();
                                }
                            }
                            PreferencesTab::Shortcuts => {
                                if shortcuts::render_shortcuts(
                                    ui,
                                    &mut self.preferences.shortcuts,
                                    &mut self.shortcut_editor,
                                ) {
                                    self.persist();
                                }
                            }
                        }
                    });
            });
    }
}

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    configure_linux_backend();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_title("Gamut Preferences")
            .with_decorations(true)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Gamut Preferences",
        options,
        Box::new(|cc| Ok(Box::new(PreferencesApp::new(cc)))),
    )
    .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
}

fn configure_linux_backend() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WINIT_UNIX_BACKEND").is_none()
            && std::env::var_os("DISPLAY").is_some()
        {
            unsafe {
                std::env::set_var("WINIT_UNIX_BACKEND", "x11");
            }
        }
    }
}
