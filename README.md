# gawk

`gawk` is a simple but flexible observer library for Rust.

## Usage
There are two ways to subscribe to a `Producer` for events:
- Wrap a closure that takes the relevant `Event` as its argument in the `Handler` struct and subscribe that to the `Producer`
- Implement the `Handle` trait for your own type and subscribe that to the `Producer`

## Example
```rust

use gawk::{Event, Handler, Publisher};
use std::sync::Arc;

#[derive(Copy, Clone)]
struct Info(&'static str);
impl Event for Info {}

#[derive(Copy, Clone)]
struct Warning(&'static str);
impl Event for Warning {}

struct InfoHandler {}

impl Handle for InfoHandler {
    type Event = Info;

    fn handle(&self, event: Self::Event) {
        self.log(event);
    }
}


#[derive(Copy, Clone)]
struct Warning(&'static str);

impl Event for Warning {}

fn main() {
    let mut publisher = Publisher::default();

    let info_handler = InfoHandler {};
    let warning_handler = Handler::new(|warning: Warning| println!("Warning: {}", warning.0));

    let info_id = publisher.subscribe(Arc::new(info_handler));
    let warning_id = publisher.subscribe(Arc::new(warning_handler));

    let _ = publisher.publish(Arc::new(Warning("Looks sus")));
    let _ = publisher.publish(Arc::new(Info("All good"))); 

    publisher.unsubscribe(info_id);
    publisher.unsubscribe(warning_id);
}

```
