use oxide_mvu::{Emitter, Effect, TestMvuRuntime, TestMvuDriver, MvuLogic, Renderer};

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use portable_atomic_util::Arc;
use spin::Mutex;

#[derive(Clone, Debug, PartialEq)]
enum TestEvent {
    Increment,
}

#[derive(Clone, Debug, PartialEq)]
struct TestModel {
    count: i32,
}

struct TestProps {
    count: i32,
    on_increment: Box<dyn Fn() + Send>,
}

struct TestLogic {
    initial_events: Vec<TestEvent>
}

impl MvuLogic<TestEvent, TestModel, TestProps> for TestLogic {
    fn init(&self, model: TestModel) -> (TestModel, Effect<TestEvent>) {
        let effect = if self.initial_events.is_empty() {
            Effect::none()
        } else {
            Effect::batch(
                self.initial_events
                    .iter()
                    .map(|event| Effect::just(event.clone()))
                    .collect()
            )
        };
        (model, effect)
    }

    fn update(&self, event: TestEvent, model: &TestModel) -> (TestModel, Effect<TestEvent>) {
        match event {
            TestEvent::Increment => {
                let new_model = TestModel {
                    count: model.count + 1,
                };
                (new_model, Effect::none())
            }
        }
    }

    fn view(&self, model: &TestModel, emitter: &Emitter<TestEvent>) -> TestProps {
        let emitter = emitter.clone();
        TestProps {
            count: model.count,
            on_increment: Box::new(move || {
                emitter.emit(TestEvent::Increment);
            }),
        }
    }
}

struct TestRenderer {
    renders: Arc<Mutex<Vec<TestProps>>>,
}

impl Renderer<TestProps> for TestRenderer {
    fn render(&mut self, props: TestProps) {
        // Simply capture each Props emission for later analysis
        self.renders.lock().push(props);
    }
}

// Test helper that runs and returns both driver and renders
fn run_test(initial_events: Vec<TestEvent>) -> (TestMvuDriver<TestEvent, TestModel, TestProps>, Arc<Mutex<Vec<TestProps>>>) {
    let renders = Arc::new(Mutex::new(Vec::new()));
    let renderer = Box::new(TestRenderer {
        renders: renders.clone(),
    });

    let model = TestModel { count: 0 };
    let logic = Box::new(TestLogic { initial_events });

    let runtime = TestMvuRuntime::new(model, logic, renderer);
    let driver = runtime.run();

    (driver, renders)
}

// Test helper that returns both runtime driver and renders for interactive testing
fn setup_test(initial_events: Vec<TestEvent>) -> (TestMvuDriver<TestEvent, TestModel, TestProps>, Arc<Mutex<Vec<TestProps>>>) {
    let renders = Arc::new(Mutex::new(Vec::new()));
    let renderer = Box::new(TestRenderer {
        renders: renders.clone(),
    });

    let model = TestModel { count: 0 };
    let logic = Box::new(TestLogic { initial_events });

    let runtime = TestMvuRuntime::new(model, logic, renderer);
    let driver = runtime.run();

    (driver, renders)
}

#[test]
fn given_no_initial_effect_when_ran_should_render_initial_props() {
    let (_driver, renders) = run_test(vec![]);

    let final_renders = renders.lock();
    let counter_values: Vec<i32> = final_renders.iter().map(|props| props.count).collect();
    assert_eq!(counter_values, vec![0]);
}

#[test]
fn given_an_initial_increment_event_when_ran_should_render_twice() {
    let (mut driver, renders) = run_test(vec![TestEvent::Increment]);

    // Process events emitted by initial effects
    driver.process_events();

    let final_renders = renders.lock();
    let counter_values: Vec<i32> = final_renders.iter().map(|props| props.count).collect();
    assert_eq!(counter_values, vec![0, 1]);
}

#[test]
fn given_props_with_callback_when_invoked_should_render_again() {
    let (mut driver, renders) = setup_test(vec![]);

    // Call the callback from the first render
    {
        let initial_renders = renders.lock();
        (initial_renders[0].on_increment)();
    }

    // TODO: The primary runtime should process events on its own.
    // Process the event triggered by the callback
    driver.process_events();

    // Verify new render was emitted just incremented count
    let final_renders = renders.lock();
    assert_eq!(final_renders.len(), 2);
    assert_eq!(final_renders[1].count, 1);
}