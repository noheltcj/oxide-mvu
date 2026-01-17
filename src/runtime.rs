//! The MVU runtime that orchestrates the event loop.

#[cfg(feature = "no_std")]
use alloc::boxed::Box;

use core::future::Future;
use core::pin::Pin;

use async_channel::{bounded, Receiver};

use crate::Event as EventTrait;
use crate::{Emitter, MvuLogic, Renderer};

/// Default event channel capacity.
///
/// This bounds the number of events that can be queued before `emit()` drops events.
/// Sized for embedded systems with limited heap. Increase this via
/// [`MvuRuntimeBuilder::capacity`] if your application generates high-frequency bursts of events.
pub const DEFAULT_EVENT_CAPACITY: usize = 32;

/// Builder for configuring and constructing an [`MvuRuntime`].
///
/// Created via [`MvuRuntime::builder`]. Allows customizing runtime parameters
/// like event buffer capacity before building the runtime.
///
/// # Example
///
/// ```rust,no_run
/// # use oxide_mvu::{Emitter, Effect, MvuLogic, MvuRuntime, Renderer};
/// # #[derive(Clone)] enum Event {}
/// # #[derive(Clone)] struct Model;
/// # struct Props;
/// # struct MyLogic;
/// # impl MvuLogic<Event, Model, Props> for MyLogic {
/// #     fn init(&self, model: Model) -> (Model, Effect<Event>) { (model, Effect::none()) }
/// #     fn update(&self, _: Event, model: &Model) -> (Model, Effect<Event>) { (model.clone(), Effect::none()) }
/// #     fn view(&self, _: &Model, _: &Emitter<Event>) -> Props { Props }
/// # }
/// # struct MyRenderer;
/// # impl Renderer<Props> for MyRenderer { fn render(&mut self, _: Props) {} }
/// // Use builder for custom configuration
/// let runtime = MvuRuntime::builder(Model, MyLogic, MyRenderer, |_| {})
///     .capacity(64)
///     .build();
/// ```
pub struct MvuRuntimeBuilder<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    init_model: Model,
    logic: Logic,
    renderer: Render,
    spawner: Spawn,
    capacity: usize,
    _event: core::marker::PhantomData<Event>,
    _props: core::marker::PhantomData<Props>,
}

impl<Event, Model, Props, Logic, Render, Spawn>
    MvuRuntimeBuilder<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone + 'static,
    Props: 'static,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    /// Set the event buffer capacity.
    ///
    /// This bounds the number of events that can be queued before
    /// [`Emitter::try_emit`](crate::Emitter::try_emit) returns `false`.
    ///
    /// Defaults to [`DEFAULT_EVENT_CAPACITY`] (32).
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Build the runtime with the configured settings.
    pub fn build(self) -> MvuRuntime<Event, Model, Props, Logic, Render, Spawn> {
        let (event_sender, event_receiver) = bounded(self.capacity);
        let emitter = Emitter::new(event_sender);

        MvuRuntime {
            logic: self.logic,
            renderer: self.renderer,
            event_receiver,
            model: self.init_model,
            emitter,
            spawner: self.spawner,
            _props: core::marker::PhantomData,
        }
    }
}

/// A spawner trait for executing futures on an async runtime.
///
/// This abstraction allows you to use whatever concurrency model you want (tokio, async-std, embassy, etc.).
///
/// Function pointers and closures automatically implement this trait via the blanket implementation.
pub trait Spawner {
    /// Spawn a future on the async runtime.
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);
}

/// Implement Spawner for any callable type that matches the signature.
///
/// This includes function pointers, closures, and function items.
impl<F> Spawner for F
where
    F: Fn(Pin<Box<dyn Future<Output = ()> + Send>>),
{
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self(future)
    }
}

