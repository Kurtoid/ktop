
use std::path::{self, Path};
use std::{cmp::Ordering, net::ToSocketAddrs};
use std::collections::HashMap;
use sysinfo::{Pid, Process, ProcessExt};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use crate::AppState;
pub fn get_process_vec<'a>(processes: &HashMap<i32, Process>, app_state: &AppState) ->  Vec<Vec<Spans<'a>>> {
    // let mut hash_vec: Vec<_> = processes.iter().filter(|n| !n.1.cpu_usage().is_nan()).collect();
    let mut proc_vec: Vec<_> = processes.iter().collect();
    if app_state.should_sort{
        proc_vec.sort_by(|a, b| {
            b.1.cpu_usage()
                .partial_cmp(&a.1.cpu_usage())
                .unwrap_or(Ordering::Equal)
        });
    }
    let mut vec = Vec::new();
    for (pid, process) in proc_vec.iter() {
        // println!("[{}] {} {:?}", pid, process.name(), process.cpu_usage());
        vec.push(vec![
            Spans::from(Span::styled(pid.to_string(), Style::default())),
            Spans::from(pretty_cmd(process.name(), process.exe(), process.cmd())),
            Spans::from(Span::styled(format!("{:.2}",process.cpu_usage()), Style::default())),
        ]);
    }
    vec
}

fn pretty_cmd<'a>(name: &str, exe: &Path, cmd: &[String]) -> Span<'a> {
    let green = Style::default().fg(Color::Green);
    // TODO: cleanup
    // TODO: configuable hide/show path
    if cmd.len() <= 0 {
        // kernel thread or something
        return Span::styled(format!("{}{}{}", "[", name, "]"),green);
    }
    // how is it even possible that we have to split these twice???
    let first_frag = cmd[0].splitn(1, " ").next().unwrap_or("unknown").split(" ").next();

    // check if exec_name != proc_name - if so, append proc_name to string
    let file_name = exe.file_name();
    if file_name.is_none(){
        // dont worry about it
        // TODO: move/reorganize this
        return Span::styled(first_frag.unwrap_or("unknown").to_string(), green);
    }
    let file_name = file_name.unwrap().to_string_lossy();
    if !file_name.eq(name){
        return Span::styled(format!("{}|{}|", first_frag.unwrap_or("unknown"), name).to_string(), green);
    }
    else{
        return Span::styled(first_frag.unwrap_or("unknown").to_string(), green);
    }
}