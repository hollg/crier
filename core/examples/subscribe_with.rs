use crier::{Event, Publisher};

#[derive(Clone, Event)]
struct Message(String);

fn main() {
    let mut publisher = Publisher::default();

    let handler_id =
        publisher.subscribe_with(|message: Message| println!("Message is: {}", message.0));

    let _ = publisher.publish(Message(String::from("Hello, world!")));

    publisher.unsubscribe(handler_id);
}
