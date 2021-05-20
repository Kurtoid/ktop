pub struct AppConfig {
    pub delay: u64,
    pub run_once: bool,
}

pub fn create_config_from_args(delay: u64, run_once: bool) -> AppConfig {
    // TODO: read a config file

    // re-create an AppConfig with the argument values and config file values
    AppConfig { delay, run_once }
}
