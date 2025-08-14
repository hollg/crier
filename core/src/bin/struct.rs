use gawk::{Event, Handler, Publisher};

#[allow(dead_code)]
#[derive(Clone, Event)]
struct Info(String);

#[derive(Clone, Event)]
struct Warning(String);

fn main() {
    let mut publisher = Publisher::default();

    let warning_handler = Handler::new(|info: Warning| println!("Warning: {}", info.0));

    let warning_id = publisher.subscribe(warning_handler);

    let _ = publisher.publish(Warning(String::from("Looks sus")));

    // This event will be ignored by the
    // warning_handler
    let _ = publisher.publish(Info(String::from("All good")));

    publisher.unsubscribe(warning_id);
}
