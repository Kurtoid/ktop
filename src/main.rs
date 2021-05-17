use clap::{App, Arg};
const DEFAULT_DELAY: i32 = 5;
mod config;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::{thread, time};
use sysinfo::{ProcessExt, System, SystemExt};
fn main() {
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let matches = App::new("ktop")
        .version("0.1.0")
        .author("Kurt Wilson <kurt@kurtw.dev>")
        .about("A system monitor inspired by glances and htop")
        .arg(
            Arg::with_name("config file")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("config file for ktop. not used yet"),
        )
        .arg(
            Arg::with_name("refresh time")
                .short("d")
                .long("refresh")
                .takes_value(true)
                .help("refresh time in seconds"),
        )
        .get_matches();

    // TODO: pass the config file location to confy
    // let myfile = matches.value_of("file").unwrap_or("input.txt");
    // println!("The file passed is: {}", myfile);

    let delay_str = matches.value_of("refresh time");
    let delay_time = match delay_str {
        None => DEFAULT_DELAY,
        Some(s) => match s.parse::<i32>() {
            Ok(n) => n,
            Err(_) => {
                println!("Invalid value passed to refresh time: {}", s);
                DEFAULT_DELAY
            }
        },
    };
    let app_config = config::create_config_from_args(delay_time);
    println!("Refresh time is {}", app_config.delay);

    let mut sys = System::new_all();

    // Components temperature:
    for component in sys.get_components() {
        println!("{:?}", component);
    }

    // Memory information:
    println!("total memory: {} KB", sys.get_total_memory());
    println!("used memory : {} KB", sys.get_used_memory());
    println!("total swap  : {} KB", sys.get_total_swap());
    println!("used swap   : {} KB", sys.get_used_swap());

    // Number of processors
    println!("NB processors: {}", sys.get_processors().len());

    // Display system information:
    println!("System name:             {:?}", sys.get_name());
    println!("System kernel version:   {:?}", sys.get_kernel_version());
    println!("System OS version:       {:?}", sys.get_os_version());
    println!("System host name:        {:?}", sys.get_host_name());

    let ten_millis =
        time::Duration::from_millis((app_config.delay * 1000).try_into().unwrap_or(5000));
    while true {
        thread::sleep(ten_millis);
        // To refresh all system information:
        sys.refresh_all();

        // We show the processes and some of their information:
        let processes = sys.get_processes();
        // let mut hash_vec: Vec<_> = processes.iter().filter(|n| !n.1.cpu_usage().is_nan()).collect();
        let mut hash_vec: Vec<_> = processes.iter().collect();

        hash_vec.sort_by(|a, b| {
            b.1.cpu_usage()
                .partial_cmp(&a.1.cpu_usage())
                .unwrap_or(Ordering::Equal)
        });
        for (pid, process) in hash_vec.iter().take(5) {
            println!("[{}] {} {:?}", pid, process.name(), process.cpu_usage());
        }
    }
}
