//! The MVU runtime that orchestrates the event loop.

#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use portable_atomic_util::Arc;
use spin::Mutex;

use crate::{Emitter, Effect, Renderer, MvuLogic};

/// Internal state for the MVU runtime.
struct RuntimeState<Event: Send, Model: Clone + Send> {
    model: Model,
    event_queue: Vec<Event>,
    effects_queue: Vec<Effect<Event>>
}

/// The MVU runtime that orchestrates the event loop.
///
/// This is the core of the framework. It:
/// 1. Initializes the Model and initial Effects via [`MvuLogic::init`]
/// 2. Processes events through [`MvuLogic::update`]
/// 3. Reduces the Model to Props via [`MvuLogic::view`]
/// 4. Delivers Props to the [`Renderer`] for rendering
///
/// The runtime creates a single [`Emitter`] that automatically processes events
/// when [`Emitter::emit`] is called, regardless of which thread it's called from.
/// Events are processed synchronously in a thread-safe manner.
///
/// For testing with manual control, use [`TestMvuRuntime`] with a [`crate::TestRenderer`].
///
/// See the [crate-level documentation](crate) for a complete example.
pub struct MvuRuntime<Event: Send, Model: Clone + Send, Props> {
    logic: Box<dyn MvuLogic<Event, Model, Props> + Send>,
    renderer: Box<dyn Renderer<Props> + Send>,
    state: Arc<Mutex<RuntimeState<Event, Model>>>,
    emitter: Emitter<Event>,
}

impl<Event: Send + 'static, Model: Clone + Send + 'static, Props: 'static> MvuRuntime<Event, Model, Props> {
    /// Create a new runtime.
    ///
    /// The runtime will not be started until MvuRuntime::run is called.
    ///
    /// # Arguments
    ///
    /// * `init_model` - The initial state
    /// * `logic` - Application logic implementing MvuLogic
    /// * `renderer` - Platform rendering implementation for rendering Props
    pub fn new(
        init_model: Model,
        logic: Box<dyn MvuLogic<Event, Model, Props> + Send>,
        renderer: Box<dyn Renderer<Props> + Send>,
    ) -> Self {
        // Create state and emitter that enqueues to the state's event queue
        let state = Arc::new(Mutex::new(RuntimeState {
            model: init_model,
            event_queue: Vec::new(),
            effects_queue: Vec::new()
        }));

        let state_clone = state.clone();
        let emitter = Emitter::new(move |event| {
            state_clone.lock().event_queue.push(event);
        });

        MvuRuntime { logic, renderer, state, emitter }
    }

    /// Initialize the runtime loop.
    ///
    /// - Uses the MvuLogic::init function to create and enqueue initial side effects.
    /// - Reduces the initial Model provided at construction to Props via MvuLogic::view.
    /// - Renders the initial Props.
    pub fn run(mut self) {
        // Initialize the model and get initial effects
        let init_model = {
            let mut runtime_state = self.state.lock();
            let (init_model, init_effect) = {
                let model = runtime_state.model.clone();
                self.logic.init(model)
            };

            // Update model and queue effects
            runtime_state.model = init_model;
            runtime_state.effects_queue.push(init_effect);

            runtime_state.model.clone()
        };

        let initial_props = {
            let emitter = self.emitter;
            self.logic.view(&init_model, &emitter)
        };

        self.renderer.render(initial_props);
    }

    #[cfg(any(test, feature = "testing"))]
    fn step(&mut self, event: Event) {
        // Reduce event and render props
        let (model, effect, props) = self.reduce_event(event);

        self.renderer.render(props);

        // Update model
        {
            let state_mutex = self.state.clone();
            let mut runtime_state = state_mutex.lock();
            runtime_state.model = model;
        }

        // Execute the effect (which may enqueue more events)
        effect.execute(&self.emitter);

        // Process any newly queued events
        self.process_queued_events()
    }

    #[cfg(any(test, feature = "testing"))]
    /// Dispatch a single event through update -> view -> render.
    fn reduce_event(&self, event: Event) -> (Model, Effect<Event>, Props) {
        // Update model just event
        let (new_model, effect) = {
            let runtime_state = self.state.lock();
            self.logic.update(event, &runtime_state.model)
        };

        // Reduce the new model and emitter to props
        let emitter = &self.emitter;
        let props = self.logic.view(&new_model, emitter);

        (new_model, effect, props)
    }

    #[cfg(any(test, feature = "testing"))]
    /// Process all queued events (for testing).
    ///
    /// This is exposed for TestMvuRuntime to manually drive event processing.
    fn process_queued_events(&mut self) {
        loop {
            let state_mutex = self.state.clone();
            let next_event = {
                let mut runtime_state = state_mutex.lock();
                if runtime_state.event_queue.is_empty() {
                    break;
                }
                runtime_state.event_queue.remove(0)
            }; // Lock is dropped here
            self.step(next_event);
        }
    }
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
pub struct TestMvuDriver<Event: Send + 'static, Model: Clone + Send + 'static, Props: 'static> {
    _runtime: MvuRuntime<Event, Model, Props>,
}

