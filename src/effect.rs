//! Declarative effect system for describing deferred event processing.

#[cfg(not(feature = "no_std"))]
use std::future::Future;
#[cfg(feature = "no_std")]
use core::future::Future;
#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

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
    ///     tokio::time::sleep(Duration::from_secs(1)).await;
    ///     Ok("data from API".to_string())
    /// }
    ///
    /// let effect = Effect::from_async_fn(
    ///     |fut| { tokio::spawn(fut); },
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
    /// enum Event { TimerExpired }
    ///
    /// let effect = Effect::from_async(
    ///     |fut| { async_std::task::spawn(fut); },
    ///     |emitter| async move {
    ///         async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    ///         emitter.emit(Event::TimerExpired);
    ///     }
    /// );
    /// ```
    pub fn from_async<F, Fut, S>(spawner: S, f: F) -> Self
    where
        F: Fn(Emitter<Event>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self(Box::new(move |emitter: &Emitter<Event>| {
            let future = f(emitter.clone());
            // TODO: The spawner absolutely shouldn't be used here.
            spawner(future);
        }))
    }
}
