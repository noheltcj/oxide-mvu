//! Event emitter for embedding callbacks in Props.

#[cfg(feature = "no_std")]
use alloc::boxed::Box;

use portable_atomic_util::Arc;
use spin::Mutex;

/// Event emitter that can be embedded in Props.
///
/// Clone this handle to create callbacks in your Props that can trigger
/// events when invoked (e.g., by user interaction).
///
/// `Emitter` uses interior mutability via `Arc<Mutex<...>>`, making it
/// cheap to clone (just an atomic reference count increment) and thread-safe.
///
/// # Example
///
/// ```rust
/// use oxide_mvu::{Emitter, MvuLogic, Effect};
///
/// enum Event { Click }
///
/// #[derive(Clone)]
/// struct Model { clicks: u32 }
///
/// struct Props {
///     clicks: u32,
///     on_click: Box<dyn Fn()>,
/// }
///
/// struct MyApp;
///
/// impl MvuLogic<Event, Model, Props> for MyApp {
///     fn init(&self, model: Model) -> (Model, Effect<Event>) {
///         (model, Effect::none())
///     }
///
///     fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>) {
///         match event {
///             Event::Click => {
///                 let new_model = Model {
///                     clicks: model.clicks + 1,
///                     ..model.clone()
///                 };
///                 (new_model, Effect::none())
///             }
///         }
///     }
///
///     fn view(&self, model: &Model, emitter: &Emitter<Event>) -> Props {
///         let emitter = emitter.clone();
///         Props {
///             clicks: model.clicks,
///             on_click: Box::new(move || {
///                 emitter.emit(Event::Click);
///             }),
///         }
///     }
/// }
/// ```
#[allow(clippy::type_complexity)]
pub struct Emitter<Event>(pub(crate) Arc<Mutex<Box<dyn FnMut(Event) + Send + 'static>>>);

impl<Event> Clone for Emitter<Event> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Event> Emitter<Event> {
    /// Create a new emitter from a callback function.
    ///
    /// The callback will be invoked when [`emit`](Self::emit) is used.
    pub fn new<F: FnMut(Event) + Send + 'static>(f: F) -> Self {
        Self(Arc::new(Mutex::new(Box::new(f))))
    }

    /// Emit an event.
    ///
    /// This queues the event for processing by the runtime. Multiple threads
    /// can safely call this method concurrently.
    pub fn emit(&self, event: Event) {
        let mut f = self.0.lock();
        f(event);
    }
}
