# oxide-mvu

A lightweight Model-View-Update (MVU) runtime framework for Rust with `no_std` support.

## Overview

__(Note that this framework is not yet ready for production, and APIs should not be considered stable)__

`oxide-mvu` implements the MVU pattern for building predictable, testable applications in Rust. The framework is powerful enough
to be a good choice for most applications, but will always evolve with support for no_std, low-memory, single-cpu, embedded systems as a baseline.

Oxide is intended to clarify the management of state in an application using clear separation of concerns between cleanly isolated view and application logic layers.

## Features

- **Unidirectional data flow**: State transitions occur in a single loop:
  - → event emission is triggered from an effect or user input
  - → update function receives event data and yields new state
  - → view function reduces state to renderable props
  - → renderer function receives props for display
- **Type-safe event dispatch**: Callbacks in Props maintain type safety
- **Effect system**: Controlled, declarative side effects
- **no_std support**: Works in embedded environments (requires `alloc` for heap allocation)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
oxide-mvu = "0.2.0"
```

For `no_std` environments:
```toml
[dependencies]
oxide-mvu = { version = "0.2.0", features = ["no_std"] }
```

## Usage

### Define your types

```rust
use oxide_mvu::{Emitter, Effect, MvuLogic};

#[derive(Clone)]
enum Event {
    Increment,
    Decrement,
}

#[derive(Clone)]
struct Model {
    count: i32,
}

struct Props {
    count: i32,
    on_increment: Box<dyn Fn()>,
    on_decrement: Box<dyn Fn()>,
}
```

### Implement the MvuLogic trait

```rust
struct Logic;

impl MvuLogic<Event, Model, Props> for Logic {
    fn init(&self, model: Model) -> (Model, Effect<Event>) {
        (model, Effect::none())
    }

    fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>) {
        let new_model = match event {
            Event::Increment => Model { count: model.count + 1 },
            Event::Decrement => Model { count: model.count - 1 },
        };
        (new_model, Effect::none())
    }

    fn view(&self, model: &Model, emitter: &Emitter<Event>) -> Props {
        let emitter_inc = emitter.clone();
        let emitter_dec = emitter.clone();
        Props {
            count: model.count,
            on_increment: Box::new(move || emitter_inc.emit(Event::Increment)),
            on_decrement: Box::new(move || emitter_dec.emit(Event::Decrement)),
        }
    }
}
```

### Implement a Renderer

```rust
use oxide_mvu::Renderer;

struct MyRenderer;

impl Renderer<Props> for MyRenderer {
    fn render(&mut self, props: Props) {
        println!("Count: {}", props.count);
        // In a real app, you'd render UI here and wire up callbacks:
        // button.on_click = props.on_increment;
    }
}
```

### Create a Spawner

The runtime needs a spawner to execute async effects. The spawner is framework-agnostic,
allowing you to use any async runtime (tokio, async-std, smol, etc.). Any function or
closure that implements the `Spawner` trait will work:

#### Using tokio:
```rust
let spawner = |fut| {
    tokio::spawn(fut);
};
```

#### Using async-std:
```rust
let spawner = |fut| {
    async_std::task::spawn(fut);
};
```

### Run the application

```rust
use oxide_mvu::MvuRuntime;

fn main() {
    let model = Model { count: 0 };
    let logic = Logic;
    let renderer = MyRenderer;

    // Create a spawner for your chosen async runtime
    let spawner = |fut| {
        tokio::spawn(fut);
    };

    let runtime = MvuRuntime::new(model, logic, renderer, spawner);
    runtime.run();
}
```

## Testing

`oxide-mvu` provides specialized testing utilities for integration testing your MVU applications.

### Enabling Testing Utilities

To access the testing helpers in your project, enable the `testing` feature:

```toml
[dev-dependencies]
oxide-mvu = { version = "0.2.0", features = ["testing"] }
```

This gives you access to:
- `TestMvuRuntime` - Runtime with manual event processing control
- `TestMvuDriver` - Driver for manually processing events in tests
- `TestRenderer` - Renderer that captures Props for assertions

### Unit Testing

State transitions are pure functions, making them easy to unit test:

```rust
#[test]
fn test_increment() {
    let logic = Logic;
    let model = Model { count: 0 };
    let (new_model, _effect) = logic.update(Event::Increment, &model);
    assert_eq!(new_model.count, 1);
}
```

### Integration Testing

Use `TestMvuRuntime` and `TestRenderer` to test the full MVU loop:

```rust
use oxide_mvu::{TestMvuRuntime, TestRenderer, create_test_spawner};

#[test]
fn test_full_mvu_loop() {
    // Create a TestRenderer to capture renders
    let renderer = TestRenderer::new();

    // Create runtime with test helpers
    let runtime = TestMvuRuntime::new(
        Model { count: 0 },
        Logic,
        renderer.clone(),
        create_test_spawner(),
    );

    // Run and get driver for manual event control
    let mut driver = runtime.run();

    // Verify initial render
    assert_eq!(renderer.count(), 1);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
    });

    // Trigger event via Props callback
    renderer.with_renders(|renders| {
        (renders[0].on_increment)();
    });

    // Manually process events
    driver.process_events();

    // Verify a second render occurred
    assert_eq!(renderer.count(), 2);
    renderer.with_renders(|renders| {
        assert_eq!(renders[1].count, 1);
    });
}
```

### Key Testing Concepts

- **`TestMvuRuntime`**: Unlike `MvuRuntime`, events are not automatically processed. You must call `driver.process_events()` to manually drive the event loop.
- **`TestRenderer`**: Captures all rendered Props, allowing you to verify the runtime's output and trigger callbacks.
- **`renderer.with_renders()`**: Access all captured Props for assertions or to trigger callbacks.
- **`driver.process_events()`**: Process all queued events until the queue is empty.
- **`create_test_spawner()`**: Creates a spawner that executes effects synchronously for deterministic testing.

See the `tests` directory for complete examples.

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
