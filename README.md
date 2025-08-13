# gawk

`gawk` is a simple but flexible observer library for Rust.

## Usage
There are two ways to subscribe to a `Producer` for events:
- Wrap a closure that takes the relevant `Event` as its argument in the `Handler` struct and subscribe that to the `Producer`
- Implement the `Handle` trait for your own type and subscribe that to the `Producer`

## Example
```rust

use gawk::{Event, Handler, Publisher, Handle};

#[derive(Copy, Clone)]
struct Info(&'static str);
impl Event for Info {}

#[derive(Copy, Clone)]
struct Warning(&'static str);
impl Event for Warning {}

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
    let warning_handler = Handler::new(|warning: Warning| println!("Warning: {}", warning.0));

    let info_id = publisher.subscribe(info_handler);
    let warning_id = publisher.subscribe(warning_handler);

    let _ = publisher.publish(Warning("Looks sus"));
    let _ = publisher.publish(Info("All good")); 

    publisher.unsubscribe(info_id);
    publisher.unsubscribe(warning_id);
}

```
