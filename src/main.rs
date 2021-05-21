use clap::{App, Arg};
mod config;
use std::cmp::Ordering;
use std::collections::HashMap;
use sysinfo::{Pid, Process, ProcessExt, System, SystemExt};
mod util;

use std::time::Duration;
use std::{error::Error, io};
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use util::event::{Config, Event, Events};
use util::StatefulTable;
mod debug_permissions;

fn get_process_vec(processes: &HashMap<Pid, Process>) -> Vec<Vec<String>> {
    // let mut hash_vec: Vec<_> = processes.iter().filter(|n| !n.1.cpu_usage().is_nan()).collect();
    let mut hash_vec: Vec<_> = processes.iter().collect();
    hash_vec.sort_by(|a, b| {
        b.1.cpu_usage()
            .partial_cmp(&a.1.cpu_usage())
            .unwrap_or(Ordering::Equal)
    });
    let mut vec = Vec::new();
    for (pid, process) in hash_vec.iter() {
        // println!("[{}] {} {:?}", pid, process.name(), process.cpu_usage());
        vec.push(vec![
            pid.to_string(),
            process.name().to_string(),
            process.cpu_usage().to_string(),
        ]);
    }
    vec
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("ktop")
        .version("0.1.0")
        .author("Kurt Wilson <kurt@kurtw.dev>")
        .about("A system monitor inspired by glances and htop")
        .arg(
            Arg::with_name("zswap")
            .short("z")
            .long("zswap")
            .help("read and display zswap debug stats")
        )
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
    let app_config = config::create_config_from_matches(matches);
    println!("Refresh time is {}", app_config.delay);

    // take care of the permissions first
    let can_use_debugfs = app_config.can_use_debugfs && match debug_permissions::can_read_debug() {
        false => {
            debug_permissions::set_debug_permissions();
            debug_permissions::can_read_debug()
        }
        true => true,
    };
    println!("using debugfs: {}", can_use_debugfs);

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new_all();
    let config = Config {
        tick_rate: Duration::from_millis(app_config.delay * 1000),
        ..Default::default()
    };
    let events = Events::with_config(config);
    let mut table = StatefulTable::new(vec![]);
    let processes = sys.get_processes();
    table.items = get_process_vec(processes);
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
                let cells = item.iter().map(|c| Cell::from(Span::raw(c)));
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
        match events.next()? {
            Event::Input(input) => match input {
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
            },
            Event::Tick => {
                sys.refresh_all();
                let processes = sys.get_processes();
                table.items = get_process_vec(processes);
            }
        }
    }

    Ok(())
}
