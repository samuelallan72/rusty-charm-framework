use crate::types::{Event, LogLevel, Status};
use std::collections::HashMap;

use crate::backend::Backend;
use crate::error::Result;

pub struct PortManager<'a, B> {
    backend: &'a B,
}

impl<'a, B> PortManager<'a, B>
where
    B: Backend,
{
    fn new(backend: &'a B) -> Self {
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
    /// Methods to work with server-side state key/value pairs.
    /// Use these to persist state about the unit across hook invocations.
    /// For example:
    /// ```
    /// model.unit.state.set("key1", "value1");
    /// model.unit.state.read().get("key1"); // -> Some("value1")
    /// ```
    pub state: UnitStateManager<'a, B>,
}

impl<'a, B> Unit<'a, B>
where
    B: Backend,
{
    fn new(backend: &'a B) -> Self {
        Self {
            backend,
            state: UnitStateManager::new(backend),
        }
    }

    /// If the unit is the leader,
    /// return a set of methods that are only applicable if the unit is the leader.
    /// Otherwise return `None`.
    ///
    /// ```
    /// if let Some(leader_tools) = model.unit.leader() {
    ///     leader_tools.set("key", "value");
    ///     leader_tools.app_status.maintenance("app units are busy doing things");
    /// }
    /// ```
    /// This can also be used to simply check if the unit is leader:
    ///
    /// ```
    /// let is_leader: bool = model.unit.leader().is_some();
    /// ```
    pub fn leader(&self) -> Option<LeaderTools<'a, B>> {
        if self.backend.is_leader() {
            Some(LeaderTools::new(self.backend))
        } else {
            None
        }
    }

    pub fn config<C>(&self) -> Result<C>
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

/// Methods for managing the server-side unit state.
/// This state is scoped to the unit - it is not visible to other units.
///
/// Example:
/// ```
/// let data = model.state.read();
/// if ! data.contains_key("install-completed") {
///     install_things();
///     model.state.set("install-completed", "yes");
/// }
/// ```
pub struct UnitStateManager<'a, B> {
    backend: &'a B,
}

impl<'a, B> UnitStateManager<'a, B>
where
    B: Backend,
{
    fn new(backend: &'a B) -> Self {
        Self { backend }
    }

    /// Load the full unit state from the server.
    /// Calls the `state-get` hook-tool.
    /// Returns a hashmap of key -> value as strings.
    /// This is **not** cached.
    /// Changes to the returned hashmap are not updated server-side;
    /// use `UnitStateManager.set(key, value)` or `UnitStateManager.del(key)` to persist
    /// changes.
    pub fn read(&self) -> HashMap<String, String> {
        self.backend.get_unit_state()
    }

    /// Set `key` to `value` in the server-side state.
    /// Uses the `state-set` hook-tool.
    pub fn set(&self, key: &str, value: &str) {
        self.backend.set_unit_state(key, value)
    }

    /// Delete a `key` from the server-side state.
    /// Uses the `state-delete` hook-tool.
    pub fn del(&self, key: &str) {
        self.backend.delete_unit_state(key)
    }
}

/// Methods to update the unit status.
/// For example:
///
/// ```
/// model.status.maintenance("running some maintenance task");
/// do_maintenance();
/// model.status.maintenance("running stage 2 maintenance");
/// do_more_maintenance();
/// ```
///
/// ---
///
/// Note that it's not possible to set an error status.
/// Error status and message will be set by juju if the hook fails
/// (in the context of the charm code,
/// it means if the code panics or an Err(_) is returned from the handler.
///
/// ---
///
/// Passing empty messages is fine, but usually only recommended if charm is operational (active)
/// and there is nothing special to report.
///
/// ```
/// model.status.active("");
/// ```
///
/// In other cases it's important to provide a message to indicate to the user why the charm isn't
/// active, and what can be done to rectify.
///
/// ```
/// model.status.blocked("a db relation is required");
/// model.status.waiting("db is connected but not ready yet");
/// ```
pub struct StatusManager<'a, B> {
    backend: &'a B,
}

impl<'a, B> StatusManager<'a, B>
where
    B: Backend,
{
    fn new(backend: &'a B) -> Self {
        Self { backend }
    }

    /// Set the unit status to active.
    /// Use this status to indicate everything is operational and running.
    /// ```
    /// model.status.active("");
    /// ```
    pub fn active(&self, msg: &str) {
        self.backend.set_status(Status::Active(msg))
    }

    /// Set the unit status to blocked.
    /// Use this if the charm requires manual intervention to continue operation.
    /// ```
    /// model.status.blocked("a relation to a database is required");
    /// ```
    pub fn blocked(&self, msg: &str) {
        self.backend.set_status(Status::Blocked(msg))
    }

    /// Set the unit status to maintenance.
    /// Use this to indicate the charm is busy doing tasks and not currently operational.
    /// ```
    /// model.status.maintenance("migrating db tables");
    /// ```
    pub fn maintenance(&self, msg: &str) {
        self.backend.set_status(Status::Maintenance(msg))
    }

    /// Set the unit status to waiting.
    /// Use this to indicate the charm doesn't have everything it needs yet,
    /// but expects it will automatically soon (no manual intervention required).
    /// ```
    /// model.status.waiting("db is connected but not ready yet");
    /// ```
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
    fn new(backend: &'a B) -> Self {
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
    /// Contains methods to update the unit status.
    /// Usually, the event handler should not need to use these;
    /// instead it should return a status to be set on hook completion.
    /// However, this may be useful to keep the user updated when there are long running
    /// hooks.
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, B> EventModel<'a, B>
where
    B: Backend,
{
    pub(crate) fn new(backend: &'a B, event: Event) -> Self {
        Self {
            event,
            backend,
            unit: Unit::new(backend),
            ports: PortManager::new(backend),
            status: StatusManager::new(backend),
            log: Logger::new(backend),
        }
    }

    // these can only be called in an event hook

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
    /// Contains methods to update the unit status.
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, A, B> ActionModel<'a, A, B>
where
    B: Backend,
{
    pub(crate) fn new(backend: &'a B, action: A) -> Self {
        Self {
            action,
            backend,
            unit: Unit::new(backend),
            ports: PortManager::new(backend),
            status: StatusManager::new(backend),
            log: Logger::new(backend),
        }
    }

    // these can only be called in an action

    pub fn action_log(&self, msg: &str) {
        self.backend.action_log(msg)
    }
}

pub struct LeaderTools<'a, B> {
    backend: &'a B,
    pub app_status: AppStatus<'a, B>,
}

impl<'a, B> LeaderTools<'a, B>
where
    B: Backend,
{
    fn new(backend: &'a B) -> Self {
        Self {
            backend,
            app_status: AppStatus::new(backend),
        }
    }

    pub fn set(&self, key: &str, value: &str) {
        self.backend.leader_set(key, value)
    }

    pub fn get(&self) -> Result<HashMap<String, String>> {
        self.backend.leader_get()
    }
}

pub struct AppStatus<'a, B> {
    backend: &'a B,
}

impl<'a, B> AppStatus<'a, B>
where
    B: Backend,
{
    fn new(backend: &'a B) -> Self {
        Self { backend }
    }

    pub fn active(&self, msg: &str) {
        self.backend.set_app_status(Status::Active(msg))
    }

    pub fn blocked(&self, msg: &str) {
        self.backend.set_app_status(Status::Blocked(msg))
    }

    pub fn maintenance(&self, msg: &str) {
        self.backend.set_app_status(Status::Maintenance(msg))
    }

    pub fn waiting(&self, msg: &str) {
        self.backend.set_app_status(Status::Waiting(msg))
    }
}
