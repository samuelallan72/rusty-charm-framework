#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
// TODO: relations
// TODO: logging
// TODO: encode all metadata.yaml content in the Framework
// TODO: figure out error handling

pub mod log;
pub mod status;

use std::collections::HashMap;

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

pub struct State<C> {
    pub config: C,
    pub relations: HashMap<String, String>,
    // TODO: application data bag, unit data bag
}

pub enum ActionResult {
    Success,
    Failure,
}

pub enum Status {
    Active(String),
    Blocked(String),
    Error,
    Waiting(String),
}

/// Process the current event, hook, or action from the environment,
/// populating local state, and calling the handler functions as appropriate.
pub fn execute<C, A>(
    event_handler: fn(State<C>, Event) -> Status,
    action_handler: fn(State<C>, A) -> ActionResult,
) {
    // Print all environment variables.
    for (key, value) in std::env::vars() {
        log::debug(format!("{key}: {value}").as_str());
    }

    let hook = std::env::var("JUJU_HOOK_NAME").expect("JUJU_HOOK_NAME unexpectedly unset");
    log::info(format!("running handlers for {hook} hook").as_str());

    status::active("hi");

    // let state: State<C> = todo!();
    // let event: Option<Event> = todo!();
    // if let Some(event) = event {
    //     event_handler(state, event);
    // } else {
    //     action_handler(state, todo!());
    // }
}

// TODO: macro to write the config.yaml, etc. to file at compile time,
// so the code is the source of truth.
// #[proc_macro_attribute]
// pub fn write_config(_args: TokenStream, input: TokenStream) -> TokenStream  {
//     todo!()
// }
