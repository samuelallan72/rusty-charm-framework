use crate::types::{
    action_result_to_dotted_values, ActionResultKey, ActionValue, LogLevel, Status,
};
use serde_json::{self, Map, Value};
use std::{collections::HashMap, process::Command};

/// This trait is designed to allow for using a different backend for testing or to be mocked.
/// The charm event handlers should use the `CharmBackend` provided by the state;
/// they should not use this lower level backend.
pub trait Backend {
    fn set_application_version(&self, version: &str);
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
    fn is_leader(&self) -> Result<bool, String>;
    fn opened_ports(&self) -> Vec<String>;
    fn open_port(&self, port: &str, endpoints: Vec<&str>);
    fn close_port(&self, port: &str, endpoints: Vec<&str>);
    fn get_unit_state(&self) -> HashMap<String, String>;
    fn set_unit_state(&self, key: &str, value: &str);
    fn delete_unit_state(&self, key: &str);
    fn resource_path(&self, name: &str) -> String;
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
            .args(["--log-level", &level.to_string(), msg])
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

    fn set_application_version(&self, version: &str) {
        Command::new("application-version-set")
            .args([version])
            .output()
            .expect("failed to execute application-version-set");
    }

    fn is_leader(&self) -> Result<bool, String> {
        let output = Command::new("is-leader")
            .args(["--format", "json"])
            .output()
            .map_err(|e| format!("{e}"))?;
        Ok(serde_json::from_slice::<bool>(&output.stdout).unwrap())
    }

    fn opened_ports(&self) -> Vec<String> {
        let output = Command::new("opened-ports")
            .args(["--format", "json"])
            .output()
            .unwrap();
        serde_json::from_slice::<Vec<String>>(&output.stdout).unwrap()
    }

    fn open_port(&self, port: &str, endpoints: Vec<&str>) {
        let mut args = vec![];
        let endpoints = endpoints.join(",");
        if !endpoints.is_empty() {
            args.push("--endpoints");
            args.push(&endpoints);
        }
        args.push(port);

        Command::new("open-port").args(&args).output().unwrap();
    }

    fn close_port(&self, port: &str, endpoints: Vec<&str>) {
        let mut args = vec![];
        let endpoints = endpoints.join(",");
        if !endpoints.is_empty() {
            args.push("--endpoints");
            args.push(&endpoints);
        }
        args.push(port);

        Command::new("close-port").args(&args).output().unwrap();
    }

    fn get_unit_state(&self) -> HashMap<String, String> {
        let output = Command::new("state-get")
            .args(["--format", "json"])
            .output()
            .unwrap();
        serde_json::from_slice(&output.stdout).unwrap()
    }

    // NOTE: could use file from stdin if need to save large state.
    // A possible optimisation for the future.
    fn set_unit_state(&self, key: &str, value: &str) {
        // TODO: limit key to not contain `=`?
        Command::new("state-set")
            .args([&format!("{key}={value}")])
            .output()
            .unwrap();
    }

    fn delete_unit_state(&self, key: &str) {
        Command::new("state-delete").args([key]).output().unwrap();
    }

    fn resource_path(&self, name: &str) -> String {
        let output = Command::new("resource-get").args([name]).output().unwrap();
        String::from_utf8(output.stdout).unwrap()
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

    /// Set the workload application version.
    pub fn set_application_version(&self, version: &str) {
        self.backend.set_application_version(version)
    }
}

pub struct Ports<'a, B> {
    pub ports: Vec<String>,
    backend: &'a B,
}

impl<'a, B> Ports<'a, B>
where
    B: Backend,
{
    pub fn load_from_backend(backend: &'a B) -> Self {
        Self {
            backend,
            ports: backend.opened_ports(),
        }
    }

    pub fn open_port(&self, port: &str, endpoints: Vec<&str>) {
        self.backend.open_port(port, endpoints)
    }

    pub fn close_port(&self, port: &str, endpoints: Vec<&str>) {
        self.backend.close_port(port, endpoints)
    }
}

pub struct Unit<'a, B> {
    backend: &'a B,
    pub leader: bool,
    pub state: UnitState<'a, B>,
}

pub struct UnitState<'a, B> {
    backend: &'a B,
    state: HashMap<String, String>,
}

impl<'a, B> Unit<'a, B>
where
    B: Backend,
{
    pub fn load_from_backend(backend: &'a B) -> Self {
        Self {
            backend,
            leader: backend.is_leader().unwrap(),
            state: UnitState::load_from_backend(backend),
        }
    }

    pub fn resource_path(&self, name: &str) -> String {
        self.backend.resource_path(name)
    }
}

impl<'a, B> UnitState<'a, B>
where
    B: Backend,
{
    // TODO: is loading from the backend on init a good pattern?
    // Another option may be to load and cache on first call to get, etc.
    // Same thought for the others with load_from_backend.
    pub fn load_from_backend(backend: &'a B) -> Self {
        Self {
            backend,
            state: backend.get_unit_state(),
        }
    }

    pub fn set(&self, key: &str, value: &str) {
        self.backend.set_unit_state(key, value)
    }

    pub fn del(&self, key: &str) {
        self.backend.delete_unit_state(key)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.state.get(key).map(|x| x.as_str())
    }
}
