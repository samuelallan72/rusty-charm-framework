// This is a test charm using the charm framework.
// No framework lib code should go here.
#![allow(dead_code)]
#![allow(unused_variables)]
use rusty_charm_framework::{
    action, execute, log,
    status::{self, Status},
    ActionResult, Event, State,
};
use serde::Deserialize;
use std::{thread, time};

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
#[serde(rename_all_fields(deserialize = "kebab-case"))]
enum Action {
    Test {
        name: Option<String>,
        dry_run: bool,
        param_with_default: String,
    },
    // NOTE: currently with the way of building an intermediate representation of the action
    // before deserialising, we must use struct variants, not bare variants (eg. `Log` is not
    // possible, but `Log {}` is fine).
    Log {},
}

#[derive(Deserialize)]
struct Config {
    /// The region name
    region: String,
}

fn event_handler(state: State<Config>, event: Event) -> Status {
    log::info(format!("region config = {}", state.config.region).as_str());
    match event {
        Event::UpdateStatus => {
            if state.config.region.is_empty() {
                return Status::Blocked("region option cannot be empty".to_string());
            } else {
                return Status::Active("".to_string());
            }
        }
        Event::Install => {
            status::active("hi".to_owned());
        }
        _ => {}
    }

    return Status::Active("all good (probably)".to_string());
}

fn action_handler(state: State<Config>, action_: Action) -> ActionResult {
    log::debug(&format!("deserialised action: {:?}", action_));
    match action_ {
        Action::Test {
            name,
            dry_run,
            param_with_default,
        } => todo!(),
        Action::Log {} => {
            action::log("Logging a message at the beginning of the handler.");

            action::log("Sleeping for 1 second");
            thread::sleep(time::Duration::from_secs(1));

            action::log("Sleeping for another second");
            thread::sleep(time::Duration::from_secs(1));

            action::log("Done!");

            ActionResult::Success
        }
    }
}

fn main() {
    execute(event_handler, action_handler);
}
