mod simple_logic;

use std::sync::Arc;
use oxide_mvu::{create_test_spawner, Effect, TestMvuDriver, TestMvuRuntime, TestRenderer};
pub(crate) use simple_logic::*;

mod reduction_and_emission_tests;
mod effect_dispatch_tests;

pub(crate) fn given_an_initial_effect(
    effect: Effect<TestEvent>,
) -> (
    TestMvuDriver<TestEvent, TestModel, TestProps>,
    TestRenderer<TestProps>,
) {
    create_test_driver_and_renderer(
        TestCreationParameters {
            initial_effect: effect,
            on_increment_effect: Effect::none(),
        }
    )
}

pub(crate) fn given_no_initial_event() -> (
    TestMvuDriver<TestEvent, TestModel, TestProps>,
    TestRenderer<TestProps>,
) {
    create_test_driver_and_renderer(
        TestCreationParameters {
            initial_effect: Effect::none(),
            on_increment_effect: Effect::none(),
        }
    )
}

// Assumes Effect::none is returned by the runtime's init function
pub(crate) fn given_an_on_increment_side_effect(
    effect: Effect<TestEvent>,
) -> (
    TestMvuDriver<TestEvent, TestModel, TestProps>,
    TestRenderer<TestProps>,
) {
    create_test_driver_and_renderer(
        TestCreationParameters {
            initial_effect: Effect::none(),
            on_increment_effect: effect,
        }
    )
}

struct TestCreationParameters {
    initial_effect: Effect<TestEvent>,
    on_increment_effect: Effect<TestEvent>
}

fn create_test_driver_and_renderer(
    test_creation_parameters: TestCreationParameters
) -> (
    TestMvuDriver<TestEvent, TestModel, TestProps>,
    TestRenderer<TestProps>,
) {
    let renderer = TestRenderer::new();
    let model = TestModel { count: 0 };

    // Only acceptable in the context of integration testing.
    // TODO: implement unit tests that verify interactions.
    let mut mock_initial_effects = MockInitialEffectsDependency::new();
    mock_initial_effects.expect_on_init()
        .return_once(move || test_creation_parameters.initial_effect);

    let mut on_increment_side_effect = Arc::new(test_creation_parameters.on_increment_effect);
    let mut mock_effects = MockEffectsDependency::new();
    mock_effects.expect_on_increment_side_effect()
        .returning(move || { Effect::none() });

    let logic = Box::new(
        TestLogic {
            initial_effects: Box::new(mock_initial_effects),
            effects: Box::new(mock_effects)
        }
    );

    let runtime = TestMvuRuntime::new(model, logic, renderer.boxed(), create_test_spawner());
    let driver = runtime.run();

    (driver, renderer)
}