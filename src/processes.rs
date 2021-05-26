
use std::collections::btree_map::ValuesMut;
use std::{cmp::Ordering, net::ToSocketAddrs};
use std::collections::{BTreeMap, HashMap};
use psutil::process::{self, Process, ProcessError, processes};
use crate::AppState;

struct CachedProcess<'a>{
    process: &'a psutil::process::Process,
    cpu_percent: f32
}
pub fn get_process_vec(processes: ValuesMut<u32, Process>, app_state: &AppState) -> Vec<Vec<String>> {
    let mut cached_processes = Vec::new();
    for process_entry in processes{
        let cpu_usage = process_entry.cpu_percent().unwrap_or(0.0);
        cached_processes.push(CachedProcess{process: process_entry, cpu_percent: cpu_usage});
    }
    if app_state.should_sort{
        cached_processes.sort_by(|a, b| {
            b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(Ordering::Equal)
        });
    }

    let mut process_list = Vec::new();
    for process_entry in cached_processes.iter(){
        let mut process = process_entry.process;
        let cpu_usage = process_entry.cpu_percent ;
        let name = process.name().unwrap_or_else(|_error| -> String {"Unknown".to_string()});
        let pid = process.pid();
        process_list.push(vec![pid.to_string(), name, cpu_usage.to_string()]);

    }
    // if app_state.should_sort{
    //     processes.sort_by(|a,b| {
    //         b.unwrap().cpu_percent()                .partial_cmp(&a.unwrap().cpu_percent())
    //             .unwrap_or(Ordering::Equal)

    //     })
    // }
    process_list
}

// fn pretty_cmd(process: &Process) -> String {
//     // TODO: cleanup
//     // TODO: configuable hide/show path
//     let cmd = process.cmd();
//     if cmd.len() <= 0 {
//         // kernel thread or something
//         return format!("{}{}{}", "[", process.name(), "]").to_string();
//     }
//     // how is it even possible that we have to split these twice???
//     let first_frag = cmd[0].splitn(1, " ").next().unwrap_or("unknown").split(" ").next();

//     // check if exec_name != proc_name - if so, append proc_name to string
//     let file_name = process.exe().file_name();
//     if file_name.is_none(){
//         // dont worry about it
//         // TODO: move/reorganize this
//         return first_frag.unwrap_or("unknown").to_string();
//     }
//     let file_name = file_name.unwrap().to_string_lossy();
//     if !file_name.eq(process.name()){
//         return format!("{}|{}|", first_frag.unwrap_or("unknown"), process.name());
//     }
//     else{
//         return first_frag.unwrap_or("unknown").to_string();
//     }
// }