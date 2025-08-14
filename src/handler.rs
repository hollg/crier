use std::panic::RefUnwindSafe;

use crate::{DynEvent, Event};

/// Trait for an object which can subscribe to a Producer for specific events
pub trait Handle {
    type EventType: Event;

    fn handle(&self, event: Self::EventType) -> ();
}

/// Wrapper for code that handles Events of a specific type.
pub struct Handler<T: Event> {
    /// Complex seeming type allows closures?
    handle: Box<dyn Fn(T) + Send + Sync>,
}

impl<T: Event> RefUnwindSafe for Handler<T> {}

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
pub trait DynHandle: Send + Sync + RefUnwindSafe {
    fn dyn_handle(&self, event: &dyn DynEvent) -> ();
}

// Allow Handler to take any DynEvent object and decide whether to run its handle method.
// This is what enables the Publisher to take handlers and events of any type — as long as they are
// all DynHandler and DynEvent, the handler can decide whether to handle the event
impl<T: Event> DynHandle for Handler<T> {
    fn dyn_handle(&self, event: &dyn DynEvent) {
        if let Some(event_data) = event.get_data().downcast_ref::<T>() {
            (self.handle)(event_data.clone())
        }
    }
}

// Allow any Handle object to take any DynEvent object and decide whether to run its handle method.
// This is what enables the Publisher to take handlers and events of any type — as long as they are
// all DynHandler and DynEvent, the handler can decide whether to handle the event
impl<T, U> DynHandle for U
where
    T: Event,
    U: Handle<EventType = T> + Send + Sync + RefUnwindSafe,
{
    fn dyn_handle(&self, event: &dyn DynEvent) {
        if let Some(event_data) = event.get_data().downcast_ref::<T>() {
            self.handle(event_data.clone())
        }
    }
}
