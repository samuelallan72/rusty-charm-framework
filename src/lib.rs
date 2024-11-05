#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
// TODO: relations
// TODO: logging
// TODO: encode all metadata.yaml content in the Framework
// TODO: figure out error handling

pub mod log;
pub mod status;

use std::process::Command;

use serde_json;

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

pub enum Status {
    Active(String),
    Blocked(String),
    Error,
    Waiting(String),
}

// TODO: drop the unwraps, figure out error handling/propogating
fn config<C>() -> C
where
    C: serde::de::DeserializeOwned,
{
    let output = Command::new("config-get")
        .args(["--format", "json", "--all"])
        .output()
        .expect("failed to execute config-get");
    serde_json::from_slice::<C>(&output.stdout).unwrap()
}

/// Process the current event, hook, or action from the environment,
/// populating local state, and calling the handler functions as appropriate.
pub fn execute<C, A>(
    event_handler: fn(State<C>, Event) -> Status,
    action_handler: fn(State<C>, A) -> ActionResult,
) where
    C: serde::de::DeserializeOwned,
{
    // Print all environment variables.
    for (key, value) in std::env::vars() {
        log::debug(format!("{key}: {value}").as_str());
    }

    let hook = std::env::var("JUJU_HOOK_NAME").expect("JUJU_HOOK_NAME unexpectedly unset");
    log::info(format!("running handlers for {hook} hook").as_str());

    let event = match hook.as_str() {
        "install" => Event::Install,
        "config-changed" => Event::ConfigChanged,
        "remove" => Event::Remove,
        "update-status" => Event::UpdateStatus,
        "upgrade-charm" => Event::UpgradeCharm,
        _ => unimplemented!(),
    };

    let state: State<C> = State::<C> { config: config() };
    event_handler(state, event);

    // TODO: action_handler(state, todo!());
}

// TODO: macro to write the config.yaml, etc. to file at compile time,
// so the code is the source of truth.
// #[proc_macro_attribute]
// pub fn write_config(_args: TokenStream, input: TokenStream) -> TokenStream  {
//     todo!()
// }
