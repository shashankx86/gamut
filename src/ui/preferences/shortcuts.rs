use egui::{Button, CornerRadius, Key, Modifiers, RichText, Stroke, StrokeKind, Ui, Vec2};

use crate::core::preferences::{ShortcutBinding, ShortcutPreferences};
use std::str::FromStr;

use super::model::{shortcut_preferences_from_values, ShortcutAction};
use super::theme;
use super::widgets::section_heading;

impl ShortcutAction {
    fn description(self) -> &'static str {
        self.helper_text()
    }

    fn get_binding(self, shortcuts: &ShortcutPreferences) -> &ShortcutBinding {
        match self {
            Self::LaunchSelected => &shortcuts.launch_selected,
            Self::ExpandOrMoveDown => &shortcuts.expand_or_move_down,
            Self::MoveUp => &shortcuts.move_up,
            Self::CloseLauncher => &shortcuts.close_launcher,
        }
    }
}

// ── Editor state for shortcut text inputs ──────────────────────────────────

#[derive(Clone)]
pub struct ShortcutEditor {
    buffers: [String; 4],
    error: Option<String>,
    capturing: Option<ShortcutAction>,
}

impl ShortcutEditor {
    pub fn from_shortcuts(shortcuts: &ShortcutPreferences) -> Self {
        Self {
            buffers: [
                shortcuts.launch_selected.to_string(),
                shortcuts.expand_or_move_down.to_string(),
                shortcuts.move_up.to_string(),
                shortcuts.close_launcher.to_string(),
            ],
            error: None,
            capturing: None,
        }
    }

    pub fn sync_from_shortcuts(&mut self, shortcuts: &ShortcutPreferences) {
        *self = Self::from_shortcuts(shortcuts);
    }

    fn buffer(&mut self, action: ShortcutAction) -> &mut String {
        &mut self.buffers[action as usize]
    }

    fn buffer_ref(&self, action: ShortcutAction) -> &str {
        &self.buffers[action as usize]
    }

    fn preview_binding(
        &self,
        action: ShortcutAction,
        shortcuts: &ShortcutPreferences,
    ) -> ShortcutBinding {
        ShortcutBinding::from_str(self.buffer_ref(action))
            .unwrap_or_else(|_| action.get_binding(shortcuts).clone())
    }

    fn is_capturing(&self, action: ShortcutAction) -> bool {
        self.capturing == Some(action)
    }

    fn toggle_capture(&mut self, action: ShortcutAction) {
        if self.is_capturing(action) {
            self.capturing = None;
        } else {
            self.capturing = Some(action);
            self.error = None;
        }
    }

    fn capture_from_input(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<Result<ShortcutPreferences, String>> {
        let action = self.capturing?;
        let captured = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } => binding_string_from_key(*key, *modifiers),
                _ => None,
            })
        });

        let captured = captured?;
        *self.buffer(action) = captured;
        self.capturing = None;
        Some(self.try_apply())
    }

    /// Attempt to parse and validate all buffers into a `ShortcutPreferences`.
    /// Returns `Ok(shortcuts)` on success or `Err(msg)` on validation failure.
    fn try_apply(&self) -> Result<ShortcutPreferences, String> {
        shortcut_preferences_from_values(&[
            (
                ShortcutAction::LaunchSelected,
                self.buffer_ref(ShortcutAction::LaunchSelected).to_string(),
            ),
            (
                ShortcutAction::ExpandOrMoveDown,
                self.buffer_ref(ShortcutAction::ExpandOrMoveDown)
                    .to_string(),
            ),
            (
                ShortcutAction::MoveUp,
                self.buffer_ref(ShortcutAction::MoveUp).to_string(),
            ),
            (
                ShortcutAction::CloseLauncher,
                self.buffer_ref(ShortcutAction::CloseLauncher).to_string(),
            ),
        ])
    }
}

// ── Render ─────────────────────────────────────────────────────────────────

/// Returns `true` if shortcuts were changed and need persisting.
pub fn render_shortcuts(
    ui: &mut Ui,
    shortcuts: &mut ShortcutPreferences,
    editor: &mut ShortcutEditor,
) -> bool {
    let mut changed = false;
    let tokens = theme::tokens(ui);

    if let Some(result) = editor.capture_from_input(ui.ctx()) {
        match result {
            Ok(new_shortcuts) => {
                *shortcuts = new_shortcuts;
                editor.error = None;
                changed = true;
            }
            Err(err) => {
                editor.error = Some(err);
            }
        }
    }

    section_heading(ui, "Keyboard Shortcuts");

    ui.add_space(4.0);
    ui.label(
        RichText::new("Click reassign, then press the shortcut you want to save.")
            .size(11.0)
            .color(tokens.text_secondary),
    );
    ui.add_space(6.0);

    // ── Shortcut rows ──────────────────────────────────────────────────
    for action in ShortcutAction::ALL {
        changed |= shortcut_edit_row(ui, action, shortcuts, editor);
        ui.add_space(2.0);
    }

    // ── Error feedback ─────────────────────────────────────────────────
    if let Some(err) = &editor.error {
        ui.add_space(4.0);
        let tokens = theme::tokens(ui);
        ui.label(
            RichText::new(format!("⚠ {err}"))
                .size(11.0)
                .color(tokens.accent),
        );
    }

    // ── Footer ─────────────────────────────────────────────────────────
    ui.add_space(12.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let btn = egui::Button::new(
            RichText::new("Restore Defaults")
                .size(11.0)
                .color(tokens.text_secondary),
        )
        .fill(tokens.surface)
        .stroke(Stroke::new(1.0, tokens.border))
        .corner_radius(4);

        if ui.add(btn).clicked() {
            *shortcuts = ShortcutPreferences::default();
            editor.sync_from_shortcuts(shortcuts);
            changed = true;
        }
    });

    changed
}

