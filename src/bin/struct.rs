use gawk::{Event, Handler, Publisher};

#[allow(dead_code)]
#[derive(Copy, Clone)]
struct Info(&'static str);

impl Event for Info {}

#[derive(Copy, Clone)]
struct Warning(&'static str);

impl Event for Warning {}

fn main() {
    let mut publisher = Publisher::default();

    let warning_handler = Handler::new(|info: Warning| println!("Warning: {}", info.0));

    let warning_id = publisher.subscribe(warning_handler);

    let _ = publisher.publish(Warning("Looks sus"));
    let _ = publisher.publish(Info("All good")); // This event will be ignored

    publisher.unsubscribe(warning_id);
}