#[cfg(any(test, feature = "testing"))]
impl<Event: Send + 'static, Model: Clone + Send + 'static, Props: 'static> TestMvuDriver<Event, Model, Props> {
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
/// # enum Event { Increment }
/// # #[derive(Clone)]
/// # struct Model { count: i32 }
/// # struct Props { count: i32, on_click: Box<dyn Fn() + Send> }
/// # struct MyApp;
/// # impl MvuLogic<Event, Model, Props> for MyApp {
/// #     fn init(&self, model: Model) -> (Model, Effect<Event>) { (model, Effect::none()) }
/// #     fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>) {
/// #         (Model { count: model.count + 1 }, Effect::none())
/// #     }
/// #     fn view(&self, model: &Model, emitter: &Emitter<Event>) -> Props {
/// #         let e = emitter.clone();
/// #         Props { count: model.count, on_click: Box::new(move || e.emit(Event::Increment)) }
/// #     }
/// # }
/// # struct TestRenderer;
/// # impl Renderer<Props> for TestRenderer { fn render(&mut self, _props: Props) {} }
///
/// let runtime = TestMvuRuntime::new(
///     Model { count: 0 },
///     Box::new(MyApp),
///     Box::new(TestRenderer)
/// );
/// let mut driver = runtime.run();
/// driver.process_events(); // Manually process events
/// ```
pub struct TestMvuRuntime<Event: Send + 'static, Model: Clone + Send + 'static, Props: 'static> {
    runtime: MvuRuntime<Event, Model, Props>,
}

#[cfg(any(test, feature = "testing"))]
impl<Event: Send + 'static, Model: Clone + Send + 'static, Props: 'static> TestMvuRuntime<Event, Model, Props> {
    /// Create a new test runtime.
    ///
    /// Creates an emitter that enqueues events without automatically processing them.
    pub fn new(
        init_model: Model,
        logic: Box<dyn MvuLogic<Event, Model, Props> + Send>,
        renderer: Box<dyn Renderer<Props> + Send>,
    ) -> Self {
        // Create state and emitter that enqueues to the state's event queue
        let state = Arc::new(Mutex::new(RuntimeState {
            model: init_model,
            event_queue: Vec::new(),
            effects_queue: Vec::new()
        }));

        let state_clone = state.clone();
        let emitter = Emitter::new(move |event| {
            state_clone.lock().event_queue.push(event);
        });

        TestMvuRuntime {
            runtime: MvuRuntime { logic, renderer, state, emitter },
        }
    }

    /// Initializes the runtime and returns a driver for manual event processing.
    ///
    /// This processes initial effects and renders the initial state, then returns
    /// a [`TestMvuDriver`] that provides manual control over event processing.
    pub fn run(mut self) -> TestMvuDriver<Event, Model, Props> {
        // Initialize the model and get initial effects
        let init_model = {
            let mut runtime_state = self.runtime.state.lock();
            let (init_model, init_effect) = {
                let model = runtime_state.model.clone();
                self.runtime.logic.init(model)
            };

            // Update model and queue effects
            runtime_state.model = init_model;
            runtime_state.effects_queue.push(init_effect);

            runtime_state.model.clone()
        };

        let initial_props = {
            let emitter = &self.runtime.emitter;
            self.runtime.logic.view(&init_model, emitter)
        };

        self.runtime.renderer.render(initial_props);

        // Process initial effects by executing them with the emitter
        {
            let mut runtime_state = self.runtime.state.lock();
            let effects = runtime_state.effects_queue.drain(..).collect::<Vec<_>>();
            drop(runtime_state);

            for effect in effects {
                effect.execute(&self.runtime.emitter);
            }
        }

        TestMvuDriver {
            _runtime: self.runtime,
        }
    }
}
