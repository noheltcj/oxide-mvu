use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peri;
use embassy_nrf::peripherals::{P0_13};
use oxide_mvu::Renderer;

use crate::app::Props;
use crate::uart_println;

/// Renderer that outputs state via UART and controls physical LED.
pub struct EmbeddedRenderer {
    led: Output<'static>,
}

impl EmbeddedRenderer {
    /// Create renderer with GPIO LED control on P0.13.
    pub fn new(led_pin: Peri<'static, P0_13>) -> Self {
        let led = Output::new(led_pin, Level::Low, OutputDrive::Standard);
        Self { led }
    }
}

impl Renderer<Props> for EmbeddedRenderer {
    fn render(&mut self, props: Props) {
        // Control physical LED based on props
        let led = &mut self.led;
        if props.led_on {
            led.set_high();
        } else {
            led.set_low();
        }

        // Visual LED representation in terminal
        let led_visual = if props.led_on {
            "[ X ]"
        } else {
            "[   ]"
        };

        uart_println!(
            "| Tick: {} | Button: {} | LED: {}",
            props.tick_count,
            props.button_presses,
            led_visual
        );
    }
}
