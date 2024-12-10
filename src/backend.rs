use std::{
    collections::HashMap,
    io::Write,
    process::{Command, Stdio},
};

use serde_json::{self, Map, Value};

use crate::error::{Error, Result};
use crate::types::{ActionResultKey, ActionValue, JujuCredentials, LogLevel, Status};

/// This trait is designed to allow for using a different backend for testing or to be mocked.
/// The charm event handlers should use the `CharmBackend` provided by the state;
/// they should not use this lower level backend.
pub trait Backend {
    fn leader_get(&self) -> Result<HashMap<String, String>>;
    fn leader_set(&self, key: &str, value: &str) -> Result<()>;
    fn credentials(&self) -> Result<JujuCredentials>;
    fn reboot(&self, now: bool) -> Result<()>;
    fn set_application_version(&self, version: &str) -> Result<()>;
    fn set_action_fail(&self, msg: &str) -> Result<()>;
    fn set_action_result(&self, data: HashMap<ActionResultKey, ActionValue>) -> Result<()>;
    fn action_name(&self) -> Result<String>;
    fn hook_name(&self) -> Result<String>;
    /// Log a message to the juju log, at the desired log level.
    fn log(&self, msg: &str, level: LogLevel) -> Result<()>;
    fn action<A>(&self) -> Result<A>
    where
        A: serde::de::DeserializeOwned;
    /// Retrieve the charm's current config as something that can be deserialised.
    fn config<C>(&self) -> Result<C>
    where
        C: serde::de::DeserializeOwned;
    /// Set the unit status.
    fn set_status(&self, status: Status) -> Result<()>;
    /// Set the application status. This can only be called from a leader unit.
    fn set_app_status(&self, status: Status) -> Result<()>;
    /// Log a message to the action log. Only call this during an action event (ie. from the
    /// action handler function).
    fn action_log(&self, msg: &str) -> Result<()>;
    fn is_leader(&self) -> Result<bool>;
    fn opened_ports(&self) -> Result<Vec<String>>;
    fn open_port(&self, port: &str, endpoints: Vec<&str>) -> Result<()>;
    fn close_port(&self, port: &str, endpoints: Vec<&str>) -> Result<()>;
    fn get_unit_state(&self) -> Result<HashMap<String, String>>;
    fn set_unit_state(&self, key: &str, value: &str) -> Result<()>;
    fn delete_unit_state(&self, key: &str) -> Result<()>;
    fn resource_path(&self, name: &str) -> Result<String>;
}

/// The real implementation for the backend.
pub struct JujuBackend {}

impl Backend for JujuBackend {
    fn action_log(&self, msg: &str) -> Result<()> {
        Command::new("action-log").args([msg]).output()?;
        Ok(())
    }

    fn config<C>(&self) -> Result<C>
    where
        C: serde::de::DeserializeOwned,
    {
        let output = Command::new("config-get")
            .args(["--format", "json", "--all"])
            .output()?;
        Ok(serde_json::from_slice::<C>(&output.stdout)?)
    }

    fn set_status(&self, status: Status) -> Result<()> {
        Command::new("status-set")
            .args([status.name(), status.msg()])
            .output()?;
        Ok(())
    }

    fn set_app_status(&self, status: Status) -> Result<()> {
        Command::new("status-set")
            .args(["--application", status.name(), status.msg()])
            .output()?;
        Ok(())
    }

    fn log(&self, msg: &str, level: LogLevel) -> Result<()> {
        Command::new("juju-log")
            .args(["--log-level", &level.to_string(), msg])
            .output()?;
        Ok(())
    }

    fn action<A>(&self) -> Result<A>
    where
        A: serde::de::DeserializeOwned,
    {
        let name = self.action_name()?;
        let output = Command::new("action-get")
            .args(["--format", "json"])
            .output()?;
        let params = serde_json::from_slice::<Value>(&output.stdout)?;

        let action_value = Value::Object({
            let mut map = Map::new();
            map.insert(name.to_owned(), params);
            map
        });
        Ok(serde_json::from_value(action_value)?)
    }

    fn hook_name(&self) -> Result<String> {
        Ok(std::env::var("JUJU_HOOK_NAME")?)
    }

