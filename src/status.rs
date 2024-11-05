use std::fmt::{Display, Formatter};
use std::process::Command;

// Error is also a status, but not one that can be directly set.
pub enum Status {
    Active,
    Blocked,
    Maintenance,
    Waiting,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Status::Active => "active",
                Status::Blocked => "blocked",
                Status::Maintenance => "maintenance",
                Status::Waiting => "waiting",
            }
        )
    }
}

// TODO: drop the '.expect', figure out error handling/propogating
fn set(status: Status, msg: &str) {
    Command::new("status-set")
        .args([status.to_string().as_str(), msg])
        .output()
        .expect("failed to execute status-set");
}

pub fn active(msg: &str) {
    set(Status::Active, msg)
}

// TODO: fns for other statuses
// TODO: research applications vs unit status, and payload status
