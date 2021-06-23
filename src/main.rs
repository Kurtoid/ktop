use clap::{App, Arg};
mod config;
use sysinfo::{ProcessorExt, System, SystemExt};
use vmstat::vmstat_info;
mod processes;
mod util;

use std::{error::Error, io};
use std::{time::Duration, vec};
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use util::event::{Config, Event, Events};
use util::StatefulTable;

use crate::debug_permissions::DebugfsStatus;
use crate::meter_widget::MeterWidget;
use crate::zswap::read_zswap_stats;
mod debug_permissions;
mod meter_widget;
mod vmstat;
mod zswap;

pub struct AppState {
    sorting_by: Option<ColumnType>,
    sorting_column_index: usize,
    can_use_debugfs: bool,
    headers: Vec<ColumnType>,
    vminfo: vmstat_info,
    show_threads: bool,
}
#[derive(PartialEq, Clone, Copy)]
enum ColumnType {
    PID,
    NAME,
    CPU,
    RUNTIME,
    MEMORY,
    MEMORY_SWAP,
}

impl ColumnType {
    fn value(&self) -> &str {
        match *self {
            ColumnType::PID => "PID",
            ColumnType::NAME => "NAME",
            ColumnType::RUNTIME => "TIME",
            ColumnType::CPU => "CPU%",
            ColumnType::MEMORY => "MEMORY",
            ColumnType::MEMORY_SWAP => "SWAP",
        }
    }
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
        ).arg(
            Arg::with_name("show threads")
                .short("t")
                .long("show-threads")
                .takes_value(false) // TODO: take values here
                .help("show threads. Enabled by default. overrides hide-threads")
        ).arg(
            Arg::with_name("hide threads")
                .long("hide-threads") // i'd rather make 'show-threads' a boolean, but this seems to follow conventions
                .takes_value(false)
                .help("hide threads. implies accumulate-parent - thread values will be added to parent process")
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
                matches!(
                    debug_permissions::can_read_debug(),
                    DebugfsStatus::MountedAndReadable
                )
            }
            DebugfsStatus::MountedAndReadable => true,
            DebugfsStatus::NotMounted => {
                println!("Debugfs not found at TODO!");
                false
            }
        };
    let mut app_state = AppState {
        can_use_debugfs,
        headers: vec![
            ColumnType::PID,
            ColumnType::RUNTIME,
            ColumnType::CPU,
            ColumnType::MEMORY,
            ColumnType::MEMORY_SWAP,
            ColumnType::NAME,
        ],
        sorting_by: Some(ColumnType::CPU),
        sorting_column_index: 2,
        vminfo: vmstat_info::new(),
        show_threads: app_config.show_threads,
    };

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
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .margin(0)
                .split(f.size());

            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let normal_style = Style::default().bg(Color::Blue);

            // main process table
            let header_cells = app_state.headers.iter().map(|h| {
                let color = if let Some(sorting_key) = &app_state.sorting_by {
                    if h == sorting_key {
                        Color::Green
                    } else {
                        Color::Red
                    }
                } else {
                    Color::Red
                };
                Cell::from(h.value()).style(Style::default().fg(color))
            });
            let header = Row::new(header_cells)
                .style(normal_style)
                .height(1)
                .bottom_margin(0);
            let rows = table.items.iter().map(|item| {
                let cells = item.iter().map(|this_span| Cell::from(this_span.clone()));
                Row::new(cells).height(1).bottom_margin(0)
            });
            let title = match app_state.show_threads {
                true => "Processes and threads",
                false => "Processes",
            };
            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    // TODO: make this part of appconfig headers
                    Constraint::Length(8),
                    Constraint::Length(9),
                    Constraint::Length(7),
                    Constraint::Length(8),
                    Constraint::Length(8),
                    Constraint::Min(20),
                ]);
            let meter = MeterWidget {
                cpu_percent: sys.get_global_processor_info().get_cpu_usage() / 100f32,
                cpu_system_percent: sys.get_global_processor_info().get_system_percent() / 100f32,
                memory_percent: sys.get_used_memory() as f32 / sys.get_total_memory() as f32,
                memory_used: sys.get_used_memory(),
                swap_percent: sys.get_used_swap() as f32 / sys.get_total_swap() as f32,
                total_swap: sys.get_used_swap(),
                zswap_stats: match app_state.can_use_debugfs {
                    true => match read_zswap_stats() {
                        Ok(r) => Some(r),
                        Err(_) => None,
                    },
                    false => None,
                },
                swap_in: app_state.vminfo.swap_in,
                swap_out: app_state.vminfo.swap_out,
            };
            f.render_stateful_widget(t, rects[1], &mut table.state);
            f.render_widget(meter, rects[0]);
        })?;
        if app_config.run_once {
            break;
        }; // TODO: don't clear screen
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
                Key::Esc => {
                    if table.state.selected().is_some() {
                        table.unselect();
                    } else if app_state.sorting_by.is_some() {
                        app_state.sorting_by = None;
                        refresh_all(&mut sys, &mut table, &mut app_state);
                    }
                }
                Key::Right => {
                    if app_state.sorting_column_index + 1 >= app_state.headers.len() {
                        app_state.sorting_column_index = 0;
                    } else {
                        app_state.sorting_column_index += 1;
                    }
                    app_state.sorting_by = Some(app_state.headers[app_state.sorting_column_index]);
                    refresh_all(&mut sys, &mut table, &mut app_state);
                }
                Key::Left => {
                    if app_state.sorting_column_index == 0 {
                        app_state.sorting_column_index = app_state.headers.len() - 1;
                    } else {
                        app_state.sorting_column_index -= 1;
                    }
                    app_state.sorting_by = Some(app_state.headers[app_state.sorting_column_index]);
                    refresh_all(&mut sys, &mut table, &mut app_state);
                }
                Key::Char('t') => {
                    // show/hide threads
                    app_state.show_threads = !app_state.show_threads;
                    refresh_all(&mut sys, &mut table, &mut app_state);
                }
                _ => {}
            },
            Event::Tick => {
                // only refresh what we use
                // sys.refresh_all();
                refresh_all(&mut sys, &mut table, &mut app_state);
            }
        }
    }

    Ok(())
}

fn refresh_all(sys: &mut System, mut table: &mut StatefulTable<'_>, app_state: &mut AppState) {
    sys.refresh_cpu();
    sys.refresh_processes();
    sys.refresh_memory();
    app_state.vminfo.update();
    let processes = sys.get_processes();
    table.items = processes::get_process_vec(processes, &app_state);
    if let Some(index) = table.state.selected() {
        if index >= table.items.len() - 1 {
            table.state.select(Some(table.items.len() - 1));
        }
    }
}
