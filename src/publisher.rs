use std::{collections::HashMap, sync::Arc, thread};

use crate::{DynEvent, DynHandle};

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
    // we use Arc so that a reference to the handler can be passed to other threads for
    // execution
    handlers: HashMap<usize, Arc<dyn DynHandle>>,
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

    /// Remove a handler from the publisher so that it stops receiving events
    pub fn unsubscribe(&mut self, id: usize) {
        self.handlers.remove_entry(&id);
    }

    /// Publish an event to all subscribed handlers, utilizing as many threads as possible to run
    /// handlers in parallel
    pub fn publish<T>(&self, event: T) -> Result<(), Vec<Box<dyn std::any::Any + Send + 'static>>>
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

                handles.push(s.spawn(move || handler.dyn_handle(cloned_event.as_ref())));

                if handles.len() == max_threads {
                    for handle in handles.drain(..) {
                        if let Err(e) = handle.join() {
                            errors.push(e)
                        }
                    }
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
