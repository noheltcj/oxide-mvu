//! Service abstractions for testable side effects.
//!
//! By abstracting hardware and timing behind traits, we enable:
//! - Mock services in unit tests
//! - Swappable implementations for different hardware
//! - Isolated platform-specific code
//!
//! Each service trait provides methods that create `Effect<Event>` instances.
//! This allows tests to verify which effects were created without executing them.

use alloc::boxed::Box;
use embassy_nrf::gpio::{Input, Pull};
use embassy_nrf::Peri;
use embassy_nrf::peripherals::P0_11;
use embassy_time::{Duration, Timer};
use oxide_mvu::Effect;

use crate::app::Event;

/// Trait for time-based effects.
pub trait ClockEffects: Send + Sync + 'static {
    /// Create an effect that observes the system timer for periodic tick events.
    ///
    /// Returns an effect that emits `Event::Tick`.
    fn observe_ticks(&self) -> Effect<Event>;
}

/// Trait for button/input effects.
pub trait ButtonEffects: Send + Sync + 'static {
    /// Create an effect that observes button presses.
    ///
    /// Returns an effect that emits `Event::ButtonPress` when pressed.
    fn observe_button(&self) -> Effect<Event>;
}

// ============================================================================
// Implementations
// ============================================================================

/// Clock effects implementation using Embassy timers.
pub struct EmbassyClockEffects;

impl ClockEffects for EmbassyClockEffects {
    fn observe_ticks(&self) -> Effect<Event> {
        Effect::from_async(move |emitter| async move {
            loop {
                Timer::after(Duration::from_secs(3)).await;
                emitter.emit(Event::Tick).await;
            }
        })
    }
}

/// GPIO-based button effects using real hardware button.
///
/// Monitors Button 1 on P0.11 for press events.
pub struct GpioButtonEffects {
    button_pin: Peri<'static, P0_11>,
}

impl GpioButtonEffects {
    pub fn new(button_pin: Peri<'static, P0_11>) -> Self {
        Self { button_pin }
    }
}

impl ButtonEffects for GpioButtonEffects {
    fn observe_button(&self) -> Effect<Event> {
        // SAFETY: We're taking ownership of the pin in the effect closure.
        // This is safe because observe_button is only called once during init.
        let button_pin = unsafe { core::ptr::read(&self.button_pin) };

        Effect::from_async(|emitter| async move {
            // Configure button as input with pull-up (button is active-low)
            let mut button = Input::new(button_pin, Pull::Up);

            loop {
                // Wait for falling edge (button press)
                button.wait_for_low().await;

                // Wait for rising edge (button release)
                button.wait_for_high().await;

                emitter.emit(Event::ButtonPress).await;
            }
        })
    }
}

