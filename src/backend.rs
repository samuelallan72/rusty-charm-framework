use crate::types::{
    action_result_to_dotted_values, ActionResultKey, ActionValue, LogLevel, Status,
};
use serde_json::{self, Map, Value};
use std::{collections::HashMap, process::Command};

/// This trait is designed to allow for using a different backend for testing or to be mocked.
/// The charm event handlers should use the `CharmBackend` provided by the state;
/// they should not use this lower level backend.
pub trait Backend {
    fn set_action_fail(&self, msg: &str);
    fn set_action_result(&self, data: HashMap<ActionResultKey, ActionValue>);
    fn action_name(&self) -> String;
    fn hook_name(&self) -> String;
    /// Log a message to the juju log, at the desired log level.
    fn log(&self, msg: &str, level: LogLevel);
    /// Build an action struct, given the action name.
    /// The method should pull in the the action params and return something that can be
    /// deserialised into the desired action struct.
    fn action<A>(&self, name: &str) -> A
    where
        A: serde::de::DeserializeOwned;
    /// Retrieve the charm's current config as something that can be deserialised.
    fn config<C>(&self) -> C
    where
        C: serde::de::DeserializeOwned;
    /// Set the unit status.
    fn set_status(&self, status: Status);
    /// Log a message to the action log. Only call this during an action event (ie. from the
    /// action handler function).
    fn action_log(&self, msg: &str);
}

/// The real implementation for the backend.
pub struct JujuBackend {}

impl Backend for JujuBackend {
    fn action_log(&self, msg: &str) {
        Command::new("action-log")
            .args([msg])
            .output()
            .expect("failed to execute action-log");
    }

    fn config<C>(&self) -> C
    where
        C: serde::de::DeserializeOwned,
    {
        let output = Command::new("config-get")
            .args(["--format", "json", "--all"])
            .output()
            .expect("failed to execute config-get");
        serde_json::from_slice::<C>(&output.stdout).unwrap()
    }

    fn set_status(&self, status: Status) {
        Command::new("status-set")
            .args([status.name(), status.msg()])
            .output()
            .expect("failed to execute status-set");
    }

    fn log(&self, msg: &str, level: LogLevel) {
        Command::new("juju-log")
            .args(["--log-level", level.to_string().as_str(), msg])
            .output()
            .expect("failed to execute juju-log");
    }

    fn action<A>(&self, name: &str) -> A
    where
        A: serde::de::DeserializeOwned,
    {
        let output = Command::new("action-get")
            .args(["--format", "json"])
            .output()
            .expect("failed to execute action-get");
        let params = serde_json::from_slice::<Value>(&output.stdout).unwrap();

        let action_value = Value::Object({
            let mut map = Map::new();
            map.insert(name.to_owned(), params);
            map
        });
        serde_json::from_value(action_value).unwrap()
    }

    fn hook_name(&self) -> String {
        std::env::var("JUJU_HOOK_NAME").unwrap_or("".to_owned())
    }

    fn action_name(&self) -> String {
        std::env::var("JUJU_ACTION_NAME").unwrap_or("".to_owned())
    }

    fn set_action_result(&self, data: HashMap<ActionResultKey, ActionValue>) {
        if data.is_empty() {
            return;
        }
        Command::new("action-set")
            .args(action_result_to_dotted_values(data))
            .output()
            .expect("failed to execute action-set");
    }

    fn set_action_fail(&self, msg: &str) {
        Command::new("action-fail")
            .args([msg])
            .output()
            .expect("failed to execute action-set");
    }
}

/// This is the interface for the backend that the charm will see.
/// It provides helpful methods for interacting with juju and the environment.
pub struct CharmBackend<'a, B> {
    backend: &'a B,
}

impl<'a, B> CharmBackend<'a, B>
where
    B: Backend,
{
    pub fn new(backend: &'a B) -> Self {
        Self { backend }
    }
    pub fn active(&self, msg: &str) {
        self.backend.set_status(Status::Active(msg))
    }

    pub fn blocked(&self, msg: &str) {
        self.backend.set_status(Status::Blocked(msg))
    }

    pub fn maintenance(&self, msg: &str) {
        self.backend.set_status(Status::Maintenance(msg))
    }

    pub fn waiting(&self, msg: &str) {
        self.backend.set_status(Status::Waiting(msg))
    }

    pub fn debug(&self, msg: &str) {
        self.backend.log(msg, LogLevel::Debug)
    }

    pub fn info(&self, msg: &str) {
        self.backend.log(msg, LogLevel::Info)
    }

    pub fn warn(&self, msg: &str) {
        self.backend.log(msg, LogLevel::Warning)
    }

    pub fn error(&self, msg: &str) {
        self.backend.log(msg, LogLevel::Error)
    }

    /// Log a message to the action log. Only call this during an action event (ie. from the
    /// action handler function).
    pub fn action_log(&self, msg: &str) {
        self.backend.action_log(msg)
    }
}
