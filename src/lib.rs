use std::any;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

/// An object that a Publisher can send to its subscribers
pub trait Event: Send + Sync + Clone + 'static {}

/// Dynamically typed event. Used internally to alow Publishers to support Handlers and Events of
/// multiple different types.
pub trait DynEvent: Send + Sync + 'static {
    fn get_data(&self) -> &dyn any::Any;
}

impl<T: Event> DynEvent for T {
    fn get_data(&self) -> &dyn any::Any {
        self
    }
}

/// Wrapper for code that handles Events of a specific type.
pub struct Handler<T: Event> {
    /// Complex seeming type allows closures?
    handle: Box<dyn Fn(T) + Send + Sync>,
}

impl<T: Event> Handler<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        Handler {
            handle: Box::new(f),
        }
    }
}

/// Dynamically typed Handler. Used internally to allow Publishers to support Events and Handlers
/// of multiple different types.
pub trait DynHandler: Send + Sync {
    fn dyn_handle(&self, _event: &dyn DynEvent) {}
}

impl<T: Event> DynHandler for Handler<T> {
    fn dyn_handle(&self, event: &dyn DynEvent) {
        if let Some(event_data) = event.get_data().downcast_ref::<T>() {
            (self.handle)(event_data.clone())
        }
    }
}

/// Publishes all Events to all subscribed Handlers that accept Events of that type
/// # Examples
/// ```
/// use gawk::{Event, Handler};
///
/// #[derive(Copy, Clone)]
/// struct GamePaused {}
///
/// impl Event for GamePaused {}
///
/// let pause_handler = Handler::new(|_event: GamePaused| println!("Game paused"));
/// let publisher = Publisher::default();
/// let pause_handler_id = publisher.subscribe(Arc::new(pause_handler));
///
/// publisher.publish(Arc::new(GamePaused {}));
///
/// publisher.unsubscribe(pause_handler_id);
///
/// ```
#[derive(Default)]
pub struct Publisher {
    handler_count: usize,
    // we use Arc so that a reference to the handler can be passed to other threads for
    // execution
    handlers: HashMap<usize, Arc<dyn DynHandler>>,
}

impl Publisher {
    /// Subscribe a handler to the publisher so that the handler receives all published events.
    /// Returns the ID needed to `unsubscribe` the handler.
    pub fn subscribe(&mut self, handler: Arc<dyn DynHandler>) -> usize {
        let id = self.handler_count + 1;
        self.handlers.insert(id, handler);
        self.handler_count = id;

        id
    }

    /// Remove a handler from the publisher so that it stops receiving events
    pub fn unsubscribe(&mut self, id: usize) {
        self.handlers.remove_entry(&id);
    }

    /// Publish an event to all subscribed handlers, utilizing as many threads as possible to run
    /// handlers in parallel
    pub fn publish(&self, event: Arc<dyn DynEvent>) {
        let max_threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        thread::scope(|s| {
            let mut handles = Vec::new();
            for handler in self.handlers.values() {
                let handler = Arc::clone(handler);
                let cloned_event = event.clone();

                handles.push(s.spawn(move || handler.dyn_handle(cloned_event.as_ref())));

                if handles.len() == max_threads {
                    for handle in handles.drain(..) {
                        handle.join().unwrap();
                    }
                }
            }
        });
    }
}
