
use std::{cmp::Ordering, net::ToSocketAddrs};
use std::collections::HashMap;
use psutil::process::{Process, ProcessError, processes};
use crate::AppState;

pub fn get_process_vec(processes: Result<Vec<Result<Process, ProcessError>>, psutil::Error>, app_state: &AppState) -> Vec<Vec<String>> {
    // let mut hash_vec: Vec<_> = processes.iter().filter(|n| !n.1.cpu_usage().is_nan()).collect();
    // unpack then unwrap

    // if we use unwrap_or on the next block, we don't need this, right?
    if !processes.is_ok(){
        return vec![];
    }
    let mut process_list = Vec::new();
    let processes = processes.unwrap_or(vec![]);
    for process in processes{
        if !process.is_ok(){
            continue;
        }
        let mut process = process.unwrap();
        let cpu_usage =  process.cpu_percent().unwrap_or(0.0);
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