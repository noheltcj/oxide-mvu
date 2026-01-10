//! Declarative effect system for describing deferred event processing.

#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use core::future::Future;
use core::pin::Pin;

use crate::Emitter;

/// Declarative description of events to be processed.
///
/// Effects allow you to describe asynchronous or deferred work that will
/// produce events. They are returned from [`MvuLogic::init`](crate::MvuLogic::init)
/// and [`MvuLogic::update`](crate::MvuLogic::update) with the new model state.
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
pub struct Effect<Event>(Box<dyn FnOnceBox<Event> + Send>);

impl<Event: 'static> Effect<Event> {
    /// Execute the effect, consuming it and returning a future.
    ///
    /// The returned future will be spawned on your async runtime using the provided spawner.
    pub fn execute(self, emitter: &Emitter<Event>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.0.call_box(emitter)
    }

    /// Create an empty effect.
    ///
    /// This is private - use [`Effect::none()`] instead.
    fn new() -> Self {
        fn empty_fn<Event>(_: &Emitter<Event>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
            Box::pin(async {})
        }
        Self(Box::new(empty_fn))
    }

    /// Create an effect that just emits a single event.
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
        Event: Send + 'static,
    {
        Self(Box::new(move |emitter: &Emitter<Event>| {
            let emitter = emitter.clone();
            Box::pin(async move { emitter.emit(event) }) as Pin<Box<dyn Future<Output = ()> + Send>>
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
            let emitter = emitter.clone();
            Box::pin(async move {
                for effect in effects {
                    effect.execute(&emitter).await;
                }
            }) as Pin<Box<dyn Future<Output = ()> + Send>>
        }))
    }

    /// Create an effect from an async function using a runtime-agnostic spawner.
    ///
    /// This allows you to use async/await syntax with any async runtime (tokio,
    /// async-std, smol, etc.) by providing a spawner function that knows how to
    /// execute futures on your chosen runtime.
    ///
    /// The async function receives a cloned `Emitter` that can be used to emit
    /// events when the async work completes.
    ///
    /// # Arguments
    ///
    /// * `spawner` - A function that spawns the future on your async runtime
    /// * `f` - An async function that receives an Emitter and returns a Future
    ///
    /// # Example with tokio
    ///
    /// ```rust,no_run
    /// use oxide_mvu::Effect;
    /// use std::time::Duration;
    ///
    /// #[derive(Clone)]
    /// enum Event {
    ///     FetchData,
    ///     DataLoaded(String),
    ///     DataFailed(String),
    /// }
    ///
    /// async fn fetch_from_api() -> Result<String, String> {
    ///     // Await some async operation...
    ///     Ok("data from API".to_string())
    /// }
    ///
    /// let effect = Effect::from_async(
    ///     |emitter| async move {
    ///         match fetch_from_api().await {
    ///             Ok(data) => emitter.emit(Event::DataLoaded(data)),
    ///             Err(err) => emitter.emit(Event::DataFailed(err)),
    ///         }
    ///     }
    /// );
    /// ```
    ///
    /// # Example with async-std
    ///
    /// ```rust,no_run
    /// use oxide_mvu::Effect;
    ///
    /// #[derive(Clone)]
    /// enum Event { TimerAlert }
    ///
    /// let await_timer_effect = Effect::from_async(
    ///     |emitter| async move {
    ///         // Await timer
    ///         emitter.emit(Event::TimerAlert);
    ///     }
    /// );
    /// ```
    pub fn from_async<F, Fut>(f: F) -> Self
    where
        F: FnOnce(Emitter<Event>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self(Box::new(move |emitter: &Emitter<Event>| {
            let future = f(emitter.clone());
            Box::pin(future) as Pin<Box<dyn Future<Output = ()> + Send>>
        }))
    }
}

trait FnOnceBox<Event> {
    fn call_box(
        self: Box<Self>,
        emitter: &Emitter<Event>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl<F, Event> FnOnceBox<Event> for F
where
    F: for<'a> FnOnce(&'a Emitter<Event>) -> Pin<Box<dyn Future<Output = ()> + Send>>,
{
    fn call_box(
        self: Box<Self>,
        emitter: &Emitter<Event>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        (*self)(emitter)
    }
}
