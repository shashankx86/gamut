use iced::Color;
use iced::theme::Palette;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ThemePalette {
    pub palette: Palette,
    pub appearance: ResolvedAppearance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThemeScheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedAppearance {
    pub panel_background: Color,
    pub panel_border: Color,
    pub primary_text: Color,
    pub secondary_text: Color,
    pub muted_text: Color,
    pub divider: Color,
    pub search_icon: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub accent_strong: Color,
    pub first_row_active: Color,
    pub first_row_hover: Color,
    pub first_row_pressed: Color,
    pub row_hover: Color,
    pub row_pressed: Color,
    pub scrollbar_track: Color,
    pub scrollbar_track_border: Color,
    pub scrollbar_scroller: Color,
    pub scrollbar_scroller_border: Color,
    pub scrollbar_scroller_hover: Color,
    pub scrollbar_scroller_hover_border: Color,
    pub scrollbar_scroller_dragged: Color,
    pub scrollbar_scroller_dragged_border: Color,
}
