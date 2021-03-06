use clap::ArgMatches;
const DEFAULT_DELAY: u64 = 5;
pub struct AppConfig {
    pub delay: u64,
    pub run_once: bool,
    pub can_use_debugfs: bool,
    pub show_threads: bool
}

pub fn create_config_from_matches(matches: ArgMatches)-> AppConfig{

    let delay_str = matches.value_of("refresh time");
    let delay_time = match delay_str {
        None => DEFAULT_DELAY,
        Some(s) => match s.parse::<u64>() {
            Ok(n) => n,
            Err(_) => {
                println!("Invalid value passed to refresh time: {}", s);
                std::process::exit(-1)
            }
        },
    };
    let run_once = matches.is_present("run once");
    let can_use_debugfs = matches.is_present("zswap") ;//|| cfg!(debug_assertions); // always use zswap when debugging
    let hide_threads = matches.is_present("hide threads");

    // allow show_threads to override hide_threads
    let show_threads = !hide_threads || matches.is_present("show threads");
    AppConfig { delay: delay_time, run_once, can_use_debugfs, show_threads }
}