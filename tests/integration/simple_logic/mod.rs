use oxide_mvu::{Effect, Emitter, MvuLogic};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TestEvent {
    Increment,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TestModel {
    pub(crate) count: i32,
}

pub(crate) struct TestProps {
    pub(crate) count: i32,
    pub(crate) on_increment: Box<dyn Fn() + Send>,
}

pub(crate) struct TestLogic {
    pub(crate) initial_effects: Box<dyn InitialEffectsDependency + Send>,
    pub(crate) effects: Box<dyn EffectsDependency + Send>,
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait InitialEffectsDependency {
    fn on_init(&self) -> Effect<TestEvent>;
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait EffectsDependency {
    fn on_increment_side_effect(&self) -> Effect<TestEvent>;
}

impl MvuLogic<TestEvent, TestModel, TestProps> for TestLogic {
    fn init(&self, model: TestModel) -> (TestModel, Effect<TestEvent>) {
        let effect = self.initial_effects.on_init();
        (model, effect)
    }

    fn update(&self, event: TestEvent, model: &TestModel) -> (TestModel, Effect<TestEvent>) {
        match event {
            TestEvent::Increment => {
                let new_model = TestModel {
                    count: model.count + 1,
                };
                (new_model, self.effects.on_increment_side_effect())
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
