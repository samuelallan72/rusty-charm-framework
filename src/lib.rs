pub mod backend;
pub mod types;

use backend::{Backend, Logger, Ports, StatusManager, Unit};
use types::{Event, LogLevel};

pub struct EventModel<'a, B, C> {
    pub config: C,
    pub unit: Unit<'a, B>,
    pub ports: Ports<'a, B>,
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

pub struct ActionModel<'a, B, C> {
    backend: &'a B,
    pub config: C,
    pub unit: Unit<'a, B>,
    pub ports: Ports<'a, B>,
    pub status: StatusManager<'a, B>,
    pub log: Logger<'a, B>,
}

impl<'a, B, C> ActionModel<'a, B, C>
where
    B: Backend,
{
    pub fn action_log(&self, msg: &str) {
        self.backend.action_log(msg)
    }
}

pub struct Framework<A, B, C> {
    backend: B,
    event_handler: fn(EventModel<B, C>, Event) -> types::Status,
    action_handler: fn(ActionModel<B, C>, A) -> types::ActionResult,
}

impl<A, B, C> Framework<A, B, C>
where
    B: Backend,
    C: serde::de::DeserializeOwned,
    A: serde::de::DeserializeOwned,
{
    pub fn new(
        backend: B,
        event_handler: fn(EventModel<B, C>, Event) -> types::Status,
        action_handler: fn(ActionModel<B, C>, A) -> types::ActionResult,
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

            let model: EventModel<B, C> = EventModel::<B, C> {
                config: self.backend.config(),
                unit: Unit::load_from_backend(&self.backend),
                ports: Ports::load_from_backend(&self.backend),
                status: StatusManager::load_from_backend(&self.backend),
                log: Logger::load_from_backend(&self.backend),
            };

            (self.event_handler)(model, event);
            return;
        }

        let action_name = self.backend.action_name();
        if !action_name.is_empty() {
            self.backend.log(
                format!("running handler for {action_name} action").as_str(),
                LogLevel::Debug,
            );
            let action: A = self.backend.action(action_name.as_str());

            let model: ActionModel<B, C> = ActionModel::<B, C> {
                backend: &self.backend,
                config: self.backend.config(),
                unit: Unit::load_from_backend(&self.backend),
                ports: Ports::load_from_backend(&self.backend),
                status: StatusManager::load_from_backend(&self.backend),
                log: Logger::load_from_backend(&self.backend),
            };

            let result = (self.action_handler)(model, action);

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
