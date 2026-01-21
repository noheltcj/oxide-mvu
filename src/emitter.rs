//! Event emitter for embedding callbacks in Props.

use async_channel::Sender;

use crate::Event as EventTrait;

/// Event emitter that can be embedded in Props.
///
/// Clone this handle to create callbacks in your Props that can trigger
/// events when invoked (e.g., by user interaction).
///
/// `Emitter` wraps a lock-free MPMC channel sender, making it cheap to clone
/// and thread-safe without any locking overhead.
///
/// # Performance Characteristics
///
/// Cloning an `Emitter` is a cheap operation (equivalent to cloning an `Arc`).
/// It increments a reference count using an atomic operation rather than
/// allocating new memory. This makes it safe to clone frequently without
/// significant performance cost.
///
/// However, atomic operations do have some overhead (memory barriers, cache
/// coherency), so the following patterns are recommended for optimal performance:
///
/// ## Recommended Pattern: Clone Once per View
///
/// When creating multiple callbacks in your `view()` function, clone the emitter
/// once and reuse it:
///
/// ```rust
/// # use oxide_mvu::{Emitter, MvuLogic, Effect};
/// # #[derive(Clone)] enum Event { Click, Hover, Focus }
/// # #[derive(Clone)] struct Model;
/// # struct Props {
/// #     on_click: Box<dyn Fn()>,
/// #     on_hover: Box<dyn Fn()>,
/// #     on_focus: Box<dyn Fn()>,
/// # }
/// # struct MyApp;
/// # impl MvuLogic<Event, Model, Props> for MyApp {
/// #     fn init(&self, model: Model) -> (Model, Effect<Event>) { (model, Effect::none()) }
/// #     fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>) { (model.clone(), Effect::none()) }
/// fn view(&self, model: &Model, emitter: &Emitter<Event>) -> Props {
///     // GOOD: Clone once, reuse for all callbacks
///     let emitter = emitter.clone();
///     Props {
///         on_click: Box::new({
///             let e = emitter.clone();
///             move || { e.try_emit(Event::Click); }
///         }),
///         on_hover: Box::new({
///             let e = emitter.clone();
///             move || { e.try_emit(Event::Hover); }
///         }),
///         on_focus: Box::new({
///             let e = emitter.clone();
///             move || { e.try_emit(Event::Focus); }
///         }),
///     }
/// }
/// # }
/// ```
///
/// This pattern minimizes atomic operations while maintaining clean code structure.
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
pub struct Emitter<Event: EventTrait>(pub(crate) Sender<Event>);

impl<Event: EventTrait> Clone for Emitter<Event> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Event: EventTrait> Emitter<Event> {
    /// Create a new emitter from a channel sender.
    pub(crate) fn new(sender: Sender<Event>) -> Self {
        Self(sender)
    }

    /// Emit an event without blocking.
    ///
    /// This attempts to queue the event for processing by the runtime. If the
    /// event queue is full, the event will be dropped and `false` is returned.
    ///
    /// Multiple threads can safely call this method concurrently via the lock-free channel.
    pub fn try_emit(&self, event: Event) -> bool {
        self.0.try_send(event).is_ok()
    }

    /// Emit an event, waiting until space is available.
    ///
    /// This queues the event for processing by the runtime. If the queue is full,
    /// this method will await until space becomes available.
    ///
    /// Multiple threads can safely call this method concurrently via the lock-free channel.
    pub async fn emit(&self, event: Event) {
        self.0.send(event).await.ok();
    }
}
