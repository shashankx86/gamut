use crate::core::ipc::{IpcCommand, send_command};
use crate::core::preferences::{
    AppPreferences, LauncherPlacement, LauncherSize, RadiusPreference, ShortcutPreferences,
    ThemePreference, load_preferences, save_preferences,
};
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;

const PREFERENCES_CSS: &str = include_str!("../../assets/preferences.css");
const INITIAL_HEAD: &str = r#"<style>
html, body, #main {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
  background: #141414;
  color: #f4f4f5;
}

body {
  color-scheme: dark;
}
</style>"#;
const WINDOW_WIDTH: f64 = 760.0;
const WINDOW_HEIGHT: f64 = 520.0;
#[derive(Clone, PartialEq)]
struct PreferencesState {
    active_tab: PreferencesTab,
    preferences: AppPreferences,
    shortcut_search: String,
}

impl PreferencesState {
    fn load() -> Self {
        Self {
            active_tab: PreferencesTab::General,
            preferences: load_preferences(),
            shortcut_search: String::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PreferencesTab {
    General,
    Shortcuts,
}

#[derive(Clone, Copy)]
struct PrototypeShortcut {
    label: &'static str,
    keys: &'static [&'static str],
}

const PROTOTYPE_SHORTCUTS: [PrototypeShortcut; 6] = [
    PrototypeShortcut {
        label: "Raycast Hotkey",
        keys: &["⌘", "Space"],
    },
    PrototypeShortcut {
        label: "Window Options",
        keys: &["⌘", ","],
    },
    PrototypeShortcut {
        label: "Clipboard History",
        keys: &["⌥", "⌘", "C"],
    },
    PrototypeShortcut {
        label: "File Search",
        keys: &["⌥", "⌘", "F"],
    },
    PrototypeShortcut {
        label: "Confetti",
        keys: &["⌃", "⌘", "C"],
    },
    PrototypeShortcut {
        label: "Calculator",
        keys: &["⌥", "C"],
    },
];

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    configure_linux_preferences_backend();

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_background_color((20, 20, 20, 255))
                .with_custom_head(INITIAL_HEAD.to_string())
                .with_menu(None)
                .with_window(
                    WindowBuilder::new()
                        .with_title("Gamut Preferences")
                        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                        .with_background_color((20, 20, 20, 255))
                        .with_decorations(true)
                        .with_resizable(true),
                ),
        )
        .launch(PreferencesApp);

    Ok(())
}

