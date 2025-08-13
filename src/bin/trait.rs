use gawk::{Event, Handle, Publisher};

#[derive(Copy, Clone)]
struct Info(&'static str);

impl Event for Info {}

struct InfoHandler {}

impl InfoHandler {
    fn log(&self, event: Info) {
        println!("Info: {}", event.0)
    }
}

impl Handle for InfoHandler {
    type EventType = Info;

    fn handle(&self, event: Self::EventType) {
        self.log(event);
    }
}

fn main() {
    let mut publisher = Publisher::default();

    let info_handler = InfoHandler {};
    let handler_id = publisher.subscribe(info_handler);

    let _ = publisher.publish(Info("All good"));

    publisher.unsubscribe(handler_id);
}
