use super::launcher::{
    Launcher, Message, ShortcutAction, ThemeColorField, PLACEMENT_OPTIONS, RADIUS_OPTIONS,
    SIZE_OPTIONS, THEME_OPTIONS,
};
use super::styles::{
    backdrop_style, bottom_strip_style, button_surface_style, divider_style, error_text_style,
    helper_text_style, panel_style, preferences_card_style, preferences_root_style,
    preferences_section_title_style, result_button_style, results_scroll_style, search_input_style,
};
use crate::core::desktop::{trim_label, DesktopApp};
use crate::core::preferences::{
    LauncherPlacement, LauncherSize, RadiusPreference, ThemePreference,
};
use iced::widget::svg::Handle as SvgHandle;
use iced::widget::{
    button, column, container, image, radio, row, scrollable, slider, svg, text, text_input, Id,
};
use iced::{window, Color, ContentFit, Element, Length};
use iced_shadcn::{
    icon_button, ButtonProps, ButtonRadius, ButtonSize, ButtonVariant, Palette as ShadcnPalette,
    Theme as ShadcnTheme,
};
use lucide_icons::iced::{icon_chevron_down, icon_corner_down_left, icon_search};

const BOTTOM_STRIP_ICON_BUTTON_SIZE: f32 = 20.0;
const BOTTOM_STRIP_ICON_SIZE: f32 = 12.0;

impl Launcher {
    pub(super) fn view(&self, window: window::Id) -> Element<'_, Message> {
        if self.is_preferences_window(window) {
            return self.view_preferences_window();
        }

        let appearance = self.resolved_appearance();

