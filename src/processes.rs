
use std::cmp::Ordering;
use std::collections::HashMap;
use sysinfo::{Pid, Process, ProcessExt};

use crate::AppState;

pub fn get_process_vec(processes: &HashMap<Pid, Process>, app_state: &AppState) -> Vec<Vec<String>> {
    // let mut hash_vec: Vec<_> = processes.iter().filter(|n| !n.1.cpu_usage().is_nan()).collect();
    let mut hash_vec: Vec<_> = processes.iter().collect();
    if app_state.should_sort{
        hash_vec.sort_by(|a, b| {
            b.1.cpu_usage()
                .partial_cmp(&a.1.cpu_usage())
                .unwrap_or(Ordering::Equal)
        });
    }
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