#![cfg_attr(feature = "no_std", no_std)]

//! A lightweight Model-View-Update (MVU) runtime for Rust with `no_std` support.
//!
//! Implements the MVU pattern for building predictable, testable applications with
//! unidirectional data flow and controlled side effects.
//!
//! # Overview
//!
//! The Model-View-Update (MVU) pattern, also known as the Elm Architecture, structures
//! applications as a pure functional loop with three main components:
//!
//! - **Model**: Immutable state representing your entire application
//! - **Update**: Pure function that transforms `(Event, Model) → (Model, Effect)`
//! - **View**: Pure function that derives renderable Props from the Model
//!
//! This architecture makes state transitions predictable, debuggable, and testable by
//! eliminating implicit state mutation and enforcing unidirectional data flow.
//!
//! # Core Concepts
//!
//! ## Model
//!
//! The Model is an immutable data structure representing your application's complete state.
//! All behavior is a pure function of this state - nothing happens outside of it.
//!
//! ```rust
//! #[derive(Clone)]
//! struct Model {
//!     counter: i32,
//!     user_name: String,
//!     is_loading: bool,
//! }
//! ```
//!
//! ## Event
//!
//! Events are discrete, immutable messages that trigger state transitions. They represent inputs
//! from users, systems, or asynchronous effects. Events must implement the `Clone` trait.
//!
//! ```rust
//! #[derive(Clone)]
//! enum Event {
//!     UserClickedButton,
//!     DataLoaded(String),
//!     TimerTicked,
//! }
//! ```
//!
//! ## Effect
//!
//! Effects declaratively describe side-effecting work (async I/O, timers, etc.) that produces
//! events. They maintain purity in your update logic while enabling real-world interactions.
//!
//! Depending on your spawner implementation, effects may be processed in parallel.
//!
//! ```rust
//! # use oxide_mvu::Effect;
//! # #[derive(Clone)] enum Event { DataLoaded(String) }
//! // Describe an async operation that will emit an event
//! let effect = Effect::from_async(async move |emitter| {
//!     let data = fetch_data().await;
//!     emitter.emit(Event::DataLoaded(data)).await;
//! });
//! # async fn fetch_data() -> String { String::new() }
//! ```
//!
//! ## Props
//!
//! Props are a pure, derived projection of the Model optimized for rendering or external
//! presentation. They describe WHAT to render without prescribing HOW. Props commonly
//! contain data and callbacks created via [`Emitter`].
//!
//! Props are not limited to UI - they can represent any external projection (API responses,
//! hardware states, serialized output).
//!
//! # Architecture
//!
//! The MVU runtime orchestrates a unidirectional event loop:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │                                                      │
//! │  User Input / External Signal   ◀──────────┐         │
//! │ (ie. Props callback or effect)             │         │
//! │              │                             │         │
//! │              ▼                             │         │
//! │         ┌────────┐                         │         │
//! │         │ Event  │                         │         │
//! │         └────┬───┘                         │         │
//! │              │  // sequenced by            │         │
//! │              ▼  // the runtime             │         │
//! │      ┌───────────────┐                     │         │
//! │      │ update(event, │             ┌───────┴─────┐   │
//! │      │     model)    │             │ emit(event) │   │
//! │      └───────┬───────┘             └─────────────┘   │
//! │              │                             ▲         │
//! │              ▼                             │         │
//! │    ┌─────────────────────┐                 │         │
//! │    │ (Model, Effect)     │                 │         │
//! │    └──────┬──────┬───────┘                 │         │
//! │           │      │                         │         │
//! │           │      └──► Effect task spawn ───┘         │
//! │           │                                ▲         │
//! │           ▼                                │         │
//! │      ┌─────────┐                           │         │
//! │      │  Model  │                           │         │
//! │      └────┬────┘                           │         │
//! │           │                                │         │
//! │           ▼                                │         │
//! │    ┌──────────────┐                        │         │
//! │    │ view(model,  │               ┌────────┴───────┐ │
//! │    │   emitter)   │               │ props callback │ │
//! │    └──────┬───────┘               │ uses emitter   │ │
//! │           │                       └────────────────┘ │
//! │           ▼                                ▲         │
//! │       ┌───────┐                            │         │
//! │       │ Props │  // emitter captured       │         │
//! │       └───┬───┘  // in Props callbacks     │         │
//! │           │                                │         │
//! │           ▼                                │         │
//! │   ┌───────────────┐                  ┌────────────┐  │
//! │   │ render(props) │  ─────────────►  │ user input │  │
//! │   └───────────────┘                  └────────────┘  │
//! │                                                      │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! # When to Use
//!
//! **MVU is ideal for:**
//! - Applications requiring predictable, debuggable state management
//! - Event-driven systems (GUIs, embedded controllers, game loops)
//! - Applications where state can be serialized/replayed (time-travel debugging)
//! - Teams prioritizing testability and clear separation of concerns
//! - `no_std` environments that can afford minimal heap allocations and meet prior criteria
//!
//! **Consider alternatives if:**
//! - You need direct object mutation for performance-critical inner loops
//! - Your application is primarily synchronous with minimal state
//!
//! # Platform Support
//!
//! ## Standard Environments
//!
//! By default, `oxide-mvu` works with the standard library.
//!
//! ```toml
//! [dependencies]
//! oxide-mvu = "0.4.0"
//! ```
//!
//! ## `no_std` Environments
//!
//! For embedded systems or environments without the standard library, enable the
//! `no_std` feature. This requires an allocator (`alloc` crate) but replaces all
//! standard library dependencies with `no_std` crates:
//!
//! ```toml
//! [dependencies]
//! oxide-mvu = { version = "0.4.0", features = ["no_std"] }
//! ```
//!
//! The runtime uses lock-free concurrency and bounded channels to minimize the cost of event
//! synchronization. This makes the framework concurrency-model agnostic. Effects may execute in
//! parallel or concurrently on the same thread as the runtime depending on hardware availability
//! and your spawner implementation.
//!
//! # Quick Start
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
//!
//! # Advanced Topics
//!
//! ## Effect Composition
//!
//! Multiple effects can be combined using [`Effect::batch`]:
//!
//! ```rust
//! # use oxide_mvu::Effect;
//! # #[derive(Clone)] enum Event { A, B, C }
//! let effect = Effect::batch(vec![
//!     Effect::just(Event::A),
//!     Effect::just(Event::B),
//!     Effect::just(Event::C),
//! ]);
//! ```
//!
//! ## Event Buffer Capacity
//!
//! The runtime uses a bounded channel to queue events. The default capacity
//! ([`DEFAULT_EVENT_CAPACITY`] = 32) is sized for embedded systems with limited heap.
//! Customize this via the builder:
//!
//! ```rust,no_run
//! # use oxide_mvu::MvuRuntime;
//! # struct Model; struct Logic; struct MyRenderer;
//! # let spawner = |_| {};
//! // Memory-constrained embedded systems
//! let runtime = MvuRuntime::builder(Model, Logic, MyRenderer, spawner)
//!     .capacity(8)
//!     .build();
//!
//! // High-throughput applications with event bursts
//! let runtime = MvuRuntime::builder(Model, Logic, MyRenderer, spawner)
//!     .capacity(1024)
//!     .build();
//! ```
//!
//! When the buffer is full:
//! - [`Emitter::try_emit`] returns `false` and drops the event
//! - [`Emitter::emit`] awaits until space is available (backpressure)
//!
//! ## Testing
//!
//! Enable the `testing` feature to access test utilities:
//!
//! ```toml
//! [dev-dependencies]
//! oxide-mvu = { version = "0.4.0", features = ["testing"] }
//! ```
//!
//! Test your MVU logic deterministically:
//!
//! ```rust
//! # #[cfg(feature = "testing")]
//! # {
//! use oxide_mvu::{Effect, MvuLogic, TestMvuRuntime};
//! # #[derive(Clone)] enum Event { Increment }
//! # #[derive(Clone)] struct Model { count: i32 }
//! # struct Props;
//! # struct Logic;
//! # impl MvuLogic<Event, Model, Props> for Logic {
//! #     fn init(&self, m: Model) -> (Model, Effect<Event>) { (m, Effect::none()) }
//! #     fn update(&self, _: Event, m: &Model) -> (Model, Effect<Event>) {
//! #         (Model { count: m.count + 1 }, Effect::none())
//! #     }
//! #     fn view(&self, _: &Model, _: &oxide_mvu::Emitter<Event>) -> Props { Props }
//! # }
//!
//! # async fn test() {
//! let mut driver = TestMvuRuntime::builder(Model { count: 0 }, Logic).build();
//!
//! // Process an event and get the new model
//! let model = driver.process(Event::Increment).await;
//! assert_eq!(model.count, 1);
//! # }
//! # }
//! ```
//!
//! See `TestMvuRuntime` (available with the `testing` feature) for comprehensive testing utilities.
//!
//! ## Async Runtime Integration
//!
//! The [`Spawner`] trait abstracts over different async runtimes. Common patterns:
//!
//! ```rust,no_run
//! # use oxide_mvu::MvuRuntime;
//! # struct Model; struct Logic; struct MyRenderer;
//! // tokio
//! let spawner = |fut| { tokio::spawn(fut); };
//!
//! // async-std
//! let spawner = |fut| { async_std::task::spawn(fut); };
//!
//! // smol
//! let spawner = |fut| { smol::spawn(fut).detach(); };
//! # let runtime = MvuRuntime::builder(Model, Logic, MyRenderer, spawner).build();
//! ```
//!
//! # See Also
//!
//! - [`MvuLogic`] - The core trait defining application behavior
//! - [`Effect`] - Declarative side effect system
//! - [`Emitter`] - Event dispatch from Props callbacks
//! - [`Renderer`] - Integration point for rendering systems
//! - [`MvuRuntime`] - The runtime orchestrating the event loop

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
