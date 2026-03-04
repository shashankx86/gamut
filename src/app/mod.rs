use std::error::Error;

type DynError = Box<dyn Error + Send + Sync>;

pub fn run_daemon() -> Result<(), DynError> {
    Err("daemon runtime not implemented yet".into())
}
