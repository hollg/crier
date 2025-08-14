use std::any;

/// An object that a Publisher can send to its subscribers
pub trait Event: Send + Sync + Clone + 'static {}

/// Dynamically typed event. Used internally to alow Publishers to support Handlers and Events of
/// multiple different types.
pub trait DynEvent: Send + Sync + 'static {
    fn get_data(&self) -> &dyn any::Any;
}

// Allow handlers to identify the concrete type of any Event object.
// This is what enables the Publisher to take events of any type. It ensures that anything
// implementing Event is safe to pass to any DynHandler, which can decide whether or not to handle
// the event based on its concrete type.
impl<T: Event> DynEvent for T {
    fn get_data(&self) -> &dyn any::Any {
        self
    }
}
