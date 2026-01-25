use alloc::vec;
use oxide_mvu::{Effect, Emitter, MvuLogic};

use crate::effects::{ButtonEffects, ClockEffects};

/// Events that can occur in the system.
#[derive(Clone, Debug)]
pub enum Event {
    /// Periodic timer tick (emitted every 3 seconds).
    Tick,
    /// Button press event (emitted on physical button press).
    ButtonPress,
}

/// Application state.
///
/// This is the single source of truth for the entire application.
/// It is only modified through the pure `update` function.
#[derive(Clone)]
pub struct Model {
    /// Number of timer ticks since startup.
    pub tick_count: u32,
    /// Number of button presses since startup.
    pub button_presses: u32,
    /// Current LED state (on/off).
    pub led_on: bool,
}

/// Renderable projection of the Model.
///
/// Props contain only the data needed for rendering/output.
/// In this embedded example, Props are identical to Model, but in more
/// complex applications they might include derived data or callbacks.
pub struct Props {
    pub tick_count: u32,
    pub button_presses: u32,
    pub led_on: bool,
}

/// Application logic
///
/// This struct holds injected service dependencies, enabling testability.
/// The framework does not require using dependency injection, but it is 
/// recommended.
pub struct AppLogic<C: ClockEffects, B: ButtonEffects> {
    clock_effects: C,
    button_effects: B,
}

impl<C: ClockEffects, B: ButtonEffects> AppLogic<C, B> {
    pub fn new(clock_effects: C, button_effects: B) -> Self {
        Self {
            clock_effects,
            button_effects,
        }
    }
}

impl<C: ClockEffects, B: ButtonEffects> MvuLogic<Event, Model, Props> for AppLogic<C, B> {
    fn init(&self, model: Model) -> (Model, Effect<Event>) {
        let timer_effect = self.clock_effects.observe_ticks();
        let button_effect = self.button_effects.observe_button();

        let effects = Effect::batch(vec![timer_effect, button_effect]);

        (model, effects)
    }

    fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>) {
        match event {
            Event::Tick => {
                let new_count = model.tick_count + 1;

                // Toggle the LED every 5 ticks
                let led_on = if new_count % 5 == 0 {
                    !model.led_on
                } else {
                    model.led_on
                };

                let new_model = Model {
                    tick_count: new_count,
                    led_on,
                    ..model.clone()
                };

                (new_model, Effect::none())
            }

            Event::ButtonPress => {
                // Button press always toggles the LED
                let new_model = Model {
                    button_presses: model.button_presses + 1,
                    led_on: !model.led_on,
                    ..model.clone()
                };

                (new_model, Effect::none())
            }
        }
    }

    fn view(&self, model: &Model, _emitter: &Emitter<Event>) -> Props {
        Props {
            tick_count: model.tick_count,
            button_presses: model.button_presses,
            led_on: model.led_on,
        }
    }
}