/// A single shortcut editing row. Returns `true` if the shortcut was changed.
fn shortcut_edit_row(
    ui: &mut Ui,
    action: ShortcutAction,
    shortcuts: &mut ShortcutPreferences,
    editor: &mut ShortcutEditor,
) -> bool {
    let current_binding = editor.preview_binding(action, shortcuts);
    let is_capturing = editor.is_capturing(action);
    let tokens = theme::tokens(ui);

    // Row container
    egui::Frame::new()
        .fill(if is_capturing {
            tokens.accent_dim
        } else {
            tokens.surface
        })
        .corner_radius(4)
        .stroke(if is_capturing {
            Stroke::new(1.0, tokens.accent)
        } else {
            Stroke::new(1.0, tokens.border)
        })
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // Top line: label + current key badges
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(action.label())
                        .size(12.5)
                        .color(tokens.text_primary)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    render_binding_badges(ui, &current_binding);
                });
            });

            // Description
            ui.label(
                RichText::new(action.description())
                    .size(10.5)
                    .color(tokens.muted),
            );

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                let button = Button::new(
                    RichText::new(if is_capturing {
                        "Cancel capture"
                    } else {
                        "Reassign"
                    })
                    .size(11.5)
                    .color(tokens.text_primary),
                )
                .fill(if is_capturing {
                    tokens.accent_dim
                } else {
                    tokens.surface_raised
                })
                .stroke(Stroke::new(
                    1.0,
                    if is_capturing {
                        tokens.accent
                    } else {
                        tokens.border
                    },
                ))
                .corner_radius(4)
                .min_size(Vec2::new(132.0, 28.0));

                if ui.add(button).clicked() {
                    editor.toggle_capture(action);
                }

                let hint = if is_capturing {
                    "Waiting for next key press..."
                } else {
                    "Captures the next key combo"
                };

                ui.label(RichText::new(hint).size(10.5).color(if is_capturing {
                    tokens.accent
                } else {
                    tokens.muted
                }));
            });

            if is_capturing {
                ui.add_space(3.0);
                ui.label(
                    RichText::new("Hold Ctrl / Alt / Shift, then press the final key.")
                        .size(10.0)
                        .color(tokens.text_secondary),
                );
            }
        });

    false
}

// ── Key badge rendering ────────────────────────────────────────────────────

fn render_binding_badges(ui: &mut Ui, binding: &ShortcutBinding) {
    // Render in reverse for right-to-left layout
    key_badge(ui, &binding.key);

    if binding.shift {
        key_badge(ui, "Shift");
    }
    if binding.alt {
        key_badge(ui, "Alt");
    }
    if binding.ctrl {
        key_badge(ui, "Ctrl");
    }
    if binding.super_key {
        key_badge(ui, "Super");
    }
}

fn key_badge(ui: &mut Ui, label: &str) {
    let tokens = theme::tokens(ui);
    let display = pretty_key(label);
    let galley = ui.fonts(|f| {
        f.layout_no_wrap(
            display.clone(),
            egui::FontId::proportional(10.0),
            tokens.text_primary,
        )
    });

    let padding = Vec2::new(5.0, 2.0);
    let desired = galley.size() + padding * 2.0;
    let min_width = 20.0_f32;
    let size = Vec2::new(desired.x.max(min_width), desired.y.max(18.0));

    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let rounding = CornerRadius::same(3);
        painter.rect_filled(rect, rounding, tokens.surface_raised);
        painter.rect_stroke(
            rect,
            rounding,
            Stroke::new(0.5, tokens.border),
            StrokeKind::Outside,
        );
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &display,
            egui::FontId::proportional(10.0),
            tokens.text_primary,
        );
    }
}

fn pretty_key(key: &str) -> String {
    match key.to_ascii_lowercase().as_str() {
        "arrowup" => "↑".to_string(),
        "arrowdown" => "↓".to_string(),
        "arrowleft" => "←".to_string(),
        "arrowright" => "→".to_string(),
        "escape" => "Esc".to_string(),
        "enter" | "return" => "Enter".to_string(),
        "space" => "Space".to_string(),
        other if other.len() == 1 => other.to_ascii_uppercase(),
        _ => {
            let mut chars = key.chars();
            match chars.next() {
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
                None => key.to_string(),
            }
        }
    }
}

fn binding_string_from_key(key: Key, modifiers: Modifiers) -> Option<String> {
    let key_name = key_name_for_capture(key)?;
    let mut parts = Vec::new();

    if modifiers.ctrl {
        parts.push("Ctrl".to_string());
    }
    if modifiers.alt {
        parts.push("Alt".to_string());
    }
    if modifiers.shift {
        parts.push("Shift".to_string());
    }
    if modifiers.mac_cmd {
        parts.push("Super".to_string());
    }

    parts.push(key_name.to_string());
    Some(parts.join("+"))
}

fn key_name_for_capture(key: Key) -> Option<&'static str> {
    match key {
        Key::ArrowDown => Some("ArrowDown"),
        Key::ArrowLeft => Some("ArrowLeft"),
        Key::ArrowRight => Some("ArrowRight"),
        Key::ArrowUp => Some("ArrowUp"),
        Key::Copy | Key::Cut | Key::Paste => None,
        _ => Some(key.name()),
    }
}
