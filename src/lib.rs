#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
// TODO: relations
// TODO: logging
// TODO: encode all metadata.yaml content in the Framework
// TODO: figure out error handling

pub mod action;
pub mod config;
pub mod log;
pub mod status;

// ref. https://github.com/canonical/charm-events
pub enum Event {
    CollectMetrics,
    ConfigChanged,
    Install,
    LeaderElected,
    LeaderSettingsChanged,
    PebbleCustomNotice,
    PebbleReady(String),
    PostSeriesUpgrade,
    PreSeriesUpgrade,
    RelationBroken(String),
    RelationChanged(String),
    RelationCreated(String),
    RelationDeparted(String),
    RelationJoined(String),
    Remove,
    SecretChanged,
    SecretExpire,
    SecretRemoved,
    SecretRotate,
    Start,
    Stop,
    StorageAttached(String),
    StorageDetatched(String),
    UpdateStatus,
    UpgradeCharm,
}

// TODO: application data bag, unit data bag, relations, etc.
pub struct State<C> {
    pub config: C,
}

pub enum ActionResult {
    Success,
    Failure,
}

/// Process the current event, hook, or action from the environment,
/// populating local state, and calling the handler functions as appropriate.
/// `event_handler` must return a status - this status will be the final status set before
/// execution of the hook finishes.
/// The event handler may explicitly set a status during execution.
/// This may be useful in the case of a long running hook (eg. set a maintenance ongoing status at
/// the beginning).
pub fn execute<C, A>(
    event_handler: fn(State<C>, Event) -> status::Status,
    action_handler: fn(State<C>, A) -> ActionResult,
) where
    C: serde::de::DeserializeOwned,
{
    // debug log all env vars for testing purposes
    for (key, value) in std::env::vars() {
        log::debug(format!("{key}: {value}").as_str());
    }

    let state: State<C> = State::<C> {
        config: config::config(),
    };

    // ref. https://juju.is/docs/juju/charm-environment-variables for logic
    let hook = std::env::var("JUJU_HOOK_NAME").unwrap_or("".to_owned());
    if !hook.is_empty() {
        log::info(format!("running handlers for {hook} hook").as_str());

        let event = match hook.as_str() {
            "install" => Event::Install,
            "config-changed" => Event::ConfigChanged,
            "remove" => Event::Remove,
            "update-status" => Event::UpdateStatus,
            "upgrade-charm" => Event::UpgradeCharm,
            _ => unimplemented!(),
        };

        event_handler(state, event);
        return;
    }

    let action = std::env::var("JUJU_ACTION_NAME").unwrap_or("".to_owned());
    if !action.is_empty() {
        log::info(format!("running handler for {action} action").as_str());

        // TODO: figure out how to deserialise action name + params into the user-provided
        // `A` action enum.
        log::info(format!("{}", action::params()).as_str());
        action_handler(state, todo!());
        return;
    }
}

// TODO: macro to write the config.yaml, etc. to file at compile time,
// so the code is the source of truth.
// #[proc_macro_attribute]
// pub fn write_config(_args: TokenStream, input: TokenStream) -> TokenStream  {
//     todo!()
// }
