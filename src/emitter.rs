//! Event emitter for embedding callbacks in Props.

use crossbeam_channel::Sender;

/// Event emitter that can be embedded in Props.
///
/// Clone this handle to create callbacks in your Props that can trigger
/// events when invoked (e.g., by user interaction).
///
/// `Emitter` wraps a lock-free channel sender, making it cheap to clone
/// and thread-safe without any locking overhead.
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
pub struct Emitter<Event: Send>(pub(crate) Sender<Event>);

impl<Event: Send> Clone for Emitter<Event> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Event: Send> Emitter<Event> {
    /// Create a new emitter from a channel sender.
    pub(crate) fn new(sender: Sender<Event>) -> Self {
        Self(sender)
    }

    /// Emit an event.
    ///
    /// This queues the event for processing by the runtime. Multiple threads
    /// can safely call this method concurrently via the lock-free channel.
    pub fn emit(&self, event: Event) {
        self.0.send(event).ok();
    }
}
