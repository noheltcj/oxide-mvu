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
//! enum Event { AccumulateClicked }
//!
//! #[derive(Clone)]
//! struct Model { count: i32 }
//!
//! struct Props { count: i32, on_accumulate_click: Box<dyn Fn()> }
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
//!             on_accumulate_click: Box::new(move || emitter.emit(Event::AccumulateClicked))
//!         }
//!     }
//! }
//!
//! struct MyRenderer;
//! impl Renderer<Props> for MyRenderer {
//!     fn render(&mut self, _props: Props) {}
//! }
//!
//! let runtime = MvuRuntime::new(
//!     Model { count: 0 },
//!     Box::new(MyLogic),
//!     Box::new(MyRenderer)
//! );
//! runtime.run();
//! ```

#[cfg(feature = "no_std")]
extern crate alloc;

// Module declarations
mod logic;
mod renderer;
mod effect;
mod emitter;
mod runtime;

// Public re-exports
pub use logic::MvuLogic;
pub use renderer::Renderer;
pub use effect::Effect;
pub use emitter::Emitter;
pub use runtime::MvuRuntime;

// Test utilities (only available with 'testing' feature or during tests)
#[cfg(any(test, feature = "testing"))]
pub use renderer::TestRenderer;
#[cfg(any(test, feature = "testing"))]
pub use runtime::{TestMvuRuntime, TestMvuDriver};
