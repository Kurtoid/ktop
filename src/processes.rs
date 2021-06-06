use crate::AppState;
use std::collections::HashMap;
use std::path::{self, Path};
use std::{cmp::Ordering, net::ToSocketAddrs};
use sysinfo::{Pid, Process, ProcessExt};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
pub fn get_process_vec<'a>(
    processes: &HashMap<i32, Process>,
    app_state: &AppState,
) -> Vec<Vec<Spans<'a>>> {
    // let mut hash_vec: Vec<_> = processes.iter().filter(|n| !n.1.cpu_usage().is_nan()).collect();
    let mut proc_vec: Vec<_> = processes.iter().collect();
    if app_state.should_sort {
        proc_vec.sort_by(|a, b| {
            b.1.cpu_usage()
                .partial_cmp(&a.1.cpu_usage())
                .unwrap_or(Ordering::Equal)
        });
    }
    let mut vec = Vec::new();
    for (pid, process) in proc_vec.iter() {
        // println!("[{}] {} {:?}", pid, process.name(), process.cpu_usage());
        let mut row = Vec::with_capacity(app_state.headers.len());
        for colum in &app_state.headers {
            row.push(match colum {
                crate::ColumnType::PID => {
                    Spans::from(Span::styled(pid.to_string(), Style::default()))
                }
                crate::ColumnType::NAME => {
                    Spans::from(pretty_cmd(process.name(), process.exe(), process.cmd()))
                }
                crate::ColumnType::CPU => Spans::from(Span::styled(
                    format!("{:.2}", process.cpu_usage()),
                    Style::default(),
                )),
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
    if cmd.len() <= 0 {
        // kernel thread or something
        return vec![Span::styled(format!("{}{}{}", "[", name, "]"), green)];
    }
    let cmd_args = match cmd.len() {
        0 => String::from(""),
        1 => String::from(""),
        _ => {
            let mut cmd_str = cmd[1..].join(" ");
            cmd_str.insert_str(0, " ");
            cmd_str
        }
    };
    let cmd_span = Span::styled(cmd_args, green);
    // how is it even possible that we have to split these twice???
    let first_frag = cmd[0]
        .splitn(1, " ")
        .next()
        .unwrap_or("unknown")
        .split(" ")
        .next();

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
