use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use crate::{handler::DynHandleMut, DynEvent, DynHandle};

/// Publishes all Events to all subscribed Handlers that accept Events of that type
/// # Examples
/// ```
/// use gawk::{Event, Handler, Publisher};
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
    handler_mut_count: usize,
    // we use Arc so that a reference to the handler can be passed to other threads for
    // execution
    handlers: HashMap<usize, Arc<dyn DynHandle>>,
    handlers_mut: HashMap<usize, Arc<Mutex<dyn DynHandleMut>>>,
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
        self.handlers.insert(id, handler);
        self.handler_count = id;

        id
    }

    pub fn subscribe_mut<T>(&mut self, handler: T) -> usize
    where
        T: DynHandleMut + 'static,
    {
        let handler: Arc<Mutex<dyn DynHandleMut>> = Arc::new(Mutex::new(handler));
        let id = self.handler_mut_count + 1;
        self.handlers_mut.insert(id, handler);
        self.handler_mut_count = id;

        id
    }

    /// Remove a handler from the publisher so that it stops receiving events
    pub fn unsubscribe(&mut self, id: usize) {
        self.handlers.remove_entry(&id);
    }
    /// Remove a mut handler from the publisher so that it stops receiving events
    pub fn unsubsribe_mut(&mut self, id: usize) {
        self.handlers_mut.remove_entry(&id);
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
            let mut handles = Vec::new();
            for handler in self.handlers.values() {
                let handler = Arc::clone(handler);
                let cloned_event = event.clone();

                handles.push(s.spawn(move || {
                    std::panic::catch_unwind(|| handler.dyn_handle(cloned_event.as_ref()))
                }));

                if handles.len() == max_threads {
                    for handle in handles.drain(..) {
                        if let Err(e) = handle.join().unwrap_or_else(Err) {
                            errors.push(e);
                        }
                    }
                }
            }

            for handle in handles {
                if let Err(e) = handle.join().unwrap_or_else(Err) {
                    errors.push(e);
                }
            }
        });

        for handler_mut in self.handlers_mut.values_mut() {
            let handler_mut = Arc::clone(handler_mut);
            let cloned_event = event.clone();

            let mut handler_guard = handler_mut.lock().expect("Handler mutex poisoned");
            handler_guard.dyn_handle_mut(cloned_event.as_ref());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
        publisher.unsubsribe_mut(id);
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
}
