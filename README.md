# gawk

`gawk` is a simple but flexible observer library for Rust.

## Goals
### Simplicity
`gawk` has a simple API using basic Rust types. The complex types needed to make it work are all abstracted away from the consumer.

### Flexibility
- A `gawk` `Publisher` can handle any number of different types of events and handlers.
- You can create a handler by wrapping a simple closure in `gawk`'s `Handler` struct, or you can implement the `Handle` trait manually on your own types. You can even do both for different handlers subscribed to the same `Publisher`.

## Usage
### Subscribe a simple closure 
```rust
use gawk::{Event, Handler, Publisher, Handle};

#[derive(Copy, Clone)]
struct Warning(&'static str);
impl Event for Warning {}

fn main() {
    let mut publisher = Publisher::default();
    let warning_handler = Handler::new(|warning: Warning| println!("Warning: {}", warning.0));
    let warning_id = publisher.subscribe(warning_handler);

    // `publish` returns a Result, the error variant of which contains any errors returned by triggered handlers
    let _ = publisher.publish(Warning("Looks sus"));

    publisher.unsubscribe(warning_id);
}

```

### Subscribe a custom type
```rust
use gawk::{Event, Handler, Publisher, Handle};

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
    let info_id = publisher.subscribe(info_handler);

    let _ = publisher.publish(Info("All good")); 

    publisher.unsubscribe(info_id);
}

```

## TODO:
- [X] Publisher supports any number of handlers / events of different types
- [ ] Optional async feature using Tokio
- [ ] Support handlers that take a `mut &self` receiver when implementing `Handle` for custom types to enable more complex use cases, e.g. updating some internal state when events are received

## Acknowledgements
Thanks to [@klaatu01](https://github.com/klaatu01/) for giving me the idea to build this and the coaching I needed to wrangle Rust's type system into allowing events and handlers of multiple different types in a single publisher.
