use gawk::{Event, Handle, Publisher};
use std::sync::Arc;

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
    type Event = Info;

    fn handle(&self, event: Self::Event) {
        self.log(event);
    }
}

fn main() {
    let mut publisher = Publisher::default();

    let info_handler = InfoHandler {};
    let handler_id = publisher.subscribe(Arc::new(info_handler));

    let _ = publisher.publish(Arc::new(Info("All good")));

    publisher.unsubscribe(handler_id);
}
