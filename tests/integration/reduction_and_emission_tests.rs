use oxide_mvu::Effect;
use super::{TestEvent, given_an_initial_effect, given_no_initial_event};

#[test]
fn given_no_initial_event_should_render_initial_props() {
    let (_driver, renderer) = given_no_initial_event();

    assert_eq!(renderer.count(), 1);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
    });
}

#[test]
fn given_an_increment_effect_on_init_when_processed_should_render_again_with_incremented_count() {
    let (mut driver, renderer) =
        given_an_initial_effect(Effect::just(TestEvent::Increment));

    driver.process_events();

    assert_eq!(renderer.count(), 2);
    renderer.with_renders(|renders| {
        assert_eq!(renders[1].count, 1);
    });
}

#[test]
fn given_two_batched_increment_effects_on_init_when_processed_should_render_a_third_time_with_incremented_count() {
    let (mut driver, renderer) =
        given_an_initial_effect(
            Effect::batch(
                vec![
                    Effect::just(TestEvent::Increment),
                    Effect::just(TestEvent::Increment),
                ]
            )
        );

    driver.process_events();

    assert_eq!(renderer.count(), 3);
    renderer.with_renders(|renders| {
        assert_eq!(renders[2].count, 2);
    });
}

#[test]
fn given_no_initial_event_when_props_callback_invoked_should_render_again() {
    let (mut driver, renderer) = given_no_initial_event();

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
