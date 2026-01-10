//! Renderer abstraction for rendering Props.

#[cfg(any(test, feature = "testing"))]
#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(any(test, feature = "testing"))]
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

#[cfg(any(test, feature = "testing"))]
use portable_atomic_util::Arc;
#[cfg(any(test, feature = "testing"))]
use spin::Mutex;

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

#[cfg(any(test, feature = "testing"))]
/// Test renderer that captures all rendered Props for assertions.
///
/// Only available with the `testing` feature.
///
/// Use this with [`TestMvuRuntime`](crate::TestMvuRuntime) to capture and inspect
/// Props in integration tests.
///
/// # Example
///
/// ```rust
/// use oxide_mvu::{create_test_spawner, TestRenderer, TestMvuRuntime, MvuLogic, Effect, Emitter};
///
/// # struct Props { count: i32 }
/// #
/// # #[derive(Clone)]
/// # struct Model { count: i32 }
/// #
/// # enum Event { Inc }
/// #
/// # struct Logic;
/// #
/// # impl MvuLogic<Event, Model, Props> for Logic {
/// #     fn init(&self, m: Model) -> (Model, Effect<Event>) { (m, Effect::none()) }
/// #     fn update(&self, _e: Event, m: &Model) -> (Model, Effect<Event>) {
/// #         (Model { count: m.count + 1 }, Effect::none())
/// #     }
/// #     fn view(&self, m: &Model, _: &Emitter<Event>) -> Props {
/// #         Props { count: m.count }
/// #     }
/// # }
/// // Create a TestRenderer for props assertions
/// let renderer = TestRenderer::new();
///
/// // Construct a TestMvuRuntime using the renderer
/// let runtime = TestMvuRuntime::new(
///     Model { count: 0 },
///     Logic,
///     renderer.clone(),
///     create_test_spawner()
/// );
///
/// let driver = runtime.run();
///
/// // Use renderer to inspect renders
/// renderer.with_renders(|renders| {
///     assert_eq!(renders[0].count, 0);
/// });
/// ```
pub struct TestRenderer<Props> {
    renders: Arc<Mutex<Vec<Props>>>,
}

#[cfg(any(test, feature = "testing"))]
struct InternalTestRenderer<Props> {
    renders: Arc<Mutex<Vec<Props>>>,
}

#[cfg(any(test, feature = "testing"))]
impl<Props> Renderer<Props> for InternalTestRenderer<Props> {
    fn render(&mut self, props: Props) {
        self.renders.lock().push(props);
    }
}

#[cfg(any(test, feature = "testing"))]
impl<Props> Clone for TestRenderer<Props> {
    fn clone(&self) -> Self {
        Self {
            renders: self.renders.clone(),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl<Props> Renderer<Props> for TestRenderer<Props> {
    fn render(&mut self, props: Props) {
        self.renders.lock().push(props);
    }
}

#[cfg(any(test, feature = "testing"))]
impl<Props: 'static + Send> Default for TestRenderer<Props> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(test, feature = "testing"))]
impl<Props: 'static + Send> TestRenderer<Props> {
    pub fn new() -> Self {
        Self {
            renders: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a boxed renderer to pass to the MVU runtime.
    ///
    /// The returned renderer shares the same capture storage as this TestRenderer,
    /// so you can use [`with_renders`](Self::with_renders) to inspect captured Props.
    pub fn boxed(&self) -> Box<dyn Renderer<Props> + Send> {
        Box::new(InternalTestRenderer {
            renders: self.renders.clone(),
        })
    }

    /// Get the number of renders that have occurred.
    pub fn count(&self) -> usize {
        self.renders.lock().len()
    }

    /// Access the captured renders with a closure.
    ///
    /// The closure receives a reference to the Vec of all captured Props.
    /// This allows you to make assertions on Props emissions or execute
    /// callbacks for further testing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use oxide_mvu::TestRenderer;
    /// # struct Props { count: i32, on_click: Box<dyn Fn() + Send> }
    /// # let renderer = TestRenderer::<Props>::new();
    ///
    /// // Compute render count
    /// let count = renderer.with_renders(|renders| renders.len());
    ///
    /// // Make Props assertions
    /// renderer.with_renders(|renders| {
    ///     // assert_eq!(renders[0].count, 42);
    /// });
    ///
    /// // Execute a specific Props callback
    /// renderer.with_renders(|renders| {
    ///     // (renders[0].on_click)();
    /// });
    /// ```
    pub fn with_renders<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Vec<Props>) -> R,
    {
        let renders = self.renders.lock();
        f(&renders)
    }
}
