mod simple_logic;

use std::sync::Arc;
use oxide_mvu::{create_test_spawner, Effect, TestMvuDriver, TestMvuRuntime, TestRenderer};
pub(crate) use simple_logic::*;

mod effect_dispatch_tests;
mod reduction_and_emission_tests;

pub(crate) struct IntegrationTestStubbing {
    mock_initial_effects_dependency: MockInitialEffectsDependency,
    mock_effects_dependency: MockEffectsDependency
}

pub(crate) struct IntegrationTestHarness {
    driver: TestMvuDriver<TestEvent, TestModel, TestProps>,
    renders: TestRenderer<TestProps>,
    mock_initial_effects_dependency: Arc<MockInitialEffectsDependency>,
    mock_effects_dependency: Arc<MockEffectsDependency>
}

impl IntegrationTestStubbing {
    pub(crate) fn given_an_initial_effect(mut self, effect: Effect<TestEvent>) -> Self {
        self.mock_initial_effects_dependency
            .expect_on_init()
            .return_once(|| effect);

        self
    }

    pub(crate) fn given_no_initial_event(mut self) -> Self {
        self.mock_initial_effects_dependency
            .expect_on_init()
            .return_once(|| Effect::none());

        self
    }

    pub(crate) fn given_on_increment_effect_will_succeed(mut self) -> Self {
        self.mock_effects_dependency
            .expect_on_increment_side_effect()
            .returning(|| Effect::just(TestEvent::Increment));

        self
    }

    pub(crate) fn given_on_increment_has_no_side_effect(mut self) -> Self {
        self.mock_effects_dependency
            .expect_on_increment_side_effect()
            .returning(|| Effect::none());

        self
    }

    pub(crate) fn build(self) -> IntegrationTestHarness {
        self.create_integration_test_harness()
    }

    fn create_integration_test_harness(self) -> IntegrationTestHarness {
        let renderer = TestRenderer::new();
        let model = TestModel { count: 0 };

        let mock_initial_effects_arc = Arc::new(self.mock_initial_effects_dependency);
        let mock_effects_arc = Arc::new(self.mock_effects_dependency);

        let logic = Box::new(TestLogic {
            initial_effects: Box::new(*mock_initial_effects_arc.clone()),
            effects: Box::new(*mock_effects_arc.clone()),
        });

        let runtime = TestMvuRuntime::new(model, logic, renderer.boxed(), create_test_spawner());
        let driver = runtime.run();

        IntegrationTestHarness {
            driver,
            renders: renderer,
            mock_initial_effects_dependency: mock_initial_effects_arc,
            mock_effects_dependency: mock_effects_arc,
        }
    }
}

impl IntegrationTestHarness {

}

pub(crate) fn build_integration_test() -> IntegrationTestStubbing {
    IntegrationTestStubbing {
        mock_initial_effects_dependency: MockInitialEffectsDependency::new(),
        mock_effects_dependency: MockEffectsDependency::new()
    }
}