    fn action_name(&self) -> Result<String> {
        Ok(std::env::var("JUJU_ACTION_NAME")?)
    }

    fn set_action_result(&self, data: HashMap<ActionResultKey, ActionValue>) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        Command::new("action-set")
            .args(action_result_to_dotted_values(data))
            .output()?;
        Ok(())
    }

    fn set_action_fail(&self, msg: &str) -> Result<()> {
        Command::new("action-fail").args([msg]).output()?;
        Ok(())
    }

    fn set_application_version(&self, version: &str) -> Result<()> {
        Command::new("application-version-set")
            .args([version])
            .output()?;
        Ok(())
    }

    fn is_leader(&self) -> Result<bool> {
        let output = Command::new("is-leader")
            .args(["--format", "json"])
            .output()?;
        Ok(serde_json::from_slice::<bool>(&output.stdout)?)
    }

    fn opened_ports(&self) -> Result<Vec<String>> {
        let output = Command::new("opened-ports")
            .args(["--format", "json"])
            .output()?;
        Ok(serde_json::from_slice::<Vec<String>>(&output.stdout)?)
    }

    fn open_port(&self, port: &str, endpoints: Vec<&str>) -> Result<()> {
        let mut args = vec![];
        let endpoints = endpoints.join(",");
        if !endpoints.is_empty() {
            args.push("--endpoints");
            args.push(&endpoints);
        }
        args.push(port);

        Command::new("open-port").args(&args).output()?;
        Ok(())
    }

    fn close_port(&self, port: &str, endpoints: Vec<&str>) -> Result<()> {
        let mut args = vec![];
        let endpoints = endpoints.join(",");
        if !endpoints.is_empty() {
            args.push("--endpoints");
            args.push(&endpoints);
        }
        args.push(port);

        Command::new("close-port").args(&args).output()?;
        Ok(())
    }

    fn get_unit_state(&self) -> Result<HashMap<String, String>> {
        let output = Command::new("state-get")
            .args(["--format", "json"])
            .output()?;
        Ok(serde_json::from_slice(&output.stdout)?)
    }

    // NOTE: setting the unit state will not reflect in the state returned from state-get
    // until the next hook invocation.
    fn set_unit_state(&self, key: &str, value: &str) -> Result<()> {
        let process = Command::new("state-set")
            .args(["--file", "-"])
            .stdin(Stdio::piped())
            .spawn()?;

        let json_data = Value::Object({
            let mut map = Map::new();
            map.insert(key.to_owned(), Value::String(value.to_owned()));
            map
        });
        let data = serde_json::to_vec(&json_data)?;
        let mut stdin = process.stdin.ok_or(Error::StdinError())?;
        stdin.write_all(&data)?;
        Ok(())
    }

    fn delete_unit_state(&self, key: &str) -> Result<()> {
        Command::new("state-delete").args([key]).output()?;
        Ok(())
    }

    fn resource_path(&self, name: &str) -> Result<String> {
        let output = Command::new("resource-get").args([name]).output()?;
        Ok(String::from_utf8(output.stdout)?)
    }

    fn reboot(&self, now: bool) -> Result<()> {
        let args = if now { vec!["--now"] } else { vec![] };
        Command::new("juju-reboot").args(&args).output()?;
        Ok(())
    }

    fn credentials(&self) -> Result<JujuCredentials> {
        let output = Command::new("credential-get")
            .args(["--format", "json"])
            .output()?;
        Ok(serde_json::from_slice(&output.stdout)?)
    }

    fn leader_set(&self, key: &str, value: &str) -> Result<()> {
        // TODO: limit key to not contain `=`?
        Command::new("leader-set")
            .args([&format!("{key}={value}")])
            .output()?;
        Ok(())
    }

    fn leader_get(&self) -> Result<HashMap<String, String>> {
        let output = Command::new("leader-get")
            .args(["--format", "json"])
            .output()?;
        Ok(serde_json::from_slice(&output.stdout)?)
    }
}

// Convert a custom nested hashmap into path.to.key=value notation for juju action-set.
fn action_result_to_dotted_values(data: HashMap<ActionResultKey, ActionValue>) -> Vec<String> {
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
