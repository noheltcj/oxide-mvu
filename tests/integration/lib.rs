mod simple_logic;

use oxide_mvu::{Effect, TestMvuDriver, TestMvuRuntime, TestRenderer};
pub(crate) use simple_logic::*;

mod reduction_and_emission_tests;
mod effect_dispatch_tests;

// Test helper that returns both runtime driver and renderer for interactive testing
pub(crate) fn create_integration_test(
    initial_events: Vec<TestEvent>,
) -> (
    TestMvuDriver<TestEvent, TestModel, TestProps>,
    TestRenderer<TestProps>,
) {
    let renderer = TestRenderer::new();
    let model = TestModel { count: 0 };
    let logic = Box::new(TestLogic { initial_events });

    let runtime = TestMvuRuntime::new(model, logic, renderer.boxed());
    let driver = runtime.run();

    (driver, renderer)
}

// Test helper that returns both runtime driver and renderer for interactive testing
pub(crate) fn given_initial_effect(
    effect: Effect<TestEvent>,
) -> (
    TestMvuDriver<TestEvent, TestModel, TestProps>,
    TestRenderer<TestProps>,
) {
    let renderer = TestRenderer::new();
    let model = TestModel { count: 0 };
    let logic = Box::new(TestLogic { initial_events });

    let runtime = TestMvuRuntime::new(model, logic, renderer.boxed());
    let driver = runtime.run();

    (driver, renderer)
}