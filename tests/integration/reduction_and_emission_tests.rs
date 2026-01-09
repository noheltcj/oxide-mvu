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
fn given_an_increment_effect_on_init_when_processed_should_render_again_with_incremented_count() {
    let mut test = build_integration_test()
        .given_an_initial_effect(Effect::just(TestEvent::Increment))
        .given_on_increment_has_no_side_effect()
        .build();

    test.driver.process_events();

    assert_eq!(test.renders.count(), 2);
    test.renders.with_renders(|renders| {
        assert_eq!(renders[1].count, 1);
    });
}

#[test]
fn given_two_batched_increment_effects_on_init_when_processed_should_render_a_third_time_with_incremented_count(
) {
    let mut test = build_integration_test()
        .given_an_initial_effect(Effect::batch(vec![
            Effect::just(TestEvent::Increment),
            Effect::just(TestEvent::Increment),
        ]))
        .given_on_increment_has_no_side_effect()
        .build();

    test.driver.process_events();

    assert_eq!(test.renders.count(), 3);
    test.renders.with_renders(|renders| {
        assert_eq!(renders[2].count, 2);
    });
}

#[test]
fn given_no_initial_event_when_props_callback_invoked_should_render_again() {
    let mut test = build_integration_test()
        .given_no_initial_event()
        .given_on_increment_has_no_side_effect()
        .build();

    test.renders.with_renders(|renders| {
        (renders[0].on_increment)();
    });

    test.driver.process_events();

    // Verify new render was emitted with incremented count
    assert_eq!(test.renders.count(), 2);
    test.renders.with_renders(|renders| {
        assert_eq!(renders[1].count, 1);
    });
}
