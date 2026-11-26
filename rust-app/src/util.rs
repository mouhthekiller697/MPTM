//! Tiny helpers shared by the platform-specific collectors.

use std::process::{Command, Stdio};

/// Run an external program and return its stdout when it exits successfully.
/// On Windows the console window is suppressed.
pub fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let mut c = Command::new(cmd);
    c.args(args).stdin(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = c.output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

/// Run a PowerShell snippet (Windows only helper).
pub fn powershell(script: &str) -> Option<String> {
    run("powershell", &["-NoProfile", "-NonInteractive", "-Command", script])
}

/// Spawn a detached program, ignoring failures. Returns true if it started.
pub fn launch(cmd: &str, args: &[&str]) -> bool {
    let mut c = Command::new(cmd);
    c.args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    c.spawn().is_ok()
}

pub fn read_trim(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}
