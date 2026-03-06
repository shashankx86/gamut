use super::TrayService;

pub(super) fn start() -> Result<TrayService, Box<dyn std::error::Error>> {
    Ok(TrayService::detached())
}