        if !self.is_visible {
            return container("")
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| backdrop_style())
                .into();
        }

        let mut content = column![self.view_search_header()]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Shrink);

        if self.results_progress() > 0.0 {
            content = content.push(self.view_results_section(self.results_progress()));
        }

        content = content
            .push(
                container("")
                    .height(1)
                    .width(Length::Fill)
                    .style(move |_| divider_style(&appearance)),
            )
            .push(self.view_bottom_strip());

        let launcher_panel = container(content)
            .padding(0)
            .style(move |_| panel_style(&self.layout, &appearance))
            .width(Length::Fill)
            .max_width(self.layout.panel_width as u32)
            .height(Length::Shrink);

        container(launcher_panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([
                self.layout.outer_padding_y as u16,
                self.layout.outer_padding_x as u16,
            ])
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Top)
            .style(|_| backdrop_style())
            .into()
    }

    fn view_search_header(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let input = text_input("Search for apps and commands...", &self.query)
            .id(self.input_id.clone())
            .on_input(Message::QueryChanged)
            .on_submit(Message::LaunchFirstMatch)
            .padding([self.layout.search_input_padding_y as u16, 0])
            .size(self.layout.search_input_font_size)
            .style(move |_theme, status| search_input_style(&appearance, status))
            .width(Length::Fill);

        row![icon_search().size(self.layout.search_icon_size), input,]
            .width(Length::Fill)
            .height(Length::Fixed(self.layout.search_row_height))
            .padding([0, self.layout.search_row_padding_x as u16])
            .spacing(self.layout.search_row_gap)
            .align_y(iced::alignment::Vertical::Center)
            .into()
    }

    fn view_results_section(&self, progress: f32) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let mut results = column![]
            .spacing(self.layout.result_row_gap)
            .width(Length::Fill);
        let filtered = self.filtered_indices();

        if filtered.is_empty() {
            results = results.push(
                container(
                    text("No applications found")
                        .size(self.layout.empty_state_text_size)
                        .color(Color::from_rgb(0.62, 0.64, 0.67)),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            );
        } else {
            let render_range = self.result_render_range();
            let row_step = self.layout.result_row_scroll_step();
            let visible_start = render_range.start;
            let visible_end = render_range.end;

            if visible_start > 0 {
                results = results.push(
                    container("")
                        .width(Length::Fill)
                        .height(Length::Fixed(self.results_top_spacer_height())),
                );
            }

            for (rank, index) in filtered
                .iter()
                .copied()
                .enumerate()
                .skip(visible_start)
                .take(visible_end.saturating_sub(visible_start))
            {
                results = results.push(self.view_result_row(index, rank == self.highlighted_rank));
            }

            let trailing_rows = filtered.len().saturating_sub(visible_end);
            if trailing_rows > 0 {
                results = results.push(container("").width(Length::Fill).height(Length::Fixed(
                    super::launcher::spacer_height_for_rows(trailing_rows, row_step),
                )));
            }
        }

        let list = scrollable(results)
            .id(self.results_scroll_id.clone())
            .height(Length::Fill)
            .on_scroll(Message::ResultsScrolled)
            .direction(iced::widget::scrollable::Direction::Vertical(
                iced::widget::scrollable::Scrollbar::new()
                    .width(10)
                    .scroller_width(6)
                    .spacing(4),
            ))
            .style(move |theme, status| results_scroll_style(theme, &appearance, status));

        container(list)
            .width(Length::Fill)
            .height(Length::Fixed(self.layout.results_height * progress))
            .padding([
                self.layout.results_top_bottom_padding as u16,
                self.layout.results_side_padding as u16,
            ])
            .into()
    }

    fn view_result_row(&self, index: usize, selected: bool) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let app = &self.apps[index];

        let left = row![
            container(self.view_app_icon(app, self.layout.result_icon_size))
                .width(Length::Fixed(self.layout.result_icon_box_size))
                .height(Length::Fixed(self.layout.result_icon_box_size))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .padding([0, 3]),
            column![
                text(&app.name)
                    .size(self.layout.result_primary_text_size)
                    .color(appearance.primary_text),
                text(trim_label(&app.exec_line, self.layout.result_label_max_len))
                    .size(self.layout.result_secondary_text_size)
                    .color(appearance.secondary_text),
            ]
            .spacing(1)
            .width(Length::Fill),
        ]
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        button(left)
            .on_press(Message::LaunchIndex(index))
            .padding([0, 10])
            .width(Length::Fill)
            .height(Length::Fixed(self.layout.result_row_height))
            .style(move |_theme, status| {
                result_button_style(status, selected, &self.layout, &appearance)
            })
            .into()
    }

    fn view_bottom_strip(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let shadcn_theme = bottom_strip_shadcn_theme(&appearance);
        let (label_text, icon_button) = if self.query.is_empty() && self.results_target == 0.0 {
            (
                "Show more",
                action_icon_button(
                    icon_chevron_down().size(BOTTOM_STRIP_ICON_SIZE),
                    &shadcn_theme,
                ),
            )
        } else {
            (
                "Open",
                action_icon_button(
                    icon_corner_down_left().size(BOTTOM_STRIP_ICON_SIZE),
                    &shadcn_theme,
                ),
            )
        };

        let logo = container(
            svg(SvgHandle::from_memory(include_bytes!(
                "../../assets/icons/gamut-full-transparent-dark.svg"
            )))
            .width(Length::Fixed(self.layout.logo_width))
            .height(Length::Fixed(self.layout.logo_height)),
        )
        .padding([0, 4])
        .center_y(Length::Fill);

        let show_more = container(
            row![
                text(label_text)
                    .size(self.layout.result_primary_text_size)
                    .color(appearance.muted_text),
                icon_button,
            ]
            .align_y(iced::alignment::Vertical::Center)
            .spacing(self.layout.bottom_strip_label_gap),
        )
        .height(Length::Fill)
        .center_y(Length::Fill);

        container(
            row![logo, container("").width(Length::Fill), show_more,]
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(iced::alignment::Vertical::Center)
                .spacing(0),
        )
        .height(Length::Fixed(self.layout.bottom_strip_height))
        .padding([0, self.layout.bottom_strip_padding_x as u16])
        .style(move |_| bottom_strip_style(&appearance))
        .into()
    }

    fn view_app_icon(&self, app: &DesktopApp, size: f32) -> Element<'_, Message> {
        if let Some(path) = app.icon_path.as_deref() {
            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase());

            if matches!(extension.as_deref(), Some("svg")) {
                return svg(path.to_path_buf())
                    .width(Length::Fixed(size))
                    .height(Length::Fixed(size))
                    .into();
            }

            return image(path.to_path_buf())
                .width(Length::Fixed(size))
                .height(Length::Fixed(size))
                .content_fit(ContentFit::Contain)
                .into();
        }

        let fallback = app
            .name
            .chars()
            .find(|ch| ch.is_alphanumeric())
            .unwrap_or('A')
            .to_string();

        text(fallback)
            .size(12)
            .color(self.resolved_appearance().primary_text)
            .into()
    }

    fn result_render_range(&self) -> std::ops::Range<usize> {
        let filtered = self.filtered_indices();

        if filtered.is_empty() {
            return 0..0;
        }

        if super::launcher::is_manual_expansion_in_progress(
            self.normalized_query.is_empty(),
            self.manually_expanded,
            self.results_progress,
            self.results_target,
        ) {
            return super::launcher::expansion_render_range(
                self.scroll_start_rank,
                filtered.len(),
                self.layout.visible_result_rows(),
            );
        }

        0..filtered.len()
    }

    fn results_top_spacer_height(&self) -> f32 {
        super::launcher::spacer_height_for_rows(
            self.result_render_range().start,
            self.layout.result_row_scroll_step(),
        )
    }

    fn view_preferences_window(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();

        let content = column![
            self.preferences_header(),
            self.preferences_appearance_section(),
            self.preferences_layout_section(),
            self.preferences_shortcuts_section(),
            self.preferences_feedback_section(),
        ]
        .spacing(18)
        .padding(24)
        .width(Length::Fill);

        container(
            scrollable(content)
                .id(self.preferences_scroll_id.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |theme, status| results_scroll_style(theme, &appearance, status)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| preferences_root_style(&appearance))
        .into()
    }

    fn preferences_header(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let close = button(text("Close").size(14))
            .on_press(Message::PreferencesCloseRequested)
            .style(move |_theme, status| button_surface_style(&appearance, status));

        container(
            row![
                column![
                    text("Preferences").size(26).color(appearance.primary_text),
                    text("Adjust appearance, layout, and shortcuts. Changes apply immediately.")
                        .size(14)
                        .color(appearance.secondary_text),
                ]
                .spacing(6)
                .width(Length::Fill),
                close,
            ]
            .align_y(iced::alignment::Vertical::Center)
            .spacing(12),
        )
        .style(move |_| preferences_card_style(&appearance))
        .padding(20)
        .into()
    }

    fn preferences_appearance_section(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let prefs = &self.app_preferences.appearance;

        let theme_radios = THEME_OPTIONS.iter().copied().fold(
            column![].spacing(10),
            |column: iced::widget::Column<'_, Message>, option| {
                column.push(radio(
                    theme_label(option),
                    option,
                    Some(prefs.theme),
                    Message::PreferencesThemeSelected,
                ))
            },
        );

        let radius_radios = RADIUS_OPTIONS.iter().copied().fold(
            column![].spacing(10),
            |column: iced::widget::Column<'_, Message>, option| {
                column.push(radio(
                    radius_label(option),
                    option,
                    Some(prefs.radius),
                    Message::PreferencesRadiusSelected,
                ))
            },
        );

        let custom_colors = column![
            self.preferences_text_field(
                "Background color",
                "#151516",
                self.preferences_editor
                    .theme_value(ThemeColorField::Background),
                self.custom_background_input_id.clone(),
                move |value| Message::PreferencesThemeColorChanged(
                    ThemeColorField::Background,
                    value
                ),
            ),
            self.preferences_text_field(
                "Text color",
                "#EBEDF2",
                self.preferences_editor.theme_value(ThemeColorField::Text),
                self.custom_text_input_id.clone(),
                move |value| Message::PreferencesThemeColorChanged(ThemeColorField::Text, value),
            ),
            self.preferences_text_field(
                "Accent color",
                "#5E8BFF",
                self.preferences_editor.theme_value(ThemeColorField::Accent),
                self.custom_accent_input_id.clone(),
                move |value| Message::PreferencesThemeColorChanged(ThemeColorField::Accent, value),
            ),
        ]
        .spacing(12);

        let radius_slider = column![
            text(format!("Custom radius: {:.0}px", prefs.custom_radius))
                .size(14)
                .color(appearance.primary_text),
            slider(
                0.0..=36.0,
                prefs.custom_radius,
                Message::PreferencesCustomRadiusChanged
            ),
        ]
        .spacing(8);

        self.preferences_card(
            "Appearance",
            "Theme, custom colors, and surface rounding.",
            column![theme_radios, custom_colors, radius_radios, radius_slider].spacing(16),
        )
    }

    fn preferences_layout_section(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let prefs = &self.app_preferences.layout;

        let size_radios = SIZE_OPTIONS.iter().copied().fold(
            column![].spacing(10),
            |column: iced::widget::Column<'_, Message>, option| {
                column.push(radio(
                    size_label(option),
                    option,
                    Some(prefs.size),
                    Message::PreferencesSizeSelected,
                ))
            },
        );

        let placement_radios = PLACEMENT_OPTIONS.iter().copied().fold(
            column![].spacing(10),
            |column: iced::widget::Column<'_, Message>, option| {
                column.push(radio(
                    placement_label(option),
                    option,
                    Some(prefs.placement),
                    Message::PreferencesPlacementSelected,
                ))
            },
        );

        let top_margin_slider = column![
            text(format!(
                "Custom top offset: {:.0}px",
                prefs.custom_top_margin
            ))
            .size(14)
            .color(appearance.primary_text),
            slider(
                0.0..=320.0,
                prefs.custom_top_margin,
                Message::PreferencesCustomTopMarginChanged
            ),
        ]
        .spacing(8);

        self.preferences_card(
            "Layout",
            "Control launcher scale and on-screen position.",
            column![size_radios, placement_radios, top_margin_slider].spacing(16),
        )
    }

    fn preferences_shortcuts_section(&self) -> Element<'_, Message> {
        let rows = ShortcutAction::ALL.iter().copied().fold(
            column![].spacing(14),
            |column: iced::widget::Column<'_, Message>, action| {
                column.push(self.shortcut_row(action))
            },
        );

        self.preferences_card(
            "Shortcuts",
            "These are the launcher shortcuts currently handled inside the app.",
            rows,
        )
    }

    fn preferences_feedback_section(&self) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let mut content = column![].spacing(8);
        let mut has_feedback = false;

        if let Some(message) = self.preferences_editor.theme_error() {
            has_feedback = true;
            content = content.push(
                text(message)
                    .size(13)
                    .style(move |_theme| error_text_style(&appearance)),
            );
        }

        if let Some(message) = self.preferences_editor.shortcut_error() {
            has_feedback = true;
            content = content.push(
                text(message)
                    .size(13)
                    .style(move |_theme| error_text_style(&appearance)),
            );
        }

        if let Some(message) = self.preferences_editor.save_error() {
            has_feedback = true;
            content = content.push(
                text(message)
                    .size(13)
                    .style(move |_theme| error_text_style(&appearance)),
            );
        }

        if !has_feedback {
            content = content.push(
                text("Preferences are saved to disk as soon as you change them.")
                    .size(13)
                    .style(move |_theme| helper_text_style(&appearance)),
            );
        }

        container(content).into()
    }

    fn preferences_card<'a>(
        &self,
        title: &'a str,
        subtitle: &'a str,
        body: iced::widget::Column<'a, Message>,
    ) -> Element<'a, Message> {
        let appearance = self.resolved_appearance();

        container(
            column![
                text(title)
                    .size(18)
                    .style(move |_theme| preferences_section_title_style(&appearance)),
                text(subtitle)
                    .size(13)
                    .style(move |_theme| helper_text_style(&appearance)),
                body,
            ]
            .spacing(12),
        )
        .padding(20)
        .style(move |_| preferences_card_style(&appearance))
        .into()
    }

    fn preferences_text_field<'a, F>(
        &self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        id: Id,
        on_input: F,
    ) -> Element<'a, Message>
    where
        F: 'static + Fn(String) -> Message + Clone,
    {
        let appearance = self.resolved_appearance();
        container(
            column![
                text(label)
                    .size(14)
                    .style(move |_theme| preferences_section_title_style(&appearance)),
                text_input(placeholder, value)
                    .id(id)
                    .on_input(on_input)
                    .padding([10, 12])
                    .size(14)
                    .style(move |_theme, status| search_input_style(&appearance, status)),
            ]
            .spacing(6),
        )
        .into()
    }

    fn shortcut_row(&self, action: ShortcutAction) -> Element<'_, Message> {
        let appearance = self.resolved_appearance();
        let input_id = match action {
            ShortcutAction::LaunchSelected => self.shortcut_launch_input_id.clone(),
            ShortcutAction::ExpandOrMoveDown => self.shortcut_down_input_id.clone(),
            ShortcutAction::MoveUp => self.shortcut_up_input_id.clone(),
            ShortcutAction::CloseLauncher => self.shortcut_close_input_id.clone(),
        };

        let capture = button(text("Capture modifiers + Enter").size(12))
            .on_press(Message::PreferencesCaptureShortcut(action))
            .style(move |_theme, status| button_surface_style(&appearance, status));

        container(
            column![
                text(action.label())
                    .size(14)
                    .style(move |_theme| preferences_section_title_style(&appearance)),
                text(action.helper_text())
                    .size(12)
                    .style(move |_theme| helper_text_style(&appearance)),
                row![
                    text_input(
                        "Ctrl+Shift+K",
                        self.preferences_editor.shortcut_value(action)
                    )
                    .id(input_id)
                    .on_input(move |value| Message::PreferencesShortcutChanged(action, value))
                    .padding([10, 12])
                    .size(14)
                    .style(move |_theme, status| search_input_style(&appearance, status))
                    .width(Length::Fill),
                    capture,
                ]
                .spacing(10)
                .align_y(iced::alignment::Vertical::Center),
            ]
            .spacing(6),
        )
        .into()
    }
}

