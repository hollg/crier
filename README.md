# crier

`crier` is a simple but flexible observer library for Rust.

## Goals
### Simplicity
`crier` has a simple API using basic Rust types. The complex types needed to make it work are all abstracted away from the consumer.

### Flexibility
- A `Publisher` can handle any number of different types of events and handlers.
- You have several options when creating handlers:
  - wrap a simple closure in the `Handler` struct
  - implement the `Handle` trait on your own type so that you have access to its other methods and state from the `handle` method
  - implement the `HandleMut` trait on your own type so that you have **mutable** access to its other methods and states from the `handle` method
  - mix and match all of the above

## Usage
### Subscribe a simple closure 
```rust
use crier::{Event,  Publisher};

#[derive(Clone, Event)]
struct Warning(String);

#[derive(Clone, Event)]
struct Info(String);

fn main() {
    let mut publisher = Publisher::default();
    let warning_id = publisher.subscribe_with(|warning: Warning| println!("Warning: {}", warning.0));

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
- [X] Support handlers that take a `mut &self` receiver to enable more complex use cases, e.g. updating some internal state when events are received

## Acknowledgements
Thanks to [@klaatu01](https://github.com/klaatu01/) for giving me the idea to build this and the coaching I needed to wrangle Rust's type system into allowing events and handlers of multiple different types in a single publisher.
