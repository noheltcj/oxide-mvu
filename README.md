# oxide-mvu

[![CI](https://github.com/noheltcj/oxide-mvu/workflows/CI/badge.svg)](https://github.com/noheltcj/oxide-mvu/actions)
[![Crates.io](https://img.shields.io/crates/v/oxide-mvu.svg)](https://crates.io/crates/oxide-mvu)
[![Documentation](https://docs.rs/oxide-mvu/badge.svg)](https://docs.rs/oxide-mvu)
[![License](https://img.shields.io/crates/l/oxide-mvu.svg)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/oxide-mvu.svg)](https://crates.io/crates/oxide-mvu)

A lightweight Model-View-Update (MVU) runtime for Rust with `no_std` support.

- **[Documentation](https://docs.rs/oxide-mvu)**
- **[Crates.io](https://crates.io/crates/oxide-mvu)**

---

> **Note:** This framework is under active development. APIs should not be considered stable.

## What is MVU?

The **Model-View-Update** pattern (also known as the Elm Architecture) structures applications as a pure functional loop:

- **Model**: Immutable state representing your entire application
- **Update**: Pure function transforming `(Event, Model) → (Model, Effect)`
- **View**: Pure function deriving renderable Props from the Model

This architecture eliminates implicit state mutation, making your application **predictable**, **debuggable**, and **testable**.

## Why oxide-mvu?

- **Pure functional state management** - All state transitions are explicit and testable
- **Unidirectional data flow** - Easy to reason about and debug
- **Lock-free concurrency** - Events from any thread without mutex overhead
- **Async runtime agnostic** - Works with tokio, async-std, smol, or custom executors
- **Built-in testing utilities** - Comprehensive test helpers included
- **`no_std` support** - Suitable for embedded systems (requires `alloc`)

## Quick Start

### Installation

```toml
[dependencies]
oxide-mvu = "0.4.1"
```

For embedded systems (`no_std`):
```toml
[dependencies]
oxide-mvu = { version = "0.4.1", features = ["no_std"] }
```

<details>
<summary>Example: Constructing a minimalistic runtime loop</summary>

```rust
use oxide_mvu::{Emitter, Effect, MvuLogic, MvuRuntime, Renderer};

// 1. Model the system, its behavior, and the renderable
//    projection (Props) including any controls.
#[derive(Clone)]
enum Event {
    Increment,
}

#[derive(Clone)]
struct Model {
    count: i32,
}

struct Props {
    count: i32,
    on_click: Box<dyn Fn()>,
}

// 2. Implement application logic
struct Logic;

impl MvuLogic<Event, Model, Props> for Logic {
    fn init(&self, model: Model) -> (Model, Effect<Event>) {
        (model, Effect::none())
    }

    fn update(&self, event: Event, model: &Model) -> (Model, Effect<Event>) {
        match event {
            Event::Increment => (Model { count: model.count + 1 }, Effect::none()),
        }
    }

    fn view(&self, model: &Model, emitter: &Emitter<Event>) -> Props {
        let emitter = emitter.clone();
        Props {
            count: model.count,
            on_click: Box::new(move || emitter.clone().try_emit(Event::Increment)),
        }
    }
}

// 3. Implement a renderer to display Props.
struct MyRenderer;

impl Renderer<Props> for MyRenderer {
    fn render(&mut self, props: Props) {
        println!("Count: {}", props.count);
    }
}

// 4. Run the application
async fn main() {
    let runtime = MvuRuntime::builder(
        Model { count: 0 },
        Logic {},
        MyRenderer {},
        |fut| { tokio::spawn(fut); },
    ).build();

    runtime.run().await;
}
```

</details>

## Examples

Working examples demonstrating `oxide-mvu` in various environments:

- **[cortex-m-single-core](examples/cortex-m-single-core)** - Complete `no_std` embedded example running on 
nRF52840 (Cortex-M4F) with Embassy async runtime, emulated via Renode.

See the [examples](examples) directory for more.

## Testing

The MVU pattern makes applications easy to test. State transitions are pure functions:

```rust
#[test]
fn test_increment() {
    let logic = Logic;
    let model = Model { count: 0 };
    let (new_model, _effect) = logic.update(Event::Increment, &model);
    assert_eq!(new_model.count, 1);
}
```

For integration testing, use the `testing` feature:

```toml
[dev-dependencies]
oxide-mvu = { version = "0.4.1", features = ["testing"] }
```

This provides:
- `TestMvuRuntime` - Runtime with manual event processing control
- `TestRenderer` - Renderer that captures Props for assertions
- `create_test_spawner()` - Test-friendly task spawner

<details>
<summary>Example: Integration testing the complete MVU loop</summary>

```rust
use oxide_mvu::{TestMvuRuntime, TestRenderer, create_test_spawner};

#[test]
fn test_full_mvu_loop() {
    let renderer = TestRenderer::new();
    let runtime = TestMvuRuntime::builder(
        Model { count: 0 },
        Logic {},
        renderer.clone(),
        create_test_spawner(),
    ).build();

    let mut driver = runtime.run();

    // Verify initial render
    assert_eq!(renderer.count(), 1);
    renderer.with_renders(|renders| {
        assert_eq!(renders[0].count, 0);
    });

    // Trigger the Props callback and advance the runtime
    renderer.with_renders(|renders| (renders[0].on_click)());
    driver.process_events();

    // Verify update
    assert_eq!(renderer.count(), 2);
    renderer.with_renders(|renders| {
        assert_eq!(renders[1].count, 1);
    });
}
```

</details>

See the [tests](tests) directory for more examples.

## Learn More

- **[API Documentation](https://docs.rs/oxide-mvu)** - Complete API reference with examples
- **[Examples](examples)** - Working examples in various environments
- **[Tests](tests)** - Tests demonstrating common patterns

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
