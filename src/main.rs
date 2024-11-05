// This is a test charm using the charm framework.
// No framework lib code should go here.
#![allow(dead_code)]
#![allow(unused_variables)]
use rusty_charm_framework::{execute, log, status, status::Status, ActionResult, Event, State};

enum Action {
    Test {
        name: String,
        dry_run: bool,
        param_with_default: String,
    },
    Noop,
}

use serde::Deserialize;

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

fn action_handler(state: State<Config>, action: Action) -> ActionResult {
    match action {
        Action::Test {
            name,
            dry_run,
            param_with_default,
        } => todo!(),
        Action::Noop => todo!(),
    }
}

fn main() {
    execute(event_handler, action_handler);
}
