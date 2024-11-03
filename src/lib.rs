use std::collections::HashMap;

pub enum Event {
    UpdateStatus,
    Install,
}

pub struct State<C> {
    pub config: C,
    pub relations: HashMap<String, String>,
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

pub struct Framework<C, A> {
    event_handler: fn(State<C>, Event) -> Status,
    action_handler: fn(State<C>, A) -> ActionResult,
}

impl<C, A> Framework<C, A> {
    pub fn new(event_handler: fn(State<C>, Event) -> Status, action_handler: fn(State<C>, A) -> ActionResult) -> Self {
        Self {
            event_handler,
            action_handler,
        }
    }

    pub fn run(&self) {
        let state: State<C> = todo!();
        let event: bool = todo!();
        if event {
            (self.event_handler)(state, Event::Install);
        } else {
            (self.action_handler)(state, todo!());
        }
    }
}
