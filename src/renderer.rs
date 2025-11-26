//! Renderer abstraction for rendering Props.

/// Renderer abstraction for rendering Props.
///
/// Implement this trait to integrate oxide-mvu just your rendering system
/// (UI framework, terminal, embedded display, etc.).
///
/// The [`render`](Self::render) method is called whenever the model changes, receiving
/// fresh Props derived from the current state via [`MvuLogic::view`](crate::MvuLogic::view).
///
/// # Example
///
/// ```rust
/// use oxide_mvu::Renderer;
///
/// struct Props {
///     message: &'static str,
/// }
///
/// struct ConsoleRenderer;
///
/// impl Renderer<Props> for ConsoleRenderer {
///     fn render(&mut self, props: Props) {
///         println!("{}", props.message);
///     }
/// }
/// ```
pub trait Renderer<Props> {
    /// Render the given props.
    ///
    /// This is where you integrate just your rendering system. Props may
    /// contain callbacks (via [`Emitter`](crate::Emitter)) that can trigger new events.
    ///
    /// # Arguments
    ///
    /// * `props` - The props to render, derived from the current model state
    fn render(&mut self, props: Props);
}
