use std::fs::File;
// TODO: allow custom debugfs location
pub fn can_read_debug() -> bool {
    return match File::open("/sys/kernel/debug") {
        Err(e) => {
            println!("{}", e);
            false
        },
        Ok(_) =>  true,
    }
}

// TODO: don't run a command like that (start a root-level process and use std::os::unix::fs::PermissionsExt ?)
pub fn set_debug_permissions()->bool{
    // TODO: print a warning or explaination as to whats happening
        ::std::process::Command::new("sudo")
            .arg("/usr/bin/chmod")
            .arg("755")
            .arg("/sys/kernel/debug")
            .status()
            .unwrap()
            .success()
}