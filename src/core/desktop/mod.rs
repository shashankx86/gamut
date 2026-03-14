mod cache;
mod discovery;
mod icons;
mod model;
pub(crate) mod search;

pub use cache::{load_cached_app_catalog, refresh_app_cache, save_cached_apps};
pub use discovery::resolve_icon_requests;
pub use model::{DesktopApp, IconResolveRequest, normalize_query, trim_label};
