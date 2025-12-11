//! Declarative effect system for describing deferred event processing.

#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use crate::Emitter;

/// Declarative description of events to be processed.
///
/// Effects allow you to describe asynchronous or deferred work that will
/// produce events. They are returned from [`MvuLogic::init`](crate::MvuLogic::init)
/// and [`MvuLogic::update`](crate::MvuLogic::update) alongside the new model state.
///
/// # Example
///
/// ```rust
/// use oxide_mvu::Effect;
///
/// #[derive(Clone)]
/// enum Event {
///     LoadData,
///     DataLoaded(String),
/// }
///
/// // Trigger a follow-up event
/// let effect = Effect::just(Event::LoadData);
///
/// // Combine multiple effects
/// let effect = Effect::batch(vec![
///     Effect::just(Event::LoadData),
///     Effect::just(Event::DataLoaded("cached".to_string())),
/// ]);
///
/// // No side effects
/// let effect: Effect<Event> = Effect::none();
/// ```
#[allow(clippy::type_complexity)]
pub struct Effect<Event>(Box<dyn Fn(&Emitter<Event>) + Send + 'static>);

impl<Event: 'static> Effect<Event> {
    /// Create an empty effect.
    ///
    /// This is private - use [`Effect::none()`] instead.
    fn new() -> Self {
        Self(Box::new(|_| {}))
    }

    pub fn execute(&self, emitter: &Emitter<Event>) {
        (self.0)(emitter);
    }

    /// Create an effect just a single event.
    ///
    /// Useful for triggering immediate follow-up events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxide_mvu::Effect;
    ///
    /// #[derive(Clone)]
    /// enum Event { Refresh }
    ///
    /// let effect = Effect::just(Event::Refresh);
    /// ```
    pub fn just(event: Event) -> Self
    where
        Event: Clone + Send + 'static,
    {
        Self(Box::new(move |emitter: &Emitter<Event>| {
            emitter.emit(event.clone());
        }))
    }

    /// Create an empty effect.
    ///
    /// Prefer this when semantically indicating "no side effects".
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxide_mvu::Effect;
    ///
    /// #[derive(Clone)]
    /// enum Event { Increment }
    ///
    /// let effect: Effect<Event> = Effect::none();
    /// ```
    pub fn none() -> Self {
        Self::new()
    }

    /// Combine multiple effects into a single effect.
    ///
    /// All events from all effects will be queued for processing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxide_mvu::Effect;
    ///
    /// #[derive(Clone)]
    /// enum Event { A, B, C }
    ///
    /// let combined = Effect::batch(vec![
    ///     Effect::just(Event::A),
    ///     Effect::just(Event::B),
    ///     Effect::just(Event::C),
    /// ]);
    /// ```
    pub fn batch(effects: Vec<Effect<Event>>) -> Self {
        Self(Box::new(move |emitter: &Emitter<Event>| {
            for effect in &effects {
                effect.execute(emitter);
            }
        }))
    }
}
