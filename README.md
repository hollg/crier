# crier

`crier` is a simple but flexible observer library for Rust.

## Goals
### Simplicity
`crier` has a simple API using basic Rust types. The complex types needed to make it work are all abstracted away from the consumer.

### Flexibility
- A `crier` `Publisher` can handle any number of different types of events and handlers.
- You can create a handler by wrapping a simple closure in `crier`'s `Handler` struct, or you can implement the `Handle` trait manually on your own types. You can even do both for different handlers subscribed to the same `Publisher`.

## Usage
### Subscribe a simple closure 
```rust
use crier::{Event, Handler, Publisher};

#[derive(Clone, Event)]
struct Warning(String);

#[derive(Clone, Event)]
struct Info(String);

fn main() {
    let mut publisher = Publisher::default();
    let warning_handler = Handler::new(|warning: Warning| println!("Warning: {}", warning.0));
    let warning_id = publisher.subscribe(warning_handler);

    // `publish` returns a Result, the error variant of which contains any errors returned by triggered handlers
    let _ = publisher.publish(Warning(String::from("Looks sus")));

    // This event will not trigger the warning_handler because it's of the wrong concrete type
    let _ = publisher.publish(Info(String::from("All good")));

    publisher.unsubscribe(warning_id);
}

```

### Subscribe a custom type with a mut handler function
```rust
use crier::{Event, Publisher, HandleMut};

#[derive(Clone, Event)]
struct Info(String);

#[derive(Default)]
struct InfoHandler {
    count: usize;
}

impl HandleMut for InfoHandler {
    type EventType = Info;

    fn handle_mut(&mut self, event: Self::EventType) {
        self.count += 1;
        println!("Info: {}", event.0)
        println!("Triggered {} times", self.count);
    }
}

fn main() {
    let mut publisher = Publisher::default();
    let info_handler = InfoHandler::default();
    let info_id = publisher.subscribe_mut(info_handler);

    let _ = publisher.publish(Info(String::from("All good"))); 

    publisher.unsubscribe_mut(info_id);
}
```

Check out the examples directory for more!

## TODO:
- [X] Publisher supports any number of handlers / events of different types
- [X] Derive macro for `Event` trait
- [ ] Optional async feature using Tokio
- [X] Support handlers that take a `mut &self` receiver when implementing `Handle` for custom types to enable more complex use cases, e.g. updating some internal state when events are received

## Acknowledgements
Thanks to [@klaatu01](https://github.com/klaatu01/) for giving me the idea to build this and the coaching I needed to wrangle Rust's type system into allowing events and handlers of multiple different types in a single publisher.
