use crate::types::{Event, LogLevel, Status};
use std::collections::HashMap;

use crate::backend::Backend;

pub struct PortManager<'a, B> {
    backend: &'a B,
}

impl<'a, B> PortManager<'a, B>
where
    B: Backend,
{
    pub fn new(backend: &'a B) -> Self {
        Self { backend }
    }

    pub fn ports(&self) -> Vec<String> {
        self.backend.opened_ports()
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
    pub state: UnitStateManager<'a, B>,
}

impl<'a, B> Unit<'a, B>
where
    B: Backend,
{
    pub fn new(backend: &'a B) -> Self {
        Self {
            backend,
            state: UnitStateManager::new(backend),
        }
    }

    pub fn leader(&self) -> Option<LeaderTools<'a, B>> {
        if self.backend.is_leader() {
            Some(LeaderTools::new(&self.backend))
        } else {
            None
        }
    }

    pub fn config<C>(&self) -> C
    where
        C: serde::de::DeserializeOwned,
    {
        self.backend.config()
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
}

impl<'a, B> UnitStateManager<'a, B>
where
    B: Backend,
{
    pub fn new(backend: &'a B) -> Self {
        Self { backend }
    }

    pub fn state(&self) -> HashMap<String, String> {
        self.backend.get_unit_state()
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

pub struct EventModel<'a, B> {
    backend: &'a B,
    pub event: Event,
    pub unit: Unit<'a, B>,
    pub ports: PortManager<'a, B>,
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, B> EventModel<'a, B>
where
    B: Backend,
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

pub struct ActionModel<'a, A, B> {
    backend: &'a B,
    pub action: A,
    pub unit: Unit<'a, B>,
    pub ports: PortManager<'a, B>,
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, A, B> ActionModel<'a, A, B>
where
    B: Backend,
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

pub struct LeaderTools<'a, B> {
    backend: &'a B,
}

impl<'a, B> LeaderTools<'a, B>
where
    B: Backend,
{
    pub fn new(backend: &'a B) -> Self {
        Self { backend }
    }

    pub fn set(&self, key: &str, value: &str) {
        self.backend.leader_set(key, value)
    }

    pub fn get(&self) -> HashMap<String, String> {
        self.backend.leader_get()
    }
}
