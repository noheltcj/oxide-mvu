//! Tests for event buffer capacity configuration.

use super::build_integration_test;

#[test]
fn given_capacity_of_2_when_emitting_3_events_without_processing_should_drop_third_event() {
    let mut test = build_integration_test()
        .given_no_initial_event()
        .given_a_noop_on_increment_side_effect()
        .with_event_buffer_capacity(2)
        .build();

    // First two emissions should succeed (buffer has capacity 2)
    // Third emission should fail (buffer is full)
    let (first_ok, second_ok, third_ok) = test.renders.with_renders(|renders| {
        let initial_props = renders.first().unwrap();
        let first = (initial_props.on_increment)();
        let second = (initial_props.on_increment)();
        let third = (initial_props.on_increment)();
        (first, second, third)
    });

    assert!(first_ok, "First emit should succeed");
    assert!(second_ok, "Second emit should succeed");
    assert!(!third_ok, "Third emit should fail when buffer is full");

    // Process all queued events
    test.driver.process_events();

    // Verify only 2 events were processed (third was dropped)
    test.renders.with_renders(|renders| {
        let final_props = renders.last().unwrap();
        assert_eq!(
            final_props.count, 2,
            "Only 2 increments should have been processed"
        );
    });
}

#[test]
fn given_capacity_of_2_when_processing_between_emissions_should_allow_more_events() {
    let mut test = build_integration_test()
        .given_no_initial_event()
        .given_a_noop_on_increment_side_effect()
        .with_event_buffer_capacity(2)
        .build();

    // Emit 2 events (fills buffer)
    test.renders.with_renders(|renders| {
        let initial_props = renders.first().unwrap();
        assert!((initial_props.on_increment)());
        assert!((initial_props.on_increment)());
    });

    // Process events to free up buffer space
    test.driver.process_events();

    // Now we should be able to emit more
    let (third_ok, fourth_ok) = test.renders.with_renders(|renders| {
        let props = renders.last().unwrap();
        let third = (props.on_increment)();
        let fourth = (props.on_increment)();
        (third, fourth)
    });

    assert!(third_ok, "Should be able to emit after processing");
    assert!(
        fourth_ok,
        "Should be able to emit second event after processing"
    );

    // Process remaining events
    test.driver.process_events();

    test.renders.with_renders(|renders| {
        let final_props = renders.last().unwrap();
        assert_eq!(
            final_props.count, 4,
            "All 4 increments should have been processed"
        );
    });
}

#[test]
fn given_default_capacity_should_handle_many_events() {
    let mut test = build_integration_test()
        .given_no_initial_event()
        .given_a_noop_on_increment_side_effect()
        .build();

    // Should be able to emit up to DEFAULT_EVENT_CAPACITY (32) events without processing
    // Event 33 should fail
    let emit_results = test.renders.with_renders(|renders| {
        let initial_props = renders.first().unwrap();
        let mut results = Vec::new();
        for _ in 0..33 {
            results.push((initial_props.on_increment)());
        }
        results
    });

    for (i, result) in emit_results.iter().enumerate().take(32) {
        assert!(
            result,
            "Emit {} should succeed with default capacity",
            i + 1
        );
    }
    assert!(
        !emit_results[32],
        "Emit beyond default capacity should fail"
    );

    test.driver.process_events();

    test.renders.with_renders(|renders| {
        let final_props = renders.last().unwrap();
        assert_eq!(
            final_props.count, 32,
            "32 increments should have been processed"
        );
    });
}
