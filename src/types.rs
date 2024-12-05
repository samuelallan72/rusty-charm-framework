use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::LazyLock,
};

use regex::Regex;
use serde::Deserialize;

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
    StorageDetached(String),
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

static ACTION_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[a-z](:?[a-z0-9.-]*[a-z])*").expect("hardcoded regex in codebase was invalid")
});

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
            Err(format!(
                "key must not be one of these reserved values: {reserved:?}"
            ))
        } else if value.is_empty() {
            Err("Empty key found. Keys must contain at least one character.".to_owned())
        } else if !ACTION_KEY_REGEX.is_match(&value) {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct JujuCredentialsCredentialAttrs {
    pub client_cert: String,
    pub client_key: String,
    pub server_cert: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct JujuCredentialsCredential {
    pub auth_type: String,
    pub attrs: JujuCredentialsCredentialAttrs,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct JujuCredentials {
    #[serde(rename = "type")]
    pub cloud_type: String,
    pub name: String,
    pub region: String,
    pub endpoint: String,
    pub credential: JujuCredentialsCredential,
    pub is_controller_cloud: bool,
}
