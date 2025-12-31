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
    pub(crate) initial_events: Vec<TestEvent>,
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
                    .collect(),
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
