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

pub trait HandleMut {
    type EventType: Event;

    fn handle_mut(&mut self, event: Self::EventType) -> ();
}

pub trait DynHandleMut {
    fn dyn_handle_mut(&mut self, event: &dyn DynEvent) -> ();
}

impl<T, U> DynHandleMut for U
where
    T: Event,
    U: HandleMut<EventType = T> + Send + Sync + RefUnwindSafe,
{
    fn dyn_handle_mut(&mut self, event: &dyn DynEvent) {
        if let Some(event_data) = event.get_data().downcast_ref::<T>() {
            self.handle_mut(event_data.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Debug, PartialEq)]
    struct MyEvent(pub i32);

    impl Event for MyEvent {}

    #[derive(Clone, Debug)]
    struct OtherEvent;

    impl Event for OtherEvent {}

    struct TestHandle {
        called: Arc<Mutex<bool>>,
    }

    impl Handle for TestHandle {
        type EventType = MyEvent;
        fn handle(&self, event: MyEvent) {
            assert_eq!(event.0, 99);
            *self.called.lock().unwrap() = true;
        }
    }

    #[test]
    fn test_dyn_handle_calls_handle_on_matching_type() {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        let handler = Handler::new(move |event: MyEvent| {
            assert_eq!(event.0, 42);
            *called_clone.lock().unwrap() = true;
        });

        let event = MyEvent(42);
        handler.dyn_handle(&event);

        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_dyn_handle_does_not_call_handle_on_non_matching_type() {
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        let handler = Handler::new(move |_event: MyEvent| {
            *called_clone.lock().unwrap() = true;
        });

        let other_event = OtherEvent;
        handler.dyn_handle(&other_event);

        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn test_dyn_handle_for_handle_impl_matching_type() {
        let called = Arc::new(Mutex::new(false));
        let handler = TestHandle {
            called: called.clone(),
        };

        let event = MyEvent(99);
        DynHandle::dyn_handle(&handler, &event);

        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_dyn_handle_for_handle_impl_non_matching_type() {
        let called = Arc::new(Mutex::new(false));
        let handler = TestHandle {
            called: called.clone(),
        };

        let other_event = OtherEvent;
        DynHandle::dyn_handle(&handler, &other_event);

        assert!(!*called.lock().unwrap());
    }
}
