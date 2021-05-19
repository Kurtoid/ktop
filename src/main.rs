use clap::{App, Arg};
const DEFAULT_DELAY: i32 = 5;
mod config;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::{thread, time};
use sysinfo::{ProcessExt, System, SystemExt};
mod util;

use util::event::{Event, Events};
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Terminal,
};
use util::{StatefulTable};

fn main() -> Result<(), Box<dyn Error>> {
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
        .arg(
            Arg::with_name("run once")
                .short("o")
                .short("once")
                .takes_value(false)
                .help("run once and exit"),
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
    let run_once = matches.is_present("run once");
    let app_config = config::create_config_from_args(delay_time, run_once);
    println!("Refresh time is {}", app_config.delay);

    let mut sys = System::new_all();
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut table = StatefulTable::new(vec![vec!["hi", "there", "text"]]);

    // Input
    loop {
        terminal.draw(|f| {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(2)
                .split(f.size());

            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let normal_style = Style::default().bg(Color::Blue);
            let header_cells = ["Header1", "Header2", "Header3"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
            let header = Row::new(header_cells)
                .style(normal_style)
                .height(1)
                .bottom_margin(1);
            let rows = table.items.iter().map(|item| {
                let height = item
                    .iter()
                    .map(|content| content.chars().filter(|c| *c == '\n').count())
                    .max()
                    .unwrap_or(0)
                    + 1;
                let cells = item.iter().map(|c| Cell::from(*c));
                Row::new(cells).height(height as u16).bottom_margin(1)
            });
            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Table"))
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Length(30),
                    Constraint::Max(10),
                ]);
            f.render_stateful_widget(t, rects[0], &mut table.state);
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') | Key::Esc => {
                    break;
                }
                Key::Down => {
                    table.next();
                }
                Key::Up => {
                    table.previous();
                }
                _ => {}
            }
        };
    }

    Ok(())
    /*
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
        .arg(
            Arg::with_name("run once")
                .short("o")
                .short("once")
                .takes_value(false)
                .help("run once and exit"),
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
    let run_once = matches.is_present("run once");
    let app_config = config::create_config_from_args(delay_time, run_once);
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
    loop {
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
        if app_config.run_once {
            break;
        };
    }
    */
}
