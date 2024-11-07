use crate::types::{LogLevel, Status};
use serde_json::{self, Map, Value};
use std::process::Command;

pub trait Backend {
    fn action<A>(&self, name: &str) -> A
    where
        A: serde::de::DeserializeOwned;
    fn config<C>(&self) -> C
    where
        C: serde::de::DeserializeOwned;
    fn blocked(&self, msg: &str);
    fn maintenance(&self, msg: &str);
    fn waiting(&self, msg: &str);
    fn active(&self, msg: &str);
    fn action_log(&self, msg: &str);
    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
}

pub struct RealBackend {}

impl RealBackend {
    // TODO: drop the '.expect', figure out error handling/propogating
    fn log(msg: &str, level: LogLevel) {
        Command::new("juju-log")
            .args(["--log-level", level.to_string().as_str(), msg])
            .output()
            .expect("failed to execute juju-log");
    }

    // TODO: drop the '.expect', figure out error handling/propogating
    fn set(status: Status) {
        Command::new("status-set")
            .args([status.name(), status.msg()])
            .output()
            .expect("failed to execute status-set");
    }
}

impl Backend for RealBackend {
    fn action_log(&self, msg: &str) {
        let output = Command::new("action-log")
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

    fn debug(&self, msg: &str) {
        Self::log(msg, LogLevel::Debug)
    }

    fn info(&self, msg: &str) {
        Self::log(msg, LogLevel::Info)
    }

    fn warn(&self, msg: &str) {
        Self::log(msg, LogLevel::Warning)
    }

    fn error(&self, msg: &str) {
        Self::log(msg, LogLevel::Error)
    }

    fn active(&self, msg: &str) {
        Self::set(Status::Active(msg))
    }

    fn blocked(&self, msg: &str) {
        Self::set(Status::Blocked(msg))
    }

    fn maintenance(&self, msg: &str) {
        Self::set(Status::Maintenance(msg))
    }

    fn waiting(&self, msg: &str) {
        Self::set(Status::Waiting(msg))
    }
}

// TODO: fns for other statuses
// TODO: research applications vs unit status, and payload status
