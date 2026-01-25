#![no_std]
#![no_main]

//! # Cortex-M Single-Core MVU Example
//!
//! Platform setup for running `oxide-mvu` on nRF52840 (Cortex-M4F) with Embassy.
//!
//! This file handles:
//! - Heap initialization (16KB)
//! - Embassy executor and peripheral setup
//! - Dependency injection (GPIO button, timer effects)
//! - MVU runtime configuration and startup
//!
//! For architecture details, rationale, and usage, see the [README](../README.md).

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use core::future::Future;
use core::mem::MaybeUninit;
use core::pin::Pin;
use embassy_executor::Spawner;
use embassy_nrf::config::Config;
use embassy_nrf::uarte;
use embedded_alloc::LlffHeap as Heap;
use oxide_mvu::MvuRuntime;

#[allow(unused_imports)]
use panic_halt as _;

mod app;
mod effects;
mod renderer;
mod uart;

use app::{AppLogic, Model};
use effects::{EmbassyClockEffects, GpioButtonEffects};
use renderer::EmbeddedRenderer;
use uart::{uart_task, UartIrqs};

// ============================================================================
// Global Allocator
// ============================================================================

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 16384; // 16KB heap

fn init_heap() {
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

/// Task to run a single effect future.
///
/// Effects spawned by the MVU runtime are executed as embassy tasks.
#[embassy_executor::task(pool_size = 4)]
async fn effect_runner(fut: Pin<Box<dyn Future<Output = ()> + Send>>) {
    fut.await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    init_heap();

    // Initialize nRF peripherals with default clock setup.
    // This sets up the time driver for embassy-time.
    let mut config = Config::default();
    config.lfclk_source = embassy_nrf::config::LfclkSource::Synthesized;
    let peripherals = embassy_nrf::init(config);

    let tx = uarte::UarteTx::new(peripherals.UARTE0, UartIrqs, peripherals.P0_06, uarte::Config::default());
    if let Some(error) = spawner.spawn(uart_task(tx)).err() {
        panic!("ERROR: Unable to spawn UART task. \n\n{}", error.to_string())
    };

    uart_println!("Starting MVU runtime...");

    // Configure GPIO pins for button and LED
    // Button 1: P0.11 (from REPL file)
    // LED 1: P0.13 (from REPL file)
    let button_pin = peripherals.P0_11;
    let led_pin = peripherals.P0_13;

    // Inject real service implementations with GPIO
    let clock_effects = EmbassyClockEffects;
    let button_effects = GpioButtonEffects::new(button_pin);

    // Create application logic with injected dependencies
    let logic = AppLogic::new(clock_effects, button_effects);

    // Create a renderer
    let renderer = EmbeddedRenderer::new(led_pin);

    // Create the MVU runtime
    let runtime = MvuRuntime::builder(
        Model {
            tick_count: 0,
            button_presses: 0,
            led_on: false,
        },
        logic,
        renderer,
        // Embassy spawner as the task spawner for effects
        move |fut| {
            spawner.spawn(effect_runner(fut)).ok();
        },
    )
    .capacity(16) // Small buffer for embedded system
    .build();

    uart_println!();
    uart_println!("+--------------------------------------------+");
    uart_println!("|   oxide-mvu Embedded Demo (nRF52840)      |");
    uart_println!("+--------------------------------------------+");
    uart_println!();
    uart_println!("Concurrent Event Sources:");
    uart_println!("  - Timer: Ticks every 3 seconds");
    uart_println!("  - Button: Physical button on P0.11");
    uart_println!();
    uart_println!("LED Behavior:");
    uart_println!("  - Toggled when button pressed and after every 5 ticks");
    uart_println!("  - Physical LED on P0.13 mirrors state");
    uart_println!();
    uart_println!("-----------------------------------------------");
    uart_println!();

    // Run the MVU runtime (this never returns)
    runtime.run().await;
}
