//! Application logic trait defining the MVU contract.

use crate::{Effect, Emitter};

/// Application logic trait defining the MVU contract.
///
/// Implementations must provide three pure functions:
/// - [`init`](Self::init): Initialize the model and produce initial effects
/// - [`update`](Self::update): Transform (Event, Model) → (Model, Effect)
/// - [`view`](Self::view): Derive Props from Model with event emitter capability
///
/// See the [crate-level documentation](crate) for a complete example.
pub trait MvuLogic<Event: Send, Model, Props> {
    /// Initialize the runtime from an initial model with effects and state changes as needed.
    ///
    /// This is called once when the runtime starts. Use it to set up initial
    /// state and trigger any bootstrap events.
    ///
    /// # Arguments
    ///
    /// * `model` - The initial model state
    ///
    /// # Returns
    ///
    /// A tuple of `(Model, Effect<Event>)` containing the initialized model
    /// and any effects to process during startup.
    fn init(&self, model: Model) -> (Model, Effect<Event>);

    /// Reduce an event to an updated model and side effects.
    ///
    /// This function takes an event and the current model, returning
    /// the new model and any effects to process. All state changes must
    /// happen through this function.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to process
    /// * `model` - The current model state
    ///
    /// # Returns
    ///
    /// A tuple of `(Model, Effect<Event>)` containing the updated model
    /// and any effects to process.
    fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>);

    /// Reduce to Props from the current model.
    ///
    /// This function creates a renderable representation (Props) from
    /// the model. The provided [`Emitter`] allows Props to contain callbacks
    /// that can trigger new events.
    ///
    /// # Arguments
    ///
    /// * `model` - The current model state
    /// * `emitter` - Event emitter for creating callbacks
    ///
    /// # Returns
    ///
    /// Props derived from the model, ready for rendering via [`Renderer::render`](crate::Renderer::render).
    fn view(&self, model: &Model, emitter: &Emitter<Event>) -> Props;
}
