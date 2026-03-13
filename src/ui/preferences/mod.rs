mod general;
mod model;
mod shortcuts;
mod tabs;
mod theme;
mod widgets;

use crate::core::ipc::{send_command, IpcCommand};
use crate::core::preferences::{load_preferences, save_preferences, AppPreferences};
use general::GeneralActions;
use model::{update_theme_scheme_color, ThemeColorField, ThemeEditorState};
use shortcuts::ShortcutEditor;
use tabs::PreferencesTab;

const WINDOW_WIDTH: f32 = 780.0;
const WINDOW_HEIGHT: f32 = 560.0;

struct PreferencesApp {
    preferences: AppPreferences,
    active_tab: PreferencesTab,
    theme_editor: ThemeEditorState,
    shortcut_editor: ShortcutEditor,
}

impl PreferencesApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        register_lucide_font(&cc.egui_ctx);
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

    fn reset_to_defaults(&mut self) {
        self.preferences = AppPreferences::default();
        self.theme_editor = ThemeEditorState::from_preferences(&self.preferences);
        self.shortcut_editor = ShortcutEditor::from_shortcuts(&self.preferences.shortcuts);
        self.persist();
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
        let mut reset_requested = false;
        egui::SidePanel::left("preferences_sidebar")
            .resizable(false)
            .exact_width(150.0)
            .frame(
                egui::Frame::new()
                    .fill(tokens.base)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ctx, |ui| {
                if tabs::render_sidebar(ui, &mut self.active_tab) {
                    reset_requested = true;
                }
            });

        if reset_requested {
            self.reset_to_defaults();
        }

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
                                    let tokens = theme::tokens(ui);
                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new(error).size(11.0).color(tokens.accent),
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

fn register_lucide_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "lucide".to_owned(),
        egui::FontData::from_static(lucide_icons::LUCIDE_FONT_BYTES).into(),
    );
    fonts
        .families
        .entry(egui::FontFamily::Name("lucide".into()))
        .or_default()
        .push("lucide".to_owned());
    ctx.set_fonts(fonts);
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
