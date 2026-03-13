mod color_scheme;
mod palette;
mod resolver;

pub(in crate::ui) use palette::ResolvedAppearance;
pub(in crate::ui) use resolver::{resolve_appearance, resolve_asset_theme, resolve_theme};
