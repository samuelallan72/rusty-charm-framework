use rusty_charm_framework::{ActionResult, Event, Framework, State, Status};

enum Action {
    Validate { regex: String, serial: bool },
    GetLists,
}

struct Config {
    region: String,
}

fn event_handler(state: State<Config>, event: Event) -> Status {
    match event {
        Event::UpdateStatus => {
            if state.config.region.is_empty() {
                return Status::Blocked("region option cannot be empty".to_string());
            } else {
                return Status::Active("".to_string());
            }
        },
        Event::Install => todo!(),
    }
}

fn action_handler(state: State<Config>, action: Action) -> ActionResult {
    match action {
        Action::Validate { regex, serial } => todo!(),
        Action::GetLists => todo!(),
    }
}

fn main() {
    let framework = Framework::new(event_handler, action_handler);

    framework.run();
}