#[allow(non_snake_case)]
fn PreferencesApp() -> Element {
    let window = dioxus::desktop::use_window();
    let mut state = use_signal(PreferencesState::load);
    let defaults = AppPreferences::default();
    let current = state();

    use_effect(move || {
        window.set_focus();
    });

    let shortcuts_query = current.shortcut_search.trim().to_ascii_lowercase();
    let shortcuts: Vec<PrototypeShortcut> = PROTOTYPE_SHORTCUTS
        .into_iter()
        .filter(|shortcut| {
            shortcuts_query.is_empty()
                || shortcut
                    .label
                    .to_ascii_lowercase()
                    .contains(shortcuts_query.as_str())
        })
        .collect();

    let general_active = current.active_tab == PreferencesTab::General;
    let shortcuts_active = current.active_tab == PreferencesTab::Shortcuts;
    let theme_value = theme_label(current.preferences.appearance.theme);
    let radius_value = radius_label(current.preferences.appearance.radius);
    let size_value = size_label(current.preferences.layout.size);
    let location_value = placement_label(current.preferences.layout.placement);
    let start_at_login = current.preferences.system.start_at_login;
    let search_value = current.shortcut_search.clone();

    rsx! {
        document::Title { "Gamut Preferences" }
        document::Style { "{PREFERENCES_CSS}" }

        div {
            class: "preferences-backdrop",
            div {
                class: "preferences-window",

                div {
                    class: "preferences-toolbar",

                    button {
                        class: "nav-item",
                        "data-active": if general_active { "true" } else { "false" },
                        onclick: move |_| {
                            let mut next = state();
                            next.active_tab = PreferencesTab::General;
                            state.set(next);
                        },
                        div {
                            class: "nav-icon-wrap",
                            SettingsIcon {
                                class: if general_active { "nav-icon nav-icon-active" } else { "nav-icon" }
                            }
                        }
                        span { class: "nav-label", "General" }
                    }

                    button {
                        class: "nav-item",
                        "data-active": if shortcuts_active { "true" } else { "false" },
                        onclick: move |_| {
                            let mut next = state();
                            next.active_tab = PreferencesTab::Shortcuts;
                            state.set(next);
                        },
                        div {
                            class: "nav-icon-wrap",
                            KeyboardIcon {
                                class: if shortcuts_active { "nav-icon nav-icon-active" } else { "nav-icon" }
                            }
                        }
                        span { class: "nav-label", "Shortcuts" }
                    }
                }

                div {
                    class: "preferences-content",

                    div {
                        class: "preferences-content-inner",
                        "data-tab": if general_active { "general" } else { "shortcuts" },

                        if general_active {
                            div {
                                class: "settings-stack",

                                div {
                                    class: "settings-section",
                                    div { class: "settings-section-title", "Appearance" }
                                    div {
                                        class: "grouped-list",

                                        div {
                                            class: "grouped-row",
                                            div {
                                                class: "row-label-wrap",
                                                span { class: "row-label", "Theme" }
                                                if theme_value != theme_label(defaults.appearance.theme) {
                                                    button {
                                                        class: "reset-button",
                                                        title: "Reset to default",
                                                        onclick: move |event| {
                                                            event.stop_propagation();
                                                            let mut next = state();
                                                            next.preferences.appearance.theme = defaults.appearance.theme;
                                                            persist_preferences(&next.preferences);
                                                            state.set(next);
                                                        },
                                                        RotateIcon {}
                                                    }
                                                }
                                            }
                                            div {
                                                class: "row-control",
                                                SelectControl {
                                                    value: theme_value,
                                                    options: &["Light", "Dark", "System"],
                                                    onchange: move |value: String| {
                                                        let mut next = state();
                                                        next.preferences.appearance.theme = parse_theme(&value);
                                                        persist_preferences(&next.preferences);
                                                        state.set(next);
                                                    }
                                                }
                                            }
                                        }

                                        div {
                                            class: "grouped-row",
                                            div {
                                                class: "row-label-wrap",
                                                span { class: "row-label", "Window Radius" }
                                                if radius_value != radius_label(defaults.appearance.radius) {
                                                    button {
                                                        class: "reset-button",
                                                        title: "Reset to default",
                                                        onclick: move |event| {
                                                            event.stop_propagation();
                                                            let mut next = state();
                                                            next.preferences.appearance.radius = defaults.appearance.radius;
                                                            persist_preferences(&next.preferences);
                                                            state.set(next);
                                                        },
                                                        RotateIcon {}
                                                    }
                                                }
                                            }
                                            div {
                                                class: "row-control",
                                                SelectControl {
                                                    value: radius_value,
                                                    options: &["Small", "Medium", "Large"],
                                                    onchange: move |value: String| {
                                                        let mut next = state();
                                                        next.preferences.appearance.radius = parse_radius(&value);
                                                        persist_preferences(&next.preferences);
                                                        state.set(next);
                                                    }
                                                }
                                            }
                                        }

                                        div {
                                            class: "grouped-row",
                                            div {
                                                class: "row-label-wrap",
                                                span { class: "row-label", "Window Size" }
                                                if size_value != size_label(defaults.layout.size) {
                                                    button {
                                                        class: "reset-button",
                                                        title: "Reset to default",
                                                        onclick: move |event| {
                                                            event.stop_propagation();
                                                            let mut next = state();
                                                            next.preferences.layout.size = defaults.layout.size;
                                                            persist_preferences(&next.preferences);
                                                            state.set(next);
                                                        },
                                                        RotateIcon {}
                                                    }
                                                }
                                            }
                                            div {
                                                class: "row-control",
                                                SelectControl {
                                                    value: size_value,
                                                    options: &["Small", "Medium", "Large", "Extra Large"],
                                                    onchange: move |value: String| {
                                                        let mut next = state();
                                                        next.preferences.layout.size = parse_size(&value);
                                                        persist_preferences(&next.preferences);
                                                        state.set(next);
                                                    }
                                                }
                                            }
                                        }

                                        div {
                                            class: "grouped-row",
                                            div {
                                                class: "row-label-wrap",
                                                span { class: "row-label", "Window Location" }
                                                if location_value != placement_label(defaults.layout.placement) {
                                                    button {
                                                        class: "reset-button",
                                                        title: "Reset to default",
                                                        onclick: move |event| {
                                                            event.stop_propagation();
                                                            let mut next = state();
                                                            next.preferences.layout.placement = defaults.layout.placement;
                                                            persist_preferences(&next.preferences);
                                                            state.set(next);
                                                        },
                                                        RotateIcon {}
                                                    }
                                                }
                                            }
                                            div {
                                                class: "row-control",
                                                SelectControl {
                                                    value: location_value,
                                                    options: &["Center", "Above Center"],
                                                    onchange: move |value: String| {
                                                        let mut next = state();
                                                        next.preferences.layout.placement = parse_placement(&value);
                                                        persist_preferences(&next.preferences);
                                                        state.set(next);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div {
                                    class: "settings-section",
                                    div { class: "settings-section-title", "System" }
                                    div {
                                        class: "grouped-list",
                                        div {
                                            class: "grouped-row",
                                            div {
                                                class: "row-label-wrap",
                                                span { class: "row-label", "Start at Login" }
                                                if start_at_login != defaults.system.start_at_login {
                                                    button {
                                                        class: "reset-button",
                                                        title: "Reset to default",
                                                        onclick: move |event| {
                                                            event.stop_propagation();
                                                            let mut next = state();
                                                            next.preferences.system.start_at_login = defaults.system.start_at_login;
                                                            persist_preferences(&next.preferences);
                                                            state.set(next);
                                                        },
                                                        RotateIcon {}
                                                    }
                                                }
                                            }
                                            div {
                                                class: "row-control row-control-switch",
                                                label {
                                                    class: "switch-shell",
                                                    input {
                                                        class: "switch-input",
                                                        r#type: "checkbox",
                                                        checked: start_at_login,
                                                        onchange: move |event| {
                                                            let mut next = state();
                                                            next.preferences.system.start_at_login = event.checked();
                                                            persist_preferences(&next.preferences);
                                                            state.set(next);
                                                        }
                                                    }
                                                    span {
                                                        class: "switch-track",
                                                        span { class: "switch-thumb" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            div {
                                class: "shortcuts-panel",

                                div {
                                    class: "shortcut-search",
                                    SearchIcon { class: "shortcut-search-icon" }
                                    input {
                                        class: "shortcut-search-input",
                                        value: search_value,
                                        placeholder: "Search shortcuts...",
                                        oninput: move |event| {
                                            let mut next = state();
                                            next.shortcut_search = event.value();
                                            state.set(next);
                                        }
                                    }
                                }

                                div {
                                    class: "shortcut-list-shell",
                                    div {
                                        class: "shortcut-list",
                                        for shortcut in shortcuts {
                                            div {
                                                key: shortcut.label,
                                                class: "shortcut-row",
                                                span { class: "shortcut-label", {shortcut.label} }
                                                div {
                                                    class: "shortcut-keys",
                                                    for &key_name in shortcut.keys {
                                                        div {
                                                            key: key_name,
                                                            class: "shortcut-key",
                                                            {key_name}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div {
                                    class: "shortcuts-footer",
                                    button {
                                        class: "restore-button",
                                        onclick: move |_| {
                                            let mut next = state();
                                            next.shortcut_search.clear();
                                            next.preferences.shortcuts = ShortcutPreferences::default();
                                            persist_preferences(&next.preferences);
                                            state.set(next);
                                        },
                                        "Restore Defaults"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SelectControl(
    value: String,
    options: &'static [&'static str],
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "select-shell",
            select {
                class: "value-select",
                value: value,
                onchange: move |event| onchange.call(event.value()),
                for &option in options {
                    option {
                        key: option,
                        value: option,
                        {option}
                    }
                }
            }
            ChevronDownIcon {}
        }
    }
}

#[component]
fn SettingsIcon(class: String) -> Element {
    rsx! {
        svg {
            class: class,
            width: "18",
            height: "18",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51h.01a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" }
        }
    }
}

#[component]
fn KeyboardIcon(class: String) -> Element {
    rsx! {
        svg {
            class: class,
            width: "18",
            height: "18",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "2", y: "6", width: "20", height: "12", rx: "2" }
            path { d: "M6 10h.01" }
            path { d: "M10 10h.01" }
            path { d: "M14 10h.01" }
            path { d: "M18 10h.01" }
            path { d: "M8 14h8" }
        }
    }
}

#[component]
fn SearchIcon(class: String) -> Element {
    rsx! {
        svg {
            class: class,
            width: "14",
            height: "14",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.25",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "11", cy: "11", r: "7" }
            path { d: "m21 21-4.35-4.35" }
        }
    }
}

#[component]
fn RotateIcon() -> Element {
    rsx! {
        svg {
            width: "12",
            height: "12",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 2v6h-6" }
            path { d: "M3 12a9 9 0 0 1 15-6.7L21 8" }
            path { d: "M3 22v-6h6" }
            path { d: "M21 12a9 9 0 0 1-15 6.7L3 16" }
        }
    }
}

#[component]
fn ChevronDownIcon() -> Element {
    rsx! {
        svg {
            class: "select-chevron",
            width: "14",
            height: "14",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m6 9 6 6 6-6" }
        }
    }
}

fn theme_label(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::Light => "Light",
        ThemePreference::Dark => "Dark",
        ThemePreference::System | ThemePreference::Custom => "System",
    }
}

fn radius_label(radius: RadiusPreference) -> &'static str {
    match radius {
        RadiusPreference::Small => "Small",
        RadiusPreference::Medium | RadiusPreference::Custom => "Medium",
        RadiusPreference::Large => "Large",
    }
}

fn size_label(size: LauncherSize) -> &'static str {
    match size {
        LauncherSize::Small => "Small",
        LauncherSize::Medium => "Medium",
        LauncherSize::Large => "Large",
        LauncherSize::ExtraLarge => "Extra Large",
    }
}

fn placement_label(placement: LauncherPlacement) -> &'static str {
    match placement {
        LauncherPlacement::Center => "Center",
        LauncherPlacement::RaisedCenter | LauncherPlacement::Custom => "Above Center",
    }
}

fn parse_theme(value: &str) -> ThemePreference {
    match value {
        "Light" => ThemePreference::Light,
        "Dark" => ThemePreference::Dark,
        _ => ThemePreference::System,
    }
}

fn parse_radius(value: &str) -> RadiusPreference {
    match value {
        "Small" => RadiusPreference::Small,
        "Large" => RadiusPreference::Large,
        _ => RadiusPreference::Medium,
    }
}

fn parse_size(value: &str) -> LauncherSize {
    match value {
        "Small" => LauncherSize::Small,
        "Large" => LauncherSize::Large,
        "Extra Large" => LauncherSize::ExtraLarge,
        _ => LauncherSize::Medium,
    }
}

fn parse_placement(value: &str) -> LauncherPlacement {
    match value {
        "Center" => LauncherPlacement::Center,
        _ => LauncherPlacement::RaisedCenter,
    }
}

fn persist_preferences(preferences: &AppPreferences) {
    if let Err(error) = save_preferences(preferences) {
        eprintln!("failed to save preferences: {error}");
        return;
    }

    let _ = send_command(IpcCommand::ReloadPreferences);
}

fn configure_linux_preferences_backend() {
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