/// The MVU runtime that orchestrates the event loop.
///
/// This is the core of the framework. It:
/// 1. Initializes the Model and initial Effects via [`MvuLogic::init`]
/// 2. Processes events through [`MvuLogic::update`]
/// 3. Reduces the Model to Props via [`MvuLogic::view`]
/// 4. Delivers Props to the [`Renderer`] for rendering
///
/// The runtime creates a single [`Emitter`] that can send events from any thread.
/// Events are queued via a lock-free MPMC channel and processed on the thread where
/// [`MvuRuntime::run`] was called.
///
/// For testing with manual control, use [`TestMvuRuntime`] with a [`crate::TestRenderer`].
///
/// See the [crate-level documentation](crate) for a complete example.
///
/// # Type Parameters
///
/// * `Event` - The event type for your application
/// * `Model` - The model/state type for your application
/// * `Props` - The props type produced by the view function
/// * `Logic` - The logic implementation type (implements [`MvuLogic`])
/// * `Render` - The renderer implementation type (implements [`Renderer`])
/// * `Spawn` - The spawner implementation type (implements [`Spawner`])
pub struct MvuRuntime<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    logic: Logic,
    renderer: Render,
    event_receiver: Receiver<Event>,
    model: Model,
    emitter: Emitter<Event>,
    spawner: Spawn,
    _props: core::marker::PhantomData<Props>,
}

impl<Event, Model, Props, Logic, Render, Spawn>
    MvuRuntime<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone + 'static,
    Props: 'static,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{

    /// Create a builder for configuring the runtime.
    ///
    /// Use this when you need to customize runtime parameters like event buffer capacity.
    ///
    /// # Arguments
    ///
    /// * `init_model` - The initial state
    /// * `logic` - Application logic implementing MvuLogic
    /// * `renderer` - Platform rendering implementation for rendering Props
    /// * `spawner` - Spawner to execute async effects on your chosen runtime
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use oxide_mvu::{Emitter, Effect, MvuLogic, MvuRuntime, Renderer};
    /// # #[derive(Clone)] enum Event {}
    /// # #[derive(Clone)] struct Model;
    /// # struct Props;
    /// # struct MyLogic;
    /// # impl MvuLogic<Event, Model, Props> for MyLogic {
    /// #     fn init(&self, model: Model) -> (Model, Effect<Event>) { (model, Effect::none()) }
    /// #     fn update(&self, _: Event, model: &Model) -> (Model, Effect<Event>) { (model.clone(), Effect::none()) }
    /// #     fn view(&self, _: &Model, _: &Emitter<Event>) -> Props { Props }
    /// # }
    /// # struct MyRenderer;
    /// # impl Renderer<Props> for MyRenderer { fn render(&mut self, _: Props) {} }
    /// // For memory-constrained embedded systems
    /// let runtime = MvuRuntime::builder(Model, MyLogic, MyRenderer, |_| {})
    ///     .capacity(8)
    ///     .build();
    /// ```
    pub fn builder(
        init_model: Model,
        logic: Logic,
        renderer: Render,
        spawner: Spawn,
    ) -> MvuRuntimeBuilder<Event, Model, Props, Logic, Render, Spawn> {
        MvuRuntimeBuilder {
            init_model,
            logic,
            renderer,
            spawner,
            capacity: DEFAULT_EVENT_CAPACITY,
            _event: core::marker::PhantomData,
            _props: core::marker::PhantomData,
        }
    }

    /// Initialize the runtime and run the event processing loop.
    ///
    /// - Uses the MvuLogic::init function to create and enqueue initial side effects.
    /// - Reduces the initial Model provided at construction to Props via MvuLogic::view.
    /// - Renders the initial Props.
    /// - Processes events from the channel in a loop.
    ///
    /// This is an async function that runs the event loop. You can spawn it on your
    /// chosen runtime using the spawner, or await it directly.
    ///
    /// Events can be emitted from any thread via the Emitter, but are always processed
    /// sequentially on the thread where this future is awaited/polled.
    pub async fn run(mut self) {
        let (init_model, init_effect) = self.logic.init(self.model.clone());

        let initial_props = {
            let emitter = &self.emitter;
            self.logic.view(&init_model, emitter)
        };

        self.renderer.render(initial_props);

        // Execute initial effect by spawning it
        let emitter = self.emitter.clone();
        let future = init_effect.execute(&emitter);
        self.spawner.spawn(Box::pin(future));

        // Event processing loop
        while let Ok(event) = self.event_receiver.recv().await {
            self.step(event);
        }
    }

    fn step(&mut self, event: Event) {
        // Update model with event
        let (new_model, effect) = self.logic.update(event, &self.model);

        // Reduce to props and render
        let props = self.logic.view(&new_model, &self.emitter);
        self.renderer.render(props);

        // Update model
        self.model = new_model;

        // Execute the effect
        let emitter = self.emitter.clone();
        let future = effect.execute(&emitter);
        self.spawner.spawn(Box::pin(future));
    }
}

