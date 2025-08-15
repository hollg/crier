use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use crate::{DynEvent, DynHandle, DynHandleMut, Event, Handler};

/// Publishes all Events to all subscribed Handlers that accept Events of that type
/// # Examples
/// ```
/// use crier::{Event, Handler, Publisher};
///
/// #[derive(Clone, Event)]
/// struct GamePaused {}
///
///
/// let mut publisher = Publisher::default();
/// let pause_handler = Handler::new(|_event: GamePaused| println!("Game paused"));
/// let pause_handler_id = publisher.subscribe(pause_handler);
///
/// publisher.publish(GamePaused {});
///
/// publisher.unsubscribe(pause_handler_id);
///
/// ```
#[derive(Default)]
pub struct Publisher {
    handler_count: usize,
    handlers: HashMap<usize, HandlerType>,
}

/// Represents the different types of handler and how they are stored in the `handlers` map on the
/// Publisher
enum HandlerType {
    Sync(Arc<dyn DynHandle>),
    SyncMut(Arc<Mutex<dyn DynHandleMut>>),
}

impl Publisher {
    /// Subscribe a handler to the publisher so that the handler receives all published events.
    /// Returns the ID needed to `unsubscribe` the handler.
    pub fn subscribe<T>(&mut self, handler: T) -> usize
    where
        T: DynHandle + 'static,
    {
        let handler: Arc<dyn DynHandle> = Arc::new(handler);
        let id = self.handler_count + 1;
        // self.handlers.insert(id, handler);
        self.handlers.insert(id, HandlerType::Sync(handler));
        self.handler_count = id;

        id
    }

    // Subscribe a closure to events of its input type.
    // Returns the ID needed to `unsubscribe` the handler.
    pub fn subscribe_with<T, F>(&mut self, handler: F) -> usize
    where
        T: Event,
        F: Fn(T) + Send + Sync + 'static,
    {
        let wrapped = Handler::new(handler);

        self.subscribe(wrapped)
    }

    pub fn subscribe_mut<T>(&mut self, handler: T) -> usize
    where
        T: DynHandleMut + 'static,
    {
        let handler: Arc<Mutex<dyn DynHandleMut>> = Arc::new(Mutex::new(handler));
        let id = self.handler_count + 1;
        self.handlers.insert(id, HandlerType::SyncMut(handler));
        self.handler_count = id;

        id
    }

    /// Remove a handler from the publisher so that it stops receiving events
    pub fn unsubscribe(&mut self, id: usize) {
        self.handlers.remove_entry(&id);
    }
    /// Remove a mut handler from the publisher so that it stops receiving events
    pub fn unsubscribe_mut(&mut self, id: usize) {
        self.handlers.remove_entry(&id);
    }

