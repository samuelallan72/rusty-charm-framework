// This is a test charm using the charm framework.
// No framework lib code should go here.
#![allow(dead_code)]
#![allow(unused_variables)]
use rusty_charm_framework::{
    backend::{Backend, RealBackend},
    types::{ActionResult, Event, Status},
    Framework, Model,
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

fn event_handler(model: Model<impl Backend, Config>, event: Event) -> Status {
    model
        .backend
        .info(format!("region config = {}", model.config.region).as_str());
    match event {
        Event::UpdateStatus => {
            if model.config.region.is_empty() {
                return Status::Blocked("region option cannot be empty");
            } else {
                return Status::Active("");
            }
        }
        Event::Install => {
            model.backend.active("hi");
        }
        _ => {}
    }

    return Status::Active("all good (probably)");
}

fn action_handler(model: Model<impl Backend, Config>, action: Action) -> ActionResult {
    model
        .backend
        .debug(&format!("deserialised action: {:?}", action));
    match action {
        Action::Test {
            name,
            dry_run,
            param_with_default,
        } => todo!(),
        Action::Log {} => {
            model
                .backend
                .action_log("Logging a message at the beginning of the handler.");

            model.backend.action_log("Sleeping for 1 second");
            thread::sleep(time::Duration::from_secs(1));

            model.backend.action_log("Sleeping for another second");
            thread::sleep(time::Duration::from_secs(1));

            model.backend.action_log("Done!");

            ActionResult::Success
        }
    }
}

fn main() {
    // dependency injection for the framework for easier unit testing
    let charm = Framework::new(RealBackend {}, event_handler, action_handler);
    charm.execute();
}
