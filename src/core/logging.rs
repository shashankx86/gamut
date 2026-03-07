use env_logger::{Builder, Env};

pub(crate) fn init() {
    let env = Env::default().default_filter_or("info");
    let _ = Builder::from_env(env).try_init();
}
