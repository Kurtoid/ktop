use std::{
    fs::File,
    io::{BufRead, BufReader},
    usize,
};

pub struct vmstat_info {
    pub swap_in: usize,
    pub swap_out: usize,
    swap_in_last: usize,
    swap_out_last: usize,
}

impl vmstat_info {
    pub fn new() -> Self {
        Self {
            swap_in: 0,
            swap_out: 0,
            swap_in_last: 0,
            swap_out_last: 0,
        }
    }

    pub fn update(&mut self) {
        let file = File::open("/proc/vmstat").unwrap();
        let reader = BufReader::new(file);

        // Read the file line by line using the lines() iterator from std::io::BufRead.
        for (_index, line) in reader.lines().enumerate() {
            let line = line.unwrap(); // Ignore errors.
                                      // Show the line and its number.
                                      // please let there be a better way to do this
                                      // actually, lets TODO this for now

        }
    }
}

impl Default for vmstat_info {
    fn default() -> Self {
        Self::new()
    }
}
