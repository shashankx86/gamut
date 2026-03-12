mod general;
mod model;
mod shortcuts;
mod tabs;
mod theme;
mod widgets;

use crate::core::ipc::{IpcCommand, send_command};
use crate::core::preferences::{AppPreferences, load_preferences, save_preferences};
use general::GeneralActions;
use model::{ThemeColorField, ThemeEditorState, update_theme_scheme_color};
use shortcuts::ShortcutEditor;
use tabs::PreferencesTab;

const WINDOW_WIDTH: f32 = 760.0;
const WINDOW_HEIGHT: f32 = 520.0;

struct PreferencesApp {
    preferences: AppPreferences,
    active_tab: PreferencesTab,
    theme_editor: ThemeEditorState,
    shortcut_editor: ShortcutEditor,
}

impl PreferencesApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let prefs = load_preferences();
        theme::apply_theme(&cc.egui_ctx, &prefs);
        let theme_editor = ThemeEditorState::from_preferences(&prefs);
        let editor = ShortcutEditor::from_shortcuts(&prefs.shortcuts);

        Self {
            preferences: prefs,
            active_tab: PreferencesTab::General,
            theme_editor,
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

    fn update_theme_color(
        &mut self,
        scheme: crate::core::preferences::ThemeSchemeId,
        field: ThemeColorField,
        value: String,
    ) {
        match update_theme_scheme_color(
            &mut self.preferences,
            &mut self.theme_editor,
            scheme,
            field,
            value,
        ) {
            Ok(()) => {
                self.theme_editor.set_theme_error(None);
                self.persist();
            }
            Err(error) => self.theme_editor.set_theme_error(Some(error)),
        }
    }
}

impl eframe::App for PreferencesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_theme(ctx, &self.preferences);
        let tokens = theme::tokens_from_preferences(&self.preferences);

        // Sidebar
        egui::SidePanel::left("preferences_sidebar")
            .resizable(false)
            .exact_width(140.0)
            .frame(
                egui::Frame::new()
                    .fill(tokens.base)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ctx, |ui| {
                tabs::render_sidebar(ui, &mut self.active_tab);
            });

        // Main content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(tokens.surface)
                    .inner_margin(egui::Margin {
                        left: 24,
                        right: 24,
                        top: 16,
                        bottom: 16,
                    })
                    .stroke(egui::Stroke::new(1.0, tokens.separator)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        match self.active_tab {
                            PreferencesTab::General => {
                                let GeneralActions {
                                    changed,
                                    theme_updates,
                                } = general::render_general(
                                    ui,
                                    &mut self.preferences,
                                    &self.theme_editor,
                                );

                                for (scheme, field, value) in theme_updates {
                                    self.update_theme_color(scheme, field, value);
                                }

                                if changed {
                                    self.persist();
                                }

                                if let Some(error) = self.theme_editor.theme_error() {
                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new(error)
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(255, 120, 100)),
                                    );
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
        if std::env::var_os("WINIT_UNIX_BACKEND").is_none() && std::env::var_os("DISPLAY").is_some()
        {
            unsafe {
                std::env::set_var("WINIT_UNIX_BACKEND", "x11");
            }
        }
    }
}
