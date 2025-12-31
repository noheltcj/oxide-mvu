use super::{TestEvent, given_no_initial_event, given_an_initial_effect};

#[test]
fn given_no_initial_event_should_render_initial_props() {
    let (_driver, renderer) = given_no_initial_event();

    assert_eq!(renderer.count(), 1);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
    });
}