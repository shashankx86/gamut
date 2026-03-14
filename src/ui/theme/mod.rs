mod color_scheme;
mod palette;
mod resolver;

pub(crate) use palette::ResolvedAppearance;
pub(crate) use resolver::{resolve_appearance, resolve_asset_theme, resolve_theme};
