// This is a test charm using the charm framework.
// No framework lib code should go here.
use rusty_charm_framework::{
    backend::{Backend, JujuBackend},
    types::{ActionResult, Event, Status},
    Framework, Model,
};
use serde::Deserialize;
use std::{thread, time};

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
#[serde(rename_all_fields(deserialize = "kebab-case"))]
enum Action {
    EchoParams {
        string: Option<String>,
        string_with_default: String,
        bool: bool,
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
            model.backend.active("Install hook completed");
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
        Action::EchoParams {
            string,
            string_with_default,
            bool,
        } => {
            model.backend.action_log(&format!("string = {:?}", string));
            model
                .backend
                .action_log(&format!("string-with-default = {:?}", string_with_default));
            model.backend.action_log(&format!("bool = {:?}", bool));
            ActionResult::Success
        }
    }
}

fn main() {
    // dependency injection for the framework for easier unit testing
    let charm = Framework::new(JujuBackend {}, event_handler, action_handler);
    charm.execute();
}
