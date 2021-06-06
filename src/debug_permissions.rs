use std::fs::File;
use std::io::ErrorKind;
pub enum DebugfsStatus {
    NotMounted,
    NoPermissions,
    MountedAndReadable,
}

// TODO: allow custom debugfs location
pub fn can_read_debug() -> DebugfsStatus {
    match File::open("/sys/kernel/debug") {
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => DebugfsStatus::NoPermissions,
            ErrorKind::NotFound => DebugfsStatus::NotMounted,
            _ => DebugfsStatus::NotMounted
        },
        Ok(_) => DebugfsStatus::MountedAndReadable,
    }
}

// TODO: don't run a command like that (start a root-level process and use std::os::unix::fs::PermissionsExt ?)
pub fn set_debug_permissions() -> bool {
    // TODO: print a warning or explaination as to whats happening
    println!("Trying to get permissions for debugfs");
    ::std::process::Command::new("sudo")
        .arg("/usr/bin/chmod")
        .arg("755")
        .arg("/sys/kernel/debug")
        .status()
        .unwrap()
        .success()
}
