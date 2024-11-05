use std::process::Command;

// Error is also a status, but not one that can be directly set.
pub enum Status {
    Active(String),
    Blocked(String),
    Maintenance(String),
    Waiting(String),
}

impl Status {
    pub fn name(&self) -> &str {
        match self {
            Status::Active(_) => "active",
            Status::Blocked(_) => "blocked",
            Status::Maintenance(_) => "maintenance",
            Status::Waiting(_) => "waiting",
        }
    }

    fn msg(&self) -> &str {
        match self {
            Status::Active(x)
            | Status::Blocked(x)
            | Status::Waiting(x)
            | Status::Maintenance(x) => x.as_str(),
        }
    }
}

// TODO: drop the '.expect', figure out error handling/propogating
fn set(status: Status) {
    Command::new("status-set")
        .args([status.name(), status.msg()])
        .output()
        .expect("failed to execute status-set");
}

pub fn active(msg: String) {
    set(Status::Active(msg))
}

// TODO: fns for other statuses
// TODO: research applications vs unit status, and payload status
