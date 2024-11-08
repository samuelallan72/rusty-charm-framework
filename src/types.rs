use std::{
    fmt::{Display, Formatter},
    sync::OnceLock,
};

use regex::Regex;

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

pub struct ActionResultKey(String);

static ACTION_KEY_REGEX_CELL: OnceLock<Regex> = OnceLock::new();

// TODO: should we allow dotted.key.paths or support a custom nested hashmap type?
impl TryFrom<String> for ActionResultKey {
    type Error = String;

    // action-set adds the given values to the results map of the Action. This map
    // is returned to the user after the completion of the Action. Keys must start
    // and end with lowercase alphanumeric, and contain only lowercase alphanumeric,
    // hyphens and periods.  The following special keys are reserved for internal use:
    // "stdout", "stdout-encoding", "stderr", "stderr-encoding".
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let reserved = ["stdout", "stdout-encoding", "stderr", "stderr-encoding"];
        if reserved.contains(&value.as_str()) {
            return Err(format!(
                "key must not be one of these reserved values: {reserved:?}"
            ));
        }

        let action_key_regex = ACTION_KEY_REGEX_CELL
            .get_or_init(|| Regex::new(r"[a-z](:?[a-z0-9.-]*[a-z])*").unwrap());

        for s in value.split('.') {
            if s.is_empty() {
                return Err("Empty key found. Keys must contain at least one character.".to_owned());
            } else if !action_key_regex.is_match(s) {
                return Err(format!("{:?} is invalid. Keys must start and end with lowercase alphanumeric, and contain only lowercase alphanumeric and periods.", s));
            }
        }

        Ok(Self(value))
    }
}

impl ActionResultKey {
    pub fn value(self) -> String {
        self.0
    }
}
