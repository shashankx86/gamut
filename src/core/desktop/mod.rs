mod discovery;
mod icons;
mod model;

pub use discovery::load_apps;
pub use model::{DesktopApp, normalize_query, trim_label};
