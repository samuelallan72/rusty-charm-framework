use std::fmt::{Display, Formatter};
use std::process::Command;

enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO",
                LogLevel::Warning => "WARNING",
                LogLevel::Error => "ERROR",
            }
        )
    }
}

// TODO: drop the '.expect', figure out error handling/propogating
fn log(msg: &str, level: LogLevel) {
    Command::new("juju-log")
        .args(["--log-level", level.to_string().as_str(), msg])
        .output()
        .expect("failed to execute juju-log");
}

pub fn debug(msg: &str) {
    log(msg, LogLevel::Debug)
}

pub fn info(msg: &str) {
    log(msg, LogLevel::Info)
}
