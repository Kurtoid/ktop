use crate::AppState;
use crate::ColumnType;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use sysinfo::{Process, ProcessExt};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};

fn get_threads_from_process_map(
    processes: &HashMap<i32, Process>,
    use_threads: bool,
) -> Vec<&Process> {
    let mut all_threads = Vec::with_capacity(processes.len() * 2);
    for (_, process) in processes {
        all_threads.push(process);
        if use_threads {
            all_threads.append(&mut get_threads_from_process_map(&process.tasks, true));
        }
    }
    all_threads
}

pub fn get_process_vec<'a>(
    processes: &HashMap<i32, Process>,
    app_state: &AppState,
) -> Vec<Vec<Spans<'a>>> {
    let mut all_threads = get_threads_from_process_map(processes, app_state.show_threads);
    // there has got to be a better way to do this
    if let Some(sorting_key) = &app_state.sorting_by {
        match sorting_key {
            ColumnType::PID => {
                all_threads.sort_by(|a, b| a.pid().cmp(&b.pid()));
            }
            ColumnType::NAME => {
                all_threads.sort_by(|a, b| {
                    a.name()
                        .to_string()
                        .to_lowercase()
                        .cmp(&b.name().to_string().to_lowercase())
                });
            }
            ColumnType::CPU => {
                all_threads.sort_by(|a, b| {
                    b.cpu_usage()
                        .partial_cmp(&a.cpu_usage())
                        .unwrap_or(Ordering::Equal)
                });
            }
            ColumnType::RUNTIME => {
                all_threads.sort_by(|a, b| b.total_runtime().cmp(&a.total_runtime()));
            }
            ColumnType::MEMORY => {
                all_threads.sort_by(|a, b| b.memory().cmp(&a.memory()));
            }
        }
    }
    let mut vec = Vec::new();
    for process in all_threads.iter() {
        // println!("[{}] {} {:?}", pid, process.name(), process.cpu_usage());
        let mut row = Vec::with_capacity(app_state.headers.len());
        for colum in &app_state.headers {
            row.push(match colum {
                ColumnType::PID => {
                    Spans::from(Span::styled(process.pid().to_string(), Style::default()))
                }
                ColumnType::NAME => {
                    Spans::from(pretty_cmd(process.name(), process.exe(), process.cmd()))
                    // Spans::from(Span::styled(process.name().to_string(), Style::default()))
                }
                ColumnType::CPU => Spans::from(Span::styled(
                    format!("{:.2}", process.cpu_usage()),
                    Style::default(),
                )),
                ColumnType::RUNTIME => {
                    let process_runtime = process.total_runtime();
                    let seconds = process_runtime % 60;
                    let minutes = (process_runtime / 60) % 60;
                    let hours = (process_runtime / 60) / 60;
                    Spans::from(Span::styled(
                        format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
                        Style::default(),
                    ))
                }
                ColumnType::MEMORY => {
                    let bytes = process.memory() * 1000;
                    // TODO: just do this yourself - no need for another library here!!!
                    Spans::from(Span::styled(
                        bytefmt::format(bytes).replace("B", ""),
                        Style::default(),
                    ))
                }
            });
        }
        vec.push(row);
    }
    vec
}

fn pretty_cmd<'a>(name: &str, exe: &Path, cmd: &[String]) -> Vec<Span<'a>> {
    let green = Style::default().fg(Color::Green);
    let purple = Style::default().fg(Color::LightMagenta);
    let red = Style::default().fg(Color::Red);
    // TODO: cleanup
    // TODO: configuable hide/show path
    if cmd.is_empty() {
        // kernel thread or something
        return vec![Span::styled(format!("{}{}{}", "[", name, "]"), green)];
    }
    let cmd_args = match cmd.len() {
        0 => String::from(""),
        1 => String::from(""),
        _ => {
            let mut cmd_str = cmd[1..].join(" ");
            cmd_str.insert(0, ' ');
            cmd_str
        }
    };
    let cmd_span = Span::styled(cmd_args, green);
    // how is it even possible that we have to split these twice???
    let first_frag = cmd[0].splitn(1, ' ').next();
    // .unwrap_or("unknown")
    // .split(' ')
    // .next();

    // check if exec_name != proc_name - if so, append proc_name to string
    let file_name = exe.file_name();
    if file_name.is_none() {
        // dont worry about it
        // TODO: move/reorganize this
        return vec![
            Span::styled(
                first_frag.unwrap_or("unknown").to_string(),
                Style::default(),
            ),
            cmd_span,
        ];
    }
    let file_name = file_name.unwrap().to_string_lossy();
    if !file_name.eq(name) {
        let red_bar = Span::styled("|", red);
        return vec![
            Span::styled(
                first_frag.unwrap_or("unknown").to_string(),
                Style::default(),
            ),
            red_bar.clone(),
            Span::styled(name.to_string(), purple),
            red_bar,
            cmd_span,
        ];
    } else {
        return vec![
            Span::styled(
                first_frag.unwrap_or("unknown").to_string(),
                Style::default(),
            ),
            cmd_span,
        ];
    }
}
