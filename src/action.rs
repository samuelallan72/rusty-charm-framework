use serde_json::{self, Value};
use std::process::Command;

// TODO: drop the unwraps, figure out error handling/propogating
pub(crate) fn params() -> Value {
    let output = Command::new("action-get")
        .args(["--format", "json"])
        .output()
        .expect("failed to execute action-get");
    serde_json::from_slice::<Value>(&output.stdout).unwrap()
}

pub fn log(msg: &str) {
    let output = Command::new("action-log")
        .args([msg])
        .output()
        .expect("failed to execute action-log");
}