fn bottom_strip_shadcn_theme(appearance: &super::theme::ResolvedAppearance) -> ShadcnTheme {
    ShadcnTheme::with_palette(ShadcnPalette {
        background: appearance.panel_background,
        foreground: appearance.muted_text,
        card: appearance.panel_background,
        card_foreground: appearance.primary_text,
        popover: appearance.panel_background,
        popover_foreground: appearance.primary_text,
        border: appearance.panel_border,
        input: appearance.panel_border,
        ring: appearance.selection,
        primary: appearance.primary_text,
        primary_foreground: appearance.panel_background,
        secondary: appearance.first_row_active,
        secondary_foreground: appearance.primary_text,
        accent: appearance.first_row_hover,
        accent_foreground: appearance.primary_text,
        muted: appearance.first_row_active,
        muted_foreground: appearance.muted_text,
        destructive: appearance.scrollbar_scroller_dragged_border,
        destructive_foreground: appearance.panel_background,
        chart_1: appearance.selection,
        chart_2: appearance.first_row_hover,
        chart_3: appearance.first_row_pressed,
        chart_4: appearance.row_hover,
        chart_5: appearance.row_pressed,
        sidebar: appearance.panel_background,
        sidebar_foreground: appearance.primary_text,
        sidebar_primary: appearance.primary_text,
        sidebar_primary_foreground: appearance.panel_background,
        sidebar_accent: appearance.first_row_hover,
        sidebar_accent_foreground: appearance.primary_text,
        sidebar_border: appearance.panel_border,
        sidebar_ring: appearance.selection,
    })
}

