use crier::{Event, HandleMut, Publisher};

#[derive(Clone, Event)]
struct Info(String);

#[derive(Default)]
struct InfoHandler {
    counter: usize,
}

impl InfoHandler {
    fn log(&mut self, event: Info) {
        self.counter += 1;
        println!("Info: {}", event.0);
        println!("Count: {}", self.counter);
    }
}

impl HandleMut for InfoHandler {
    type EventType = Info;

    fn handle_mut(&mut self, event: Self::EventType) {
        self.log(event);
    }
}

fn main() {
    let mut publisher = Publisher::default();

    let info_handler = InfoHandler::default();
    let handler_id = publisher.subscribe_mut(info_handler);

    let _ = publisher.publish(Info(String::from("All good")));

    publisher.unsubscribe(handler_id);
}
