use std::fmt::{Display, Formatter};

// Error is also a status, but not one that can be directly set.
pub enum Status<'a> {
    Active(&'a str),
    Blocked(&'a str),
    Maintenance(&'a str),
    Waiting(&'a str),
}

impl<'a> Status<'a> {
    pub fn name(&self) -> &str {
        match self {
            Status::Active(_) => "active",
            Status::Blocked(_) => "blocked",
            Status::Maintenance(_) => "maintenance",
            Status::Waiting(_) => "waiting",
        }
    }

    pub fn msg(&self) -> &str {
        match self {
            Status::Active(x)
            | Status::Blocked(x)
            | Status::Waiting(x)
            | Status::Maintenance(x) => x,
        }
    }
}

pub enum LogLevel {
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

// ref. https://github.com/canonical/charm-events
pub enum Event {
    CollectMetrics,
    ConfigChanged,
    Install,
    LeaderElected,
    LeaderSettingsChanged,
    PebbleCustomNotice,
    PebbleReady(String),
    PostSeriesUpgrade,
    PreSeriesUpgrade,
    RelationBroken(String),
    RelationChanged(String),
    RelationCreated(String),
    RelationDeparted(String),
    RelationJoined(String),
    Remove,
    SecretChanged,
    SecretExpire,
    SecretRemoved,
    SecretRotate,
    Start,
    Stop,
    StorageAttached(String),
    StorageDetatched(String),
    UpdateStatus,
    UpgradeCharm,
}

pub enum ActionResult {
    Success,
    Failure,
}
