pub struct AppConfig {
    pub delay: i32,
}

pub fn create_config_from_args(delay: i32) -> AppConfig {
    // TODO: read a config file

    // re-create an AppConfig with the argument values and config file values
    AppConfig {
        delay
    }
}
