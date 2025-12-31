use super::{TestEvent, create_integration_test};

#[test]
fn given_no_initial_event_should_render_initial_props() {
    let (_driver, renderer) = create_integration_test(vec![]);

    assert_eq!(renderer.count(), 1);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
    });
}