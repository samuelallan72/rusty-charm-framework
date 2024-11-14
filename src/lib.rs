pub mod backend;
pub mod model;
pub mod types;

use backend::Backend;
use model::{ActionModel, EventModel};
use types::{Event, LogLevel};

pub struct Framework<A, B> {
    backend: B,
    event_handler: fn(EventModel<B>) -> types::Status,
    action_handler: fn(ActionModel<A, B>) -> types::ActionResult,
}

impl<A, B> Framework<A, B>
where
    B: Backend,
    A: serde::de::DeserializeOwned,
{
    pub fn new(
        backend: B,
        event_handler: fn(EventModel<B>) -> types::Status,
        action_handler: fn(ActionModel<A, B>) -> types::ActionResult,
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
                "collect-metrics" => Event::CollectMetrics,
                "config-changed" => Event::ConfigChanged,
                "install" => Event::Install,
                "remove" => Event::Remove,
                "update-status" => Event::UpdateStatus,
                "upgrade-charm" => Event::UpgradeCharm,
                "leader-elected" => Event::LeaderElected,
                "leader-settings-changed" => Event::LeaderSettingsChanged,
                "pebble-custom-notice" => Event::PebbleCustomNotice,
                "pre-series-upgrade" => Event::PreSeriesUpgrade,
                "post-series-upgrade" => Event::PostSeriesUpgrade,
                "secret-changed" => Event::SecretChanged,
                "secret-expire" => Event::SecretExpire,
                "secret-removed" => Event::SecretRemoved,
                "secret-rotate" => Event::SecretRotate,
                "start" => Event::Start,
                "stop" => Event::Stop,
                name => {
                    if let Some(prefix) = name.strip_suffix("-relation-joined") {
                        Event::RelationJoined(prefix.to_owned())
                    } else if let Some(prefix) = name.strip_suffix("-relation-broken") {
                        Event::RelationBroken(prefix.to_owned())
                    } else if let Some(prefix) = name.strip_suffix("-relation-changed") {
                        Event::RelationChanged(prefix.to_owned())
                    } else if let Some(prefix) = name.strip_suffix("-relation-created") {
                        Event::RelationCreated(prefix.to_owned())
                    } else if let Some(prefix) = name.strip_suffix("-relation-departed") {
                        Event::RelationDeparted(prefix.to_owned())
                    } else if let Some(prefix) = name.strip_suffix("-storage-attached") {
                        Event::StorageAttached(prefix.to_owned())
                    } else if let Some(prefix) = name.strip_suffix("-storage-detached") {
                        Event::StorageDetached(prefix.to_owned())
                    } else {
                        // TODO: proper error handling
                        unimplemented!();
                    }
                }
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
