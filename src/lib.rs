#![cfg_attr(feature = "no_std", no_std)]

//! A lightweight Model-View-Update (MVU) runtime for Rust with `no_std` support.
//!
//! Implements the MVU pattern for building predictable, testable applications with
//! unidirectional data flow and controlled side effects.
//!
//! ## Example
//!
//! ```rust
//! use oxide_mvu::{Emitter, Effect, MvuLogic, MvuRuntime, Renderer};
//!
//! #[derive(Clone)]
//! enum Event {
//!     AccumulateClicked,
//! }
//!
//! #[derive(Clone)]
//! struct Model {
//!     count: i32,
//! }
//!
//! struct Props {
//!     count: i32,
//!     on_accumulate_click: Box<dyn Fn()>,
//! }
//!
//! struct MyLogic;
//!
//! impl MvuLogic<Event, Model, Props> for MyLogic {
//!     fn init(&self, model: Model) -> (Model, Effect<Event>) {
//!         (model, Effect::none())
//!     }
//!
//!     fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>) {
//!         match event {
//!             Event::AccumulateClicked => {
//!                 let new_model = Model {
//!                     count: model.count + 1,
//!                     ..model.clone()
//!                 };
//!                 (new_model, Effect::none())
//!             }
//!         }
//!     }
//!
//!     fn view(&self, model: &Model, emitter: &Emitter<Event>) -> Props {
//!         let emitter = emitter.clone();
//!         Props {
//!             count: model.count,
//!             on_accumulate_click: Box::new(move || {
//!                 emitter.try_emit(Event::AccumulateClicked);
//!             }),
//!         }
//!     }
//! }
//!
//! struct MyRenderer;
//!
//! impl Renderer<Props> for MyRenderer {
//!     fn render(&mut self, _props: Props) {}
//! }
//!
//! async fn main_async() {
//!     // Create a spawner for your async runtime.
//!     // This is how `Effect`s are executed.
//!     let spawner = |fut| {
//!         // Spawn the future on your chosen runtime.
//!         // Examples:
//!         // tokio::spawn(fut);
//!         // async_std::task::spawn(fut);
//!         let _ = fut;
//!     };
//!
//!     let runtime = MvuRuntime::builder(
//!         Model { count: 0 },
//!         MyLogic,
//!         MyRenderer,
//!         spawner,
//!     ).build();
//!
//!     // `run()` returns a Future representing the event loop.
//!     // It must be awaited inside an async context.
//!     runtime.run().await;
//! }
//! ```
//!
//! In a real application, `main_async` would be executed by your async runtime
//! (e.g. via `#[tokio::main]`, `async_std::main`, or an embedded executor).

#[cfg(feature = "no_std")]
extern crate alloc;

/// Trait alias for event type constraints.
///
/// All events must implement these bounds to work with the MVU runtime.
pub trait Event: Send + Sync + Clone + 'static {}

/// Blanket implementation for any type that satisfies the bounds.
impl<T> Event for T where T: Send + Sync + Clone + 'static {}

// Module declarations
mod effect;
mod emitter;
mod logic;
mod renderer;
mod runtime;

// Public re-exports
pub use effect::Effect;
pub use emitter::Emitter;
pub use logic::MvuLogic;
pub use renderer::Renderer;
pub use runtime::{MvuRuntime, MvuRuntimeBuilder, Spawner, DEFAULT_EVENT_CAPACITY};

// Test utilities (only available with 'testing' feature or during tests)
#[cfg(any(test, feature = "testing"))]
pub use renderer::TestRenderer;
#[cfg(any(test, feature = "testing"))]
pub use runtime::{create_test_spawner, TestMvuDriver, TestMvuRuntime, TestMvuRuntimeBuilder};
