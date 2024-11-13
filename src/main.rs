// This is a test charm using the charm framework.
// No framework lib code should go here.
use rusty_charm_framework::{
    backend::{Backend, JujuBackend},
    types::{ActionResult, ActionResultKey, ActionValue, Event, Status},
    ActionModel, EventModel, Framework,
};
use serde::Deserialize;
use std::{collections::HashMap, thread, time};

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
#[serde(rename_all_fields(deserialize = "kebab-case"))]
enum Action {
    EchoParams {
        string: Option<String>,
        string_with_default: String,
        fail: bool,
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

fn event_handler(model: EventModel<impl Backend, Config>, event: Event) -> Status {
    model
        .log
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
            model.status.active("Install hook completed");
        }
        _ => {}
    }

    return Status::Active("all good (probably)");
}

fn action_handler(model: ActionModel<impl Backend, Config>, action: Action) -> ActionResult {
    model
        .log
        .debug(&format!("deserialised action: {:?}", action));
    match action {
        Action::Log {} => {
            model.action_log("Logging a message at the beginning of the handler.");

            model.action_log("Sleeping for 1 second");
            thread::sleep(time::Duration::from_secs(1));

            model.action_log("Sleeping for another second");
            thread::sleep(time::Duration::from_secs(1));

            model.action_log("Done!");

            Ok(HashMap::new())
        }
        Action::EchoParams {
            string,
            string_with_default,
            fail,
        } => {
            model.action_log(&format!("string = {:?}", string));
            model.action_log(&format!("string-with-default = {:?}", string_with_default));
            model.action_log(&format!("fail = {:?}", fail));

            let mut data = HashMap::new();

            data.insert(
                ActionResultKey::try_from("params".to_owned()).unwrap(),
                ActionValue::Nested({
                    let mut inner = HashMap::new();
                    inner.insert(
                        ActionResultKey::try_from("string-with-default".to_owned()).unwrap(),
                        ActionValue::Value(string_with_default),
                    );
                    if let Some(string) = string {
                        inner.insert(
                            ActionResultKey::try_from("string".to_owned()).unwrap(),
                            ActionValue::Value(string),
                        );
                    }
                    inner.insert(
                        ActionResultKey::try_from("fail".to_owned()).unwrap(),
                        ActionValue::Value(format!("{fail}")),
                    );
                    inner
                }),
            );

            data.insert(
                ActionResultKey::try_from("example-nesting".to_owned()).unwrap(),
                ActionValue::Nested({
                    let mut inner = HashMap::new();
                    inner.insert(
                        ActionResultKey::try_from("nested".to_owned()).unwrap(),
                        ActionValue::Nested({
                            let mut inner2 = HashMap::new();
                            inner2.insert(
                                ActionResultKey::try_from("level2".to_owned()).unwrap(),
                                ActionValue::Value("example value 2".to_owned()),
                            );
                            inner2
                        }),
                    );
                    inner.insert(
                        ActionResultKey::try_from("level1".to_owned()).unwrap(),
                        ActionValue::Value("example value 1".to_owned()),
                    );
                    inner
                }),
            );

            if fail {
                Err(("this is the requested failure message".to_owned(), data))
            } else {
                Ok(data)
            }
        }
    }
}

fn main() {
    // dependency injection for the framework for easier unit testing
    let charm = Framework::new(JujuBackend {}, event_handler, action_handler);
    charm.execute();
}