fn action_icon_button<'a>(
    icon: impl Into<Element<'a, Message>>,
    theme: &ShadcnTheme,
) -> iced::widget::button::Button<'a, Message> {
    icon_button(
        icon,
        Some(Message::LaunchFirstMatch),
        ButtonProps::new()
            .variant(ButtonVariant::Outline)
            .radius(ButtonRadius::Small)
            .size(ButtonSize::Size1),
        theme,
    )
    .width(Length::Fixed(BOTTOM_STRIP_ICON_BUTTON_SIZE))
    .height(Length::Fixed(BOTTOM_STRIP_ICON_BUTTON_SIZE))
}

fn theme_label(value: ThemePreference) -> &'static str {
    match value {
        ThemePreference::Dark => "Dark",
        ThemePreference::Light => "Light",
        ThemePreference::System => "System",
        ThemePreference::Custom => "Custom",
    }
}

fn radius_label(value: RadiusPreference) -> &'static str {
    match value {
        RadiusPreference::Small => "Small",
        RadiusPreference::Medium => "Medium",
        RadiusPreference::Large => "Large",
        RadiusPreference::Custom => "Custom",
    }
}

fn size_label(value: LauncherSize) -> &'static str {
    match value {
        LauncherSize::Small => "Small",
        LauncherSize::Medium => "Medium",
        LauncherSize::Large => "Large",
        LauncherSize::ExtraLarge => "Extra large",
    }
}

fn placement_label(value: LauncherPlacement) -> &'static str {
    match value {
        LauncherPlacement::RaisedCenter => "Raised center",
        LauncherPlacement::Center => "Center",
        LauncherPlacement::Custom => "Custom",
    }
}
