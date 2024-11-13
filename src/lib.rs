pub mod backend;
pub mod types;

use backend::{Backend, Logger, Ports, StatusManager, Unit};
use types::{Event, LogLevel};

pub struct EventModel<'a, B, C> {
    backend: &'a B,
    pub event: Event,
    pub unit: Unit<'a, B, C>,
    pub ports: Ports<'a, B>,
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, B, C> EventModel<'a, B, C>
where
    B: Backend,
    C: serde::de::DeserializeOwned,
{
    pub(crate) fn new(backend: &'a B, event: Event) -> Self {
        Self {
            event,
            backend: &backend,
            unit: Unit::new(&backend),
            ports: Ports::new(&backend),
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
    pub ports: Ports<'a, B>,
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, A, B, C> ActionModel<'a, A, B, C>
where
    B: Backend,
    C: serde::de::DeserializeOwned,
{
    pub(crate) fn new(backend: &'a B, action: A) -> Self {
        Self {
            action,
            backend: &backend,
            unit: Unit::new(&backend),
            ports: Ports::new(&backend),
            status: StatusManager::new(&backend),
            log: Logger::new(&backend),
        }
    }

    // these should only be called in an action
    pub fn action_log(&self, msg: &str) {
        self.backend.action_log(msg)
    }
}

pub struct Framework<A, B, C> {
    backend: B,
    event_handler: fn(EventModel<B, C>) -> types::Status,
    action_handler: fn(ActionModel<A, B, C>) -> types::ActionResult,
}

impl<A, B, C> Framework<A, B, C>
where
    B: Backend,
    C: serde::de::DeserializeOwned,
    A: serde::de::DeserializeOwned,
{
    pub fn new(
        backend: B,
        event_handler: fn(EventModel<B, C>) -> types::Status,
        action_handler: fn(ActionModel<A, B, C>) -> types::ActionResult,
    ) -> Self {
        Self {
            backend,
            event_handler,
            action_handler,
        }
    }

    /// Process the current event, hook, or action from the environment,
    /// populating local state, and calling the handler functions as appropriate.
    /// `event_handler` must return a status - this status will be the final status set before
    /// execution of the hook finishes.
    /// The event handler may explicitly set a status during execution.
    /// This may be useful in the case of a long running hook (eg. set a maintenance ongoing status at
    /// the beginning).
    pub fn execute(self) {
        // debug log all env vars for testing purposes
        for (key, value) in std::env::vars() {
            self.backend
                .log(format!("{key}: {value}").as_str(), LogLevel::Debug);
        }

        // ref. https://juju.is/docs/juju/charm-environment-variables for logic
        let hook_name = self.backend.hook_name();
        if !hook_name.is_empty() {
            self.backend.log(
                format!("running handlers for {hook_name} hook").as_str(),
                LogLevel::Debug,
            );

            let event = match hook_name.as_str() {
                "install" => Event::Install,
                "config-changed" => Event::ConfigChanged,
                "remove" => Event::Remove,
                "update-status" => Event::UpdateStatus,
                "upgrade-charm" => Event::UpgradeCharm,
                _ => Event::UpdateStatus, // TODO: other events
            };

            let model = EventModel::new(&self.backend, event);

            (self.event_handler)(model);
            return;
        }

        let action_name = self.backend.action_name();
        if !action_name.is_empty() {
            self.backend.log(
                format!("running handler for {action_name} action").as_str(),
                LogLevel::Debug,
            );
            let action: A = self.backend.action(action_name.as_str());

            let model = ActionModel::new(&self.backend, action);

            let result = (self.action_handler)(model);

            match result {
                Ok(data) => {
                    self.backend.set_action_result(data);
                }
                Err((msg, data)) => {
                    self.backend.set_action_fail(&msg);
                    self.backend.set_action_result(data);
                }
            }
        }
    }
}
