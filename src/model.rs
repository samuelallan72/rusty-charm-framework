use crate::types::{Event, LogLevel, Status};
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::backend::Backend;

pub struct PortManager<'a, B> {
    ports: OnceLock<Vec<String>>,
    backend: &'a B,
}

impl<'a, B> PortManager<'a, B>
where
    B: Backend,
{
    pub fn new(backend: &'a B) -> Self {
        Self {
            backend,
            ports: OnceLock::new(),
        }
    }

    pub fn ports(&self) -> &Vec<String> {
        self.ports.get_or_init(|| self.backend.opened_ports())
    }

    pub fn open_port(&self, port: &str, endpoints: Vec<&str>) {
        self.backend.open_port(port, endpoints)
    }

    pub fn close_port(&self, port: &str, endpoints: Vec<&str>) {
        self.backend.close_port(port, endpoints)
    }
}

pub struct Unit<'a, B, C> {
    backend: &'a B,
    is_leader_cache: OnceLock<bool>,
    config_cache: OnceLock<C>,
    pub state: UnitStateManager<'a, B>,
}

impl<'a, B, C> Unit<'a, B, C>
where
    B: Backend,
    C: serde::de::DeserializeOwned,
{
    pub fn new(backend: &'a B) -> Self {
        Self {
            backend,
            config_cache: OnceLock::new(),
            is_leader_cache: OnceLock::new(),
            state: UnitStateManager::new(backend),
        }
    }

    pub fn is_leader(&self) -> bool {
        *self
            .is_leader_cache
            .get_or_init(|| self.backend.is_leader().unwrap())
    }

    pub fn config(&self) -> &C {
        self.config_cache.get_or_init(|| self.backend.config())
    }

    pub fn resource_path(&self, name: &str) -> String {
        self.backend.resource_path(name)
    }

    /// Set the workload application version.
    pub fn set_application_version(&self, version: &str) {
        self.backend.set_application_version(version)
    }
}

pub struct UnitStateManager<'a, B> {
    backend: &'a B,
    state: OnceLock<HashMap<String, String>>,
}

impl<'a, B> UnitStateManager<'a, B>
where
    B: Backend,
{
    pub fn new(backend: &'a B) -> Self {
        Self {
            backend,
            state: OnceLock::new(),
        }
    }

    pub fn state(&self) -> &HashMap<String, String> {
        self.state.get_or_init(|| self.backend.get_unit_state())
    }

    pub fn set(&self, key: &str, value: &str) {
        self.backend.set_unit_state(key, value)
    }

    pub fn del(&self, key: &str) {
        self.backend.delete_unit_state(key)
    }
}

pub struct StatusManager<'a, B> {
    backend: &'a B,
}

impl<'a, B> StatusManager<'a, B>
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
}

pub struct Logger<'a, B> {
    backend: &'a B,
}

impl<'a, B> Logger<'a, B>
where
    B: Backend,
{
    pub fn new(backend: &'a B) -> Self {
        Self { backend }
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
}

pub struct EventModel<'a, B, C> {
    backend: &'a B,
    pub event: Event,
    pub unit: Unit<'a, B, C>,
    pub ports: PortManager<'a, B>,
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, B, C> EventModel<'a, B, C>
where
    B: Backend,
    C: serde::de::DeserializeOwned,
{
    pub fn new(backend: &'a B, event: Event) -> Self {
        Self {
            event,
            backend: &backend,
            unit: Unit::new(&backend),
            ports: PortManager::new(&backend),
            status: StatusManager::new(&backend),
            log: Logger::new(&backend),
        }
    }

    // these should only be called in an event hook
    pub fn reboot(&self) {
        self.backend.reboot(false)
    }

    pub fn reboot_now(&self) {
        self.backend.reboot(true)
    }
}

pub struct ActionModel<'a, A, B, C> {
    backend: &'a B,
    pub action: A,
    pub unit: Unit<'a, B, C>,
    pub ports: PortManager<'a, B>,
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, A, B, C> ActionModel<'a, A, B, C>
where
    B: Backend,
    C: serde::de::DeserializeOwned,
{
    pub fn new(backend: &'a B, action: A) -> Self {
        Self {
            action,
            backend: &backend,
            unit: Unit::new(&backend),
            ports: PortManager::new(&backend),
            status: StatusManager::new(&backend),
            log: Logger::new(&backend),
        }
    }

    // these should only be called in an action
    pub fn action_log(&self, msg: &str) {
        self.backend.action_log(msg)
    }
}
