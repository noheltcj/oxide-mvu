# Cortex-M Single-Core Renode Example

A complete Model-View-Update (MVU) application demonstrating `oxide-mvu` in a `no_std` embedded environment with concurrent event sources.

## Quick Start

Run the example using Docker (includes all dependencies):

```bash
cd examples/cortex-m-single-core
./test-docker.sh
```

This builds the Docker image, compiles the firmware, and launches the Renode emulator. Type `quit` + `[Enter]` in the Renode monitor to stop.

## What This Example Demonstrates

This example showcases idiomatic MVU patterns for embedded systems:

1. **Concurrent Event Sources**: Multiple async tasks (timer, button) emit events concurrently
2. **Deterministic State Management**: All state transitions flow through a pure `update` function
3. **Dependency Injection**: Testable service abstractions for hardware interactions
4. **Resource Efficiency**: Small heap footprint (16KB) suitable for constrained systems
5. **Embassy Integration**: Production-ready async executor for embedded Rust

**Why MVU for Embedded?**

- **No race conditions**: Events are serialized, state updates are sequential
- **Predictable behavior**: Pure functions make testing and debugging straightforward
- **Easy to extend**: Add new event sources without refactoring existing code

## Application Behavior

The application manages a simple LED controller with two event sources:
- **Timer**: Emits `Tick` every 3 seconds, increments counter, toggles LED every 5 ticks
- **Button**: Emits `ButtonPress` on button press (GPIO P0.11), increments counter, toggles LED immediately

Output via UART:

```
+--------------------------------------------+
|   oxide-mvu Embedded Demo (nRF52840)      |
+--------------------------------------------+

Concurrent Event Sources:
  - Timer: Ticks every 3 seconds
  - Button: GPIO button on P0.11

LED Behavior:
  - Toggles on button press and every 5 ticks
  - LED on P0.13 mirrors state

-----------------------------------------------

| Tick: 1 | Button: 0 | LED: [   ]
| Tick: 2 | Button: 0 | LED: [   ]
| Tick: 3 | Button: 1 | LED: [ X ]  <- Button pressed
| Tick: 4 | Button: 1 | LED: [ X ]
| Tick: 5 | Button: 1 | LED: [   ]  <- Auto-toggle (5th tick)
| Tick: 6 | Button: 1 | LED: [   ]
...
```

## Technical Details

- **Target**: `thumbv7em-none-eabihf` (Cortex-M4F with hardware floating point)
- **Chip**: nRF52840 microcontroller
- **Emulator**: Renode `nrf52840` platform
- **Async Runtime**: Embassy executor (cooperative multitasking)
- **HAL**: embassy-nrf with RTC-based time driver
- **Memory**: 1MB Flash, 256KB RAM, 16KB heap
- **Output**: UART (Renode analyzer)

## Architecture

### Concurrency Model

This example demonstrates **cooperative concurrency** on a single core:
- **Timer Effect**: Infinite loop emitting `Event::Tick` every 3 seconds
- **Button Effect**: Event stream from GPIO button
- **Embassy Executor**: Schedules both effects cooperatively (no preemption)
- **MVU Runtime**: Serializes events, ensuring deterministic state updates

Note: Event serialization works the same way in multi-core scenarios.

### Dependency Injection

This example demonstrates dependency injection as a pattern for testability, though it's not required by `oxide-mvu`.

The application uses service traits to abstract side effects:

```rust
trait ClockEffects {
    fn observe_ticks(&self) -> Effect<Event>;
}

trait ButtonEffects {
    fn observe_button(&self) -> Effect<Event>;
}
```

Benefits:
- **Testable**: Mock these traits in unit tests without hardware
- **Portable**: Swap implementations for different platforms, peripheral configurations, or boards
- **Clear**: Explicit boundaries between application logic and hardware

## Project Structure

```
.
├── Cargo.toml                         # Project configuration
├── Dockerfile                         # Docker image with Renode + toolchain
├── memory.x                           # Linker script (1MB Flash, 256KB RAM, 16KB stack)
├── renode.resc                        # Renode startup script for nRF52840
├── nrf52840-with-peripherals.repl     # Renode platform definition (LED + button)
├── test-docker.sh                     # Docker build + run wrapper
└── src/
    ├── main.rs                        # Platform setup (peripherals, runtime, tasks)
    ├── app.rs                         # MVU application logic
    ├── renderer.rs                    # UART + LED renderer
    ├── effects.rs                     # Effect traits + Embassy HAL implementations
    └── uart.rs                        # UART task + channel buffering
```

## Resources

- [oxide-mvu Documentation](https://docs.rs/oxide-mvu) - Full MVU pattern documentation
- [Embassy Project](https://embassy.dev/) - Async embedded framework for Rust
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/) - Embedded Rust fundamentals
