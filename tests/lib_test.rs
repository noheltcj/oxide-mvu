use oxide_mvu::{Emitter, Effect, TestMvuRuntime, TestMvuDriver, MvuLogic, TestRenderer};

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

// Test helper that runs and returns both driver and renderer
fn run_test(initial_events: Vec<TestEvent>) -> (TestMvuDriver<TestEvent, TestModel, TestProps>, TestRenderer<TestProps>) {
    let renderer = TestRenderer::new();
    let model = TestModel { count: 0 };
    let logic = Box::new(TestLogic { initial_events });

    let runtime = TestMvuRuntime::new(model, logic, renderer.boxed());
    let driver = runtime.run();

    (driver, renderer)
}

// Test helper that returns both runtime driver and renderer for interactive testing
fn setup_test(initial_events: Vec<TestEvent>) -> (TestMvuDriver<TestEvent, TestModel, TestProps>, TestRenderer<TestProps>) {
    let renderer = TestRenderer::new();
    let model = TestModel { count: 0 };
    let logic = Box::new(TestLogic { initial_events });

    let runtime = TestMvuRuntime::new(model, logic, renderer.boxed());
    let driver = runtime.run();

    (driver, renderer)
}

#[test]
fn given_no_initial_effect_when_ran_should_render_initial_props() {
    let (_driver, renderer) = run_test(vec![]);

    assert_eq!(renderer.count(), 1);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
    });
}

#[test]
fn given_an_initial_increment_event_when_ran_should_render_twice() {
    let (mut driver, renderer) = run_test(vec![TestEvent::Increment]);

    driver.process_events();

    assert_eq!(renderer.count(), 2);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
        assert_eq!(renders[1].count, 1);
    });
}

#[test]
fn given_initial_props_when_invoked_should_render_again() {
    let (mut driver, renderer) = setup_test(vec![]);

    renderer.with_renders(|renders| {
        (renders[0].on_increment)();
    });

    driver.process_events();

    // Verify new render was emitted with incremented count
    assert_eq!(renderer.count(), 2);
    renderer.with_renders(|renders| {
        assert_eq!(renders[1].count, 1);
    });
}