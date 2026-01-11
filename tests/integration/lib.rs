mod simple_logic;

pub(crate) use simple_logic::*;

use oxide_mvu::{create_test_spawner, Effect, TestMvuDriver, TestMvuRuntime, TestRenderer};

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

mod effect_dispatch_tests;
mod reduction_and_emission_tests;

pub(crate) struct IntegrationTestStubbing {
    mock_initial_effects_dependency: MockInitialEffectsDependency,
    mock_effects_dependency: MockEffectsDependency,
}

pub(crate) type TestDriver =
    TestMvuDriver<TestEvent, TestModel, TestProps, TestLogic, TestRenderer<TestProps>, fn(Pin<Box<dyn Future<Output = ()> + Send>>)>;

pub(crate) struct IntegrationTestHarness {
    pub(crate) driver: TestDriver,
    pub(crate) renders: TestRenderer<TestProps>,
    pub(crate) mock_initial_effects_dependency: Arc<Mutex<MockInitialEffectsDependency>>,
    pub(crate) mock_effects_dependency: Arc<Mutex<MockEffectsDependency>>,
}

struct ArcMutexWrapper<T: ?Sized>(Arc<Mutex<T>>);

impl InitialEffectsDependency for ArcMutexWrapper<MockInitialEffectsDependency> {
    fn on_init(&self) -> Effect<TestEvent> {
        self.0.lock().unwrap().on_init()
    }
}

impl EffectsDependency for ArcMutexWrapper<MockEffectsDependency> {
    fn on_increment_side_effect(&self) -> Effect<TestEvent> {
        self.0.lock().unwrap().on_increment_side_effect()
    }
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
            .return_once(Effect::none);

        self
    }

    pub(crate) fn given_a_noop_on_increment_side_effect(mut self) -> Self {
        self.mock_effects_dependency
            .expect_on_increment_side_effect()
            .returning(Effect::none);

        self
    }

    pub(crate) fn build(self) -> IntegrationTestHarness {
        self.create_integration_test_harness()
    }

    fn create_integration_test_harness(self) -> IntegrationTestHarness {
        let renderer = TestRenderer::new();
        let model = TestModel { count: 0 };

        let mock_initial_effects_arc = Arc::new(Mutex::new(self.mock_initial_effects_dependency));
        let mock_effects_arc = Arc::new(Mutex::new(self.mock_effects_dependency));

        let logic = TestLogic {
            initial_effects: Box::new(ArcMutexWrapper(mock_initial_effects_arc.clone())),
            effects: Box::new(ArcMutexWrapper(mock_effects_arc.clone())),
        };

        let runtime = TestMvuRuntime::new(model, logic, renderer.clone(), create_test_spawner());
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
    pub(crate) fn verify_initial_effects_dependency_checkpoint(&self) {
        self.mock_initial_effects_dependency
            .lock()
            .unwrap()
            .checkpoint();
    }

    pub(crate) fn verify_effects_dependency_checkpoint(&self) {
        self.mock_effects_dependency.lock().unwrap().checkpoint();
    }
}

pub(crate) fn build_integration_test() -> IntegrationTestStubbing {
    IntegrationTestStubbing {
        mock_initial_effects_dependency: MockInitialEffectsDependency::new(),
        mock_effects_dependency: MockEffectsDependency::new(),
    }
}
