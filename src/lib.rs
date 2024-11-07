#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
// TODO: relations
// TODO: encode all metadata.yaml content in the Framework
// TODO: figure out error handling

pub mod backend;
pub mod types;

use types::Event;

// TODO: application data bag, unit data bag, relations, etc.
pub struct Model<'a, B, C> {
    pub config: C,
    pub backend: &'a B,
}

pub struct Framework<A, B, C> {
    backend: B,
    event_handler: fn(Model<B, C>, Event) -> types::Status,
    action_handler: fn(Model<B, C>, A) -> types::ActionResult,
}

impl<A, B, C> Framework<A, B, C>
where
    B: backend::Backend,
    C: serde::de::DeserializeOwned,
    A: serde::de::DeserializeOwned,
{
    pub fn new(
        backend: B,
        event_handler: fn(Model<B, C>, Event) -> types::Status,
        action_handler: fn(Model<B, C>, A) -> types::ActionResult,
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
            self.backend.debug(format!("{key}: {value}").as_str());
        }

        let state: Model<B, C> = Model::<B, C> {
            config: self.backend.config(),
            // TODO: the backend that the charm gets should be a smaller scoped version
            // of the full backend - eg. with config() and action() removed - maybe
            // a wrapper trait that passes through to the real backend?
            // Or is that unnecessary? How does this work with mocking for unit tests?
            backend: &self.backend,
        };

        // ref. https://juju.is/docs/juju/charm-environment-variables for logic
        let hook = std::env::var("JUJU_HOOK_NAME").unwrap_or("".to_owned());
        if !hook.is_empty() {
            self.backend
                .info(format!("running handlers for {hook} hook").as_str());

            let event = match hook.as_str() {
                "install" => Event::Install,
                "config-changed" => Event::ConfigChanged,
                "remove" => Event::Remove,
                "update-status" => Event::UpdateStatus,
                "upgrade-charm" => Event::UpgradeCharm,
                _ => unimplemented!(),
            };

            (self.event_handler)(state, event);
            return;
        }

        let action_name = std::env::var("JUJU_ACTION_NAME").unwrap_or("".to_owned());
        if !action_name.is_empty() {
            self.backend
                .debug(format!("running handler for {action_name} action").as_str());
            let action: A = self.backend.action(action_name.as_str());
            (self.action_handler)(state, action);
        }
    }
}

// TODO: macro to write the config.yaml, etc. to file at compile time,
// so the code is the source of truth.
// #[proc_macro_attribute]
// pub fn write_config(_args: TokenStream, input: TokenStream) -> TokenStream  {
//     todo!()
// }
