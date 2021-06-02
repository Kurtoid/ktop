
use std::collections::hash_map::Values;
use std::path::Path;
use std::{cmp::Ordering, net::ToSocketAddrs};
use std::collections::HashMap;
use sysinfo::{Pid, Process, ProcessExt};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use crate::AppState;
pub fn get_process_vec<'a>(processes:  &'a Values<i32, Process>, app_state: &AppState) ->  Vec<Vec<Spans<'a>>> {
    let green = Style::default().fg(Color::Green);
    // let mut hash_vec: Vec<_> = processes.iter().filter(|n| !n.1.cpu_usage().is_nan()).collect();
    let mut hash_vec: Vec<_> = processes.collect();
    if app_state.should_sort{
        hash_vec.sort_by(|a, b| {
            b.cpu_usage()
                .partial_cmp(&a.cpu_usage())
                .unwrap_or(Ordering::Equal)
        });
    }
    let mut vec = Vec::new();
    for process in hash_vec.iter() {
        // println!("[{}] {} {:?}", pid, process.name(), process.cpu_usage());
        vec.push(vec![
            Spans::from(Span::styled(process.pid().to_string(), Style::default())),
            pretty_cmd(process.name().to_string(), process.cmd(), process.exe()),
            Spans::from(Span::styled(format!("{:.2}",process.cpu_usage()), Style::default())),
        ]);
    }
    vec
}

fn pretty_cmd<'a>(name: String, cmd: &'a [String], exe: &Path) -> Spans<'a> {
    // TODO: cleanup
    // TODO: configuable hide/show path
    let green = Style::default().fg(Color::Green);
    if cmd.len() <= 0 {
        // kernel thread or something
        return Spans::from(Span::styled(format!("{}{}{}", "[", name, "]"), green));
    }
    // how is it even possible that we have to split these twice???
    let first_frag = cmd[0].splitn(1, " ").next().unwrap_or("unknown").split(" ").next();

    // check if exec_name != proc_name - if so, append proc_name to string
    let file_name = exe.file_name();
    if file_name.is_none(){
        // dont worry about it
        // split
        return Spans::from(Span::from(first_frag.unwrap_or("unknown")));
    }
    let file_name = file_name.unwrap().to_string_lossy();
    if !file_name.eq(&name){
        return Spans::from(format!("{}|{}|", first_frag.unwrap_or("unknown"), name));
    }
    else{
        return Spans::from(first_frag.unwrap_or("unknown").to_string());
    }
}