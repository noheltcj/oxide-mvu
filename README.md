# oxide-mvu

A lightweight Model-View-Update (MVU) runtime framework for Rust with `no_std` support.

## Overview

`oxide-mvu` implements the MVU pattern for building predictable, testable applications in Rust. The framework is powerful enough
to be a good choice for most applications, but will always evolve with support for no_std, low-memory, single-cpu, embedded systems as a baseline.

Oxide is intended to clarify the management of state in an application using clear separation of concerns between the view and application logic layers.

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
oxide-mvu = "0.1.0"
```

For `std` environments:
```toml
[dependencies]
oxide-mvu = { version = "0.1.0", features = ["std"] }
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

### Run the application

```rust
use oxide_mvu::MvuRuntime;

fn main() {
    let model = Model { count: 0 };
    let logic = Box::new(Logic);
    let renderer = Box::new(MyRenderer);
    let runtime = MvuRuntime::new(model, logic, renderer);
    runtime.run();
}
```

## Testing

State changes are driven via Props callbacks, making the flow easy to test:

```rust
#[test]
fn test_counter() {
    let logic = Logic;
    let model = Model { count: 0 };
    let (model, _) = logic.init(model);
    let (model, _) = logic.update(Event::Increment, &model);
    assert_eq!(model.count, 1);
}
```

See `tests/lib_test.rs` for a complete example of testing the full MVU loop.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
