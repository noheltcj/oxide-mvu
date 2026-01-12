//! Event emitter for embedding callbacks in Props.

use thingbuf::mpsc::Sender;

use crate::Event as EventTrait;

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
/// #[derive(Clone)]
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
///                 emitter.try_emit(Event::Click);
///             }),
///         }
///     }
/// }
/// ```
// Note: We wrap events in Option internally to satisfy thingbuf's Default requirement
// for its recycling mechanism. This allows us to avoid requiring Default on user events.
pub struct Emitter<Event: EventTrait>(pub(crate) Sender<Option<Event>>);

impl<Event: EventTrait> Clone for Emitter<Event> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Event: EventTrait> Emitter<Event> {
    /// Create a new emitter from a channel sender.
    pub(crate) fn new(sender: Sender<Option<Event>>) -> Self {
        Self(sender)
    }

    /// Emit an event without blocking.
    ///
    /// This attempts to queue the event for processing by the runtime. If the
    /// event queue is full, the event will be dropped and `false` is returned.
    ///
    /// Multiple threads can safely call this method concurrently via the lock-free channel.
    pub fn try_emit(&self, event: Event) -> bool {
        self.0.try_send(Some(event)).is_ok()
    }

    /// Emit an event, waiting until space is available.
    ///
    /// This queues the event for processing by the runtime. If the queue is full,
    /// this method will await until space becomes available.
    ///
    /// Multiple threads can safely call this method concurrently via the lock-free channel.
    pub async fn emit(&self, event: Event) {
        self.0.send(Some(event)).await.ok();
    }
}
