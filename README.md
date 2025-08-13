# gawk

`gawk` is a simple but flexible observer library for Rust.


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

fn main() {
    let mut publisher = Publisher::default();

    let info_handler = Handler::new(|info: Info| println!("Info: {}", info.0));
    let warning_handler = Handler::new(|warning: Warning| println!("Warning: {}", warning.0));

    let info_id = publisher.subscribe(Arc::new(info_handler));
    let warning_id = publisher.subscribe(Arc::new(warning_handler));

    publisher.publish(Arc::new(Warning("Looks sus")));
    publisher.publish(Arc::new(Info("All good")));

    publisher.unsubscribe(info_id);
    publisher.unsubscribe(warning_id);
}

```
