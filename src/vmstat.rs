use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::exit,
    usize,
};

use regex::Regex;

pub struct vmstat_info {
    pub swap_in: usize,
    pub swap_out: usize,
    pub swap_in_last: usize,
    pub swap_out_last: usize,
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
        let re = Regex::new(r"([^ ]+) ([0-9]+)").unwrap();

        // Read the file line by line using the lines() iterator from std::io::BufRead.
        for (_index, line) in reader.lines().enumerate() {
            let line = line.unwrap(); // Ignore errors.
            let mut captures = re.captures(&line);
            if let Some(captures) = captures {
                if captures.len() != 3 {
                    continue;
                }
                let field = &captures[1];
                let parsed_value = captures[2].parse();
                if parsed_value.is_ok() {
                    let value = parsed_value.ok();
                    if let Some(value) = value {
                        match field {
                            "pswpin" => {
                                self.swap_in = value - self.swap_in_last;
                                self.swap_in_last = value;
                            }
                            "pswpout" => {
                                self.swap_out = value - self.swap_out_last;
                                self.swap_out_last = value;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

impl Default for vmstat_info {
    fn default() -> Self {
        Self::new()
    }
}
