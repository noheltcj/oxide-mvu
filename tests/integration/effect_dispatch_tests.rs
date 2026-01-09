use std::sync::Arc;
use super::{build_integration_test, TestEvent};
use oxide_mvu::Effect;

#[test]
fn given_no_initial_event_should_render_initial_props() {
    let test = build_integration_test()
        .given_no_initial_event()
        .build();

    assert_eq!(test.renders.count(), 1);
    test.renders.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
    });
}

#[test]
fn given_an_on_increment_side_effect_when_increment_triggered_should_execute_side_effect() {
    let mut test = build_integration_test()
        .given_no_initial_event()
        .given_on_increment_has_no_side_effect()
        .build();

    // Trigger an increment event via props callback
    test.renders.with_renders(|renders| {
        (renders[0].on_increment)();
    });

    // Process the increment event
    test.driver.process_events();

    // Verify on_increment_side_effect
    test.mock_effects_dependency.checkpoint();

    // The on_increment_side_effect should have triggered another increment
    // So we should have 2 renders total:
    // 1. Initial render (count=0)
    // 2. After on_increment (count=1)
    assert_eq!(test.renders.count(), 2);
    test.renders.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
        assert_eq!(renders[1].count, 1);
    });
}

#[test]
fn given_a_batch_of_effects_as_initial_effect_should_execute_all_effects() {
    let mut test = build_integration_test()
        .given_an_initial_effect(Effect::batch(vec![
            Effect::just(TestEvent::Increment),
            Effect::just(TestEvent::Increment),
            Effect::just(TestEvent::Increment),
        ]))
        .given_on_increment_has_no_side_effect()
        .build();

    test.driver.process_events();

    // Should have 4 renders total:
    // 1. Initial render (count=0)
    // 2. After first increment (count=1)
    // 3. After second increment (count=2)
    // 4. After third increment (count=3)
    assert_eq!(test.renders.count(), 4);
    test.renders.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
        assert_eq!(renders[1].count, 1);
        assert_eq!(renders[2].count, 2);
        assert_eq!(renders[3].count, 3);
    });
}