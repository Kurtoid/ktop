use clap::{App, Arg};
mod config;
use sysinfo::{System, SystemExt};
mod util;
mod processes;

use std::{time::Duration, vec};
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

use crate::debug_permissions::DebugfsStatus;
mod debug_permissions;

pub struct AppState<'a>{
    should_sort: bool,
    can_use_debugfs: bool,
    headers: Vec<& 'a str>,
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
                .help("read and display zswap debug stats"),
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
                .long("once")
                .takes_value(false)
                .help("run once and exit"),
        )
        .get_matches();

    // TODO: pass the config file location to confy
    // let myfile = matches.value_of("file").unwrap_or("input.txt");
    // println!("The file passed is: {}", myfile);
    let app_config = config::create_config_from_matches(matches);
    // take care of the permissions first
    let can_use_debugfs = app_config.can_use_debugfs
        && match debug_permissions::can_read_debug() {
            DebugfsStatus::NoPermissions => {
                debug_permissions::set_debug_permissions();
                match debug_permissions::can_read_debug(){
                    DebugfsStatus::MountedAndReadable=> true,
                    _ => false
                }
            }
            DebugfsStatus::MountedAndReadable => true,
            DebugfsStatus::NotMounted => {
                println!("Debugfs not found at TODO!");
                false
            }
        };
    let mut app_state = AppState{should_sort: true, can_use_debugfs, headers: vec!["pid", "name", "CPU%"] };

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new_all();
    sys.refresh_all();
    let config = Config {
        tick_rate: Duration::from_millis(app_config.delay * 1000),
        ..Default::default()
    };
    let events = Events::with_config(config);
    let mut table = StatefulTable::new(vec![]);
    // Input
    loop {
        terminal.draw(|f| {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(0)
                .split(f.size());

            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let normal_style = Style::default().bg(Color::Blue);
            let header_cells = app_state.headers
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
            let header = Row::new(header_cells)
                .style(normal_style)
                .height(1)
                .bottom_margin(0);
            let rows = table.items.iter().map(|item| {
                let cells = item.iter().map(|this_span| Cell::from(this_span.clone()));
                Row::new(cells).height(1).bottom_margin(0)
            });
            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Processes"))
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Length(8),
                    Constraint::Percentage(60),
                    Constraint::Max(10),
                ]);
            f.render_stateful_widget(t, rects[0], &mut table.state);
        })?;
        if app_config.run_once {break}; // TODO: don't clear screen
        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Down => {
                    table.next();
                }
                Key::Up => {
                    table.previous();
                }
                Key::Esc =>{
                    app_state.should_sort = false;
                }
                _ => {}
            },
            Event::Tick => {
                // only refresh what we use
                // sys.refresh_all();
                sys.refresh_cpu();
                sys.refresh_processes();
                let processes = sys.get_processes();
                table.items = processes::get_process_vec(processes, &app_state);
            }
        }
    }

    Ok(())
}
