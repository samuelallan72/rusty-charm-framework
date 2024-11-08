use std::{
    collections::HashMap,
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

#[derive(Debug)]
pub enum ActionValue {
    Value(String),
    Nested(HashMap<ActionResultKey, ActionValue>),
}

pub type ActionResult =
    Result<HashMap<ActionResultKey, ActionValue>, (String, HashMap<ActionResultKey, ActionValue>)>;

#[derive(Debug, PartialEq, Eq, Hash)]
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
        let action_key_regex = ACTION_KEY_REGEX_CELL
            .get_or_init(|| Regex::new(r"[a-z](:?[a-z0-9.-]*[a-z])*").unwrap());

        if reserved.contains(&value.as_str()) {
            Err(format!(
                "key must not be one of these reserved values: {reserved:?}"
            ))
        } else if value.is_empty() {
            Err("Empty key found. Keys must contain at least one character.".to_owned())
        } else if !action_key_regex.is_match(&value) {
            Err(format!("{:?} is invalid. Keys must start and end with lowercase alphanumeric, and contain only lowercase alphanumeric and hyphens.", value))
        } else {
            Ok(Self(value))
        }
    }
}

impl ActionResultKey {
    pub fn value(self) -> String {
        self.0
    }
}

// Convert a custom nested hashmap into path.to.key=value notation for juju action-set.
pub(crate) fn action_result_to_dotted_values(
    data: HashMap<ActionResultKey, ActionValue>,
) -> Vec<String> {
    let mut result_values = vec![];
    for (key, value) in data.into_iter() {
        let prefix: String = key.value();

        match value {
            ActionValue::Value(value) => {
                result_values.push(format!("{prefix}={value}"));
            }
            ActionValue::Nested(hash_map) => {
                for partial_value in action_result_to_dotted_values(hash_map) {
                    result_values.push(format!("{prefix}.{partial_value}"));
                }
            }
        }
    }

    result_values
}