#[cfg(any(test, feature = "testing"))]
/// Test spawner function that executes futures synchronously.
///
/// This blocks on the future immediately rather than spawning it on an async runtime.
pub fn test_spawner_fn(fut: Pin<Box<dyn Future<Output = ()> + Send>>) {
    // Execute the future synchronously for deterministic testing
    futures::executor::block_on(fut);
}

#[cfg(any(test, feature = "testing"))]
/// Creates a test spawner that executes futures synchronously.
///
/// This is useful for testing - it blocks on the future immediately rather
/// than spawning it on an async runtime. Use this with [`TestMvuRuntime`]
/// or [`MvuRuntime`] in test scenarios.
///
/// Returns a function pointer that can be passed directly to runtime constructors
/// without heap allocation.
pub fn create_test_spawner() -> fn(Pin<Box<dyn Future<Output = ()> + Send>>) {
    test_spawner_fn
}

#[cfg(any(test, feature = "testing"))]
/// Test runtime driver for manual event processing control.
///
/// Only available with the `testing` feature or during tests.
///
/// Returned by [`TestMvuRuntime::run`]. Provides methods to manually
/// emit events and process the event queue for precise control in tests.
///
/// See [`TestMvuRuntime`] for usage.
pub struct TestMvuDriver<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone + 'static,
    Props: 'static,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    _runtime: TestMvuRuntime<Event, Model, Props, Logic, Render, Spawn>,
}

#[cfg(any(test, feature = "testing"))]
impl<Event, Model, Props, Logic, Render, Spawn>
    TestMvuDriver<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone + 'static,
    Props: 'static,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    /// Process all queued events.
    ///
    /// This processes events until the queue is empty. Call this after emitting
    /// events to drive the event loop in tests.
    pub fn process_events(&mut self) {
        self._runtime.process_queued_events();
    }
}

#[cfg(any(test, feature = "testing"))]
/// Test runtime for MVU with manual event processing control.
///
/// Only available with the `testing` feature or during tests.
///
/// Unlike [`MvuRuntime`], this runtime does not automatically
/// process events when they are emitted. Instead, tests must manually call
/// [`process_events`](TestMvuDriver::process_events) on the returned driver
/// to process the event queue.
///
/// This provides precise control over event timing in tests.
///
/// ```rust
/// use oxide_mvu::{Emitter, Effect, Renderer, MvuLogic, TestMvuRuntime};
/// # #[derive(Clone)]
/// # enum Event { Increment }
/// # #[derive(Clone)]
/// # struct Model { count: i32 }
/// # struct Props { count: i32, on_click: Box<dyn Fn()> }
/// # struct MyApp;
/// # impl MvuLogic<Event, Model, Props> for MyApp {
/// #     fn init(&self, model: Model) -> (Model, Effect<Event>) { (model, Effect::none()) }
/// #     fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>) {
/// #         (Model { count: model.count + 1 }, Effect::none())
/// #     }
/// #     fn view(&self, model: &Model, emitter: &Emitter<Event>) -> Props {
/// #         let e = emitter.clone();
/// #         Props { count: model.count, on_click: Box::new(move || { e.try_emit(Event::Increment); }) }
/// #     }
/// # }
/// # struct TestRenderer;
/// # impl Renderer<Props> for TestRenderer { fn render(&mut self, _props: Props) {} }
/// use oxide_mvu::create_test_spawner;
///
/// let runtime = TestMvuRuntime::builder(
///     Model { count: 0 },
///     MyApp,
///     TestRenderer,
///     create_test_spawner()
/// ).build();
/// let mut driver = runtime.run();
/// driver.process_events(); // Manually process events
/// ```
pub struct TestMvuRuntime<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone + 'static,
    Props: 'static,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    runtime: MvuRuntime<Event, Model, Props, Logic, Render, Spawn>,
}

