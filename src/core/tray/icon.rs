use crate::core::assets::{tray_icon_svg, AssetTheme};
#[cfg(target_os = "linux")]
use resvg::tiny_skia::{Pixmap, Transform};
#[cfg(target_os = "linux")]
use resvg::usvg;
#[cfg(target_os = "linux")]
use std::io;
#[cfg(target_os = "linux")]
use tray_icon::Icon;

#[cfg(target_os = "linux")]
const TRAY_ICON_SIZE: u32 = 32;

#[cfg(target_os = "linux")]
pub(super) fn load(theme: AssetTheme) -> Result<Icon, Box<dyn std::error::Error>> {
    let tree = usvg::Tree::from_data(tray_icon_svg(theme), &usvg::Options::default())?;
    let svg_size = tree.size();
    let scale =
        (TRAY_ICON_SIZE as f32 / svg_size.width()).min(TRAY_ICON_SIZE as f32 / svg_size.height());
    let scaled_width = svg_size.width() * scale;
    let scaled_height = svg_size.height() * scale;
    let translate_x = (TRAY_ICON_SIZE as f32 - scaled_width) * 0.5;
    let translate_y = (TRAY_ICON_SIZE as f32 - scaled_height) * 0.5;

    let mut pixmap = Pixmap::new(TRAY_ICON_SIZE, TRAY_ICON_SIZE)
        .ok_or_else(|| io::Error::other("failed to allocate tray icon pixel buffer"))?;

    let transform = Transform::from_scale(scale, scale).post_translate(translate_x, translate_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    Ok(Icon::from_rgba(
        pixmap.data().to_vec(),
        pixmap.width(),
        pixmap.height(),
    )?)
}