    /// Publish an event to all subscribed handlers, utilizing as many threads as possible to run
    /// handlers in parallel
    pub fn publish<T>(
        &mut self,
        event: T,
    ) -> Result<(), Vec<Box<dyn std::any::Any + Send + 'static>>>
    where
        T: DynEvent,
    {
        let event: Arc<dyn DynEvent> = Arc::new(event);
        let max_threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let mut errors = Vec::new();

        thread::scope(|s| {
            let mut active_handles: Vec<
                thread::ScopedJoinHandle<Result<(), Box<dyn std::any::Any + Send + 'static>>>,
            > = Vec::new();

            for handler in self.handlers.values() {
                match handler {
                    HandlerType::Sync(dyn_handle) => {
                        // if we hit the max number of threads, join the oldest before spawning a new one
                        if active_handles.len() >= max_threads {
                            let handle = active_handles.remove(0);
                            if let Err(e) = handle.join().unwrap_or_else(Err) {
                                errors.push(e);
                            }
                        }

                        let handler_clone = Arc::clone(dyn_handle);
                        let cloned_event = event.clone();
                        active_handles.push(s.spawn(move || {
                            std::panic::catch_unwind(|| {
                                handler_clone.dyn_handle(cloned_event.as_ref())
                            })
                        }));
                    }
                    HandlerType::SyncMut(mutex) => {
                        // mutable handlers are called in series to prevent problems caused by simultaneous
                        // mutation of the same object
                        let handler_mut_clone = Arc::clone(mutex);
                        let cloned_event = event.clone();

                        let mut handler_guard =
                            handler_mut_clone.lock().expect("Handler mutex poisoned");
                        handler_guard.dyn_handle_mut(cloned_event.as_ref());
                    }
                }
                // if we hit the max number of threads, join the oldest before spawning a new one
                if active_handles.len() >= max_threads {
                    let handle = active_handles.remove(0);
                    if let Err(e) = handle.join().unwrap_or_else(Err) {
                        errors.push(e);
                    }
                }
            }

            for handle in active_handles {
                if let Err(e) = handle.join().unwrap_or_else(Err) {
                    errors.push(e);
                }
            }
        });

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Event;

    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct TestEvent;
    impl Event for TestEvent {}

    struct TestHandler {
        called: Arc<Mutex<bool>>,
    }
    impl DynHandle for TestHandler {
        fn dyn_handle(&self, _event: &dyn DynEvent) {
            let mut called = self.called.lock().unwrap();
            *called = true;
        }
    }

    struct TestHandlerMut {
        called: Arc<Mutex<bool>>,
    }
    impl DynHandleMut for TestHandlerMut {
        fn dyn_handle_mut(&mut self, _event: &dyn DynEvent) {
            let mut called = self.called.lock().unwrap();
            *called = true;
        }
    }

    struct PanicHandler;
    impl DynHandle for PanicHandler {
        fn dyn_handle(&self, _event: &dyn DynEvent) {
            panic!("handler panic");
        }
    }

    #[test]
    fn test_subscribe_with_and_publish() {
        let mut publisher = Publisher::default();
        let called = Arc::new(Mutex::new(false));
        let handler = TestHandler {
            called: called.clone(),
        };
        publisher.subscribe(handler);
        let _ = publisher.publish(TestEvent);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_subscribe_and_publish() {
        let mut publisher = Publisher::default();
        let called = Arc::new(Mutex::new(false));
        let handler = TestHandler {
            called: called.clone(),
        };
        publisher.subscribe(handler);
        let _ = publisher.publish(TestEvent);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_unsubscribe() {
        let mut publisher = Publisher::default();
        let called = Arc::new(Mutex::new(false));
        let handler = TestHandler {
            called: called.clone(),
        };
        let id = publisher.subscribe(handler);
        publisher.unsubscribe(id);
        let _ = publisher.publish(TestEvent);
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn test_publish_error() {
        let mut publisher = Publisher::default();
        publisher.subscribe(PanicHandler);
        let result = publisher.publish(TestEvent);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }

    #[test]
    fn test_subscribe_mut_and_publish() {
        let mut publisher = Publisher::default();
        let called = Arc::new(Mutex::new(false));
        let handler = TestHandlerMut {
            called: called.clone(),
        };
        publisher.subscribe_mut(handler);
        let _ = publisher.publish(TestEvent);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_unsubsribe_mut() {
        let mut publisher = Publisher::default();
        let called = Arc::new(Mutex::new(false));
        let handler = TestHandlerMut {
            called: called.clone(),
        };
        let id = publisher.subscribe_mut(handler);
        publisher.unsubscribe_mut(id);
        let _ = publisher.publish(TestEvent);
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn test_publish_to_both_handler_types() {
        let mut publisher = Publisher::default();
        let called = Arc::new(Mutex::new(false));
        let called_mut = Arc::new(Mutex::new(false));
        let handler = TestHandler {
            called: called.clone(),
        };
        let handler_mut = TestHandlerMut {
            called: called_mut.clone(),
        };
        publisher.subscribe(handler);
        publisher.subscribe_mut(handler_mut);
        let _ = publisher.publish(TestEvent);
        assert!(*called.lock().unwrap());
        assert!(*called_mut.lock().unwrap());
    }
}