#[cfg(any(test, feature = "testing"))]
/// Builder for configuring and constructing a [`TestMvuRuntime`].
///
/// Created via [`TestMvuRuntime::builder`]. Allows customizing runtime parameters
/// like event buffer capacity before building the test runtime.
pub struct TestMvuRuntimeBuilder<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone + 'static,
    Props: 'static,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    init_model: Model,
    logic: Logic,
    renderer: Render,
    spawner: Spawn,
    capacity: usize,
    _event: core::marker::PhantomData<Event>,
    _props: core::marker::PhantomData<Props>,
}

#[cfg(any(test, feature = "testing"))]
impl<Event, Model, Props, Logic, Render, Spawn>
    TestMvuRuntimeBuilder<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone + 'static,
    Props: 'static,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    /// Set the event buffer capacity.
    ///
    /// This bounds the number of events that can be queued before
    /// [`Emitter::try_emit`](crate::Emitter::try_emit) returns `false`.
    ///
    /// Defaults to [`DEFAULT_EVENT_CAPACITY`].
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Build the test runtime with the configured settings.
    pub fn build(self) -> TestMvuRuntime<Event, Model, Props, Logic, Render, Spawn> {
        let (event_sender, event_receiver) = bounded(self.capacity);

        TestMvuRuntime {
            runtime: MvuRuntime {
                logic: self.logic,
                renderer: self.renderer,
                event_receiver,
                model: self.init_model,
                emitter: Emitter::new(event_sender),
                spawner: self.spawner,
                _props: core::marker::PhantomData,
            },
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl<Event, Model, Props, Logic, Render, Spawn>
    TestMvuRuntime<Event, Model, Props, Logic, Render, Spawn>
where
    Event: EventTrait,
    Model: Clone + 'static,
    Props: 'static,
    Logic: MvuLogic<Event, Model, Props>,
    Render: Renderer<Props>,
    Spawn: Spawner,
{
    /// Create a builder for configuring the test runtime.
    ///
    /// Use this when you need to customize runtime parameters like event buffer capacity.
    ///
    /// # Arguments
    ///
    /// * `init_model` - The initial state
    /// * `logic` - Application logic implementing MvuLogic
    /// * `renderer` - Platform rendering implementation for rendering Props
    /// * `spawner` - Spawner to execute async effects on your chosen runtime
    pub fn builder(
        init_model: Model,
        logic: Logic,
        renderer: Render,
        spawner: Spawn,
    ) -> TestMvuRuntimeBuilder<Event, Model, Props, Logic, Render, Spawn> {
        TestMvuRuntimeBuilder {
            init_model,
            logic,
            renderer,
            spawner,
            capacity: DEFAULT_EVENT_CAPACITY,
            _event: core::marker::PhantomData,
            _props: core::marker::PhantomData,
        }
    }

    /// Initializes the runtime and returns a driver for manual event processing.
    ///
    /// This processes initial effects and renders the initial state, then returns
    /// a [`TestMvuDriver`] that provides manual control over event processing.
    pub fn run(mut self) -> TestMvuDriver<Event, Model, Props, Logic, Render, Spawn> {
        let (init_model, init_effect) = self.runtime.logic.init(self.runtime.model.clone());

        let initial_props = { self.runtime.logic.view(&init_model, &self.runtime.emitter) };

        self.runtime.renderer.render(initial_props);

        // Execute initial effect by spawning it
        let future = init_effect.execute(&self.runtime.emitter);
        self.runtime.spawner.spawn(Box::pin(future));

        TestMvuDriver { _runtime: self }
    }

    /// Process all queued events (for testing).
    ///
    /// This is exposed for TestMvuRuntime to manually drive event processing.
    fn process_queued_events(&mut self) {
        while let Ok(event) = self.runtime.event_receiver.try_recv() {
            self.runtime.step(event);
        }
    }
}
