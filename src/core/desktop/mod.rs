mod discovery;
mod icons;
mod model;

pub use discovery::load_apps;
pub use discovery::resolve_icon_requests;
pub use model::{DesktopApp, IconResolveRequest, normalize_query, trim_label};
