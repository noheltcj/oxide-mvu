use super::{TestEvent, create_integration_test};

#[test]
fn given_no_initial_event_should_render_initial_props() {
    let (_driver, renderer) = create_integration_test(vec![]);

    assert_eq!(renderer.count(), 1);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
    });
}

#[test]
fn given_an_initial_increment_event_should_render_twice() {
    let (mut driver, renderer) = create_integration_test(vec![TestEvent::Increment]);

    driver.process_events();

    assert_eq!(renderer.count(), 2);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
        assert_eq!(renders[1].count, 1);
    });
}

#[test]
fn given_no_initial_event_when_props_callback_invoked_should_render_again() {
    let (mut driver, renderer) = create_integration_test(vec![]);

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
