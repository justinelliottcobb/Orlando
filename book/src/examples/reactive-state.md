# Reactive State Management

Orlando's `Signal` and `Stream` types (Rust API) provide reactive primitives for state management with automatic change propagation.

## Signals: Derived State

Signals represent time-varying values. When a source signal changes, all derived signals update automatically.

### Temperature Converter

```rust
use orlando_transducers::signal::Signal;

let celsius = Signal::new(0.0_f64);
let fahrenheit = celsius.map(|c| c * 9.0 / 5.0 + 32.0);
let kelvin = celsius.map(|c| c + 273.15);

assert_eq!(*fahrenheit.get(), 32.0);
assert_eq!(*kelvin.get(), 273.15);

celsius.set(100.0);
assert_eq!(*fahrenheit.get(), 212.0);   // auto-updated
assert_eq!(*kelvin.get(), 373.15);      // auto-updated
```

### Shopping Cart

```rust
use orlando_transducers::signal::Signal;

let items = Signal::new(vec![
    ("Widget", 9.99),
    ("Gadget", 24.99),
]);

let subtotal = items.map(|items| {
    items.iter().map(|(_, price)| price).sum::<f64>()
});

let tax_rate = Signal::new(0.08);

let total = subtotal.combine(&tax_rate, |sub, rate| {
    sub * (1.0 + rate)
});

assert_eq!(*subtotal.get(), 34.98);
// total = 34.98 * 1.08 = 37.7784

// Add an item
items.update(|mut items| {
    items.push(("Doohickey", 14.99));
    items
});
// subtotal, total auto-update
```

### Combining Multiple Signals

```rust
let width = Signal::new(800_u32);
let height = Signal::new(600_u32);

let aspect_ratio = width.combine(&height, |w, h| {
    *w as f64 / *h as f64
});

let resolution = width.combine(&height, |w, h| {
    format!("{}x{}", w, h)
});

assert_eq!(*resolution.get(), "800x600");

width.set(1920);
height.set(1080);
assert_eq!(*resolution.get(), "1920x1080");
```

## Streams: Event Processing

Streams handle discrete events with transformation pipelines.

### Click Counter

```rust
use orlando_transducers::signal::Signal;
use orlando_transducers::stream::Stream;

let clicks = Stream::new();
let counter = Signal::new(0_i32);

// Bridge stream events into signal state
counter.fold(&clicks, 0, |count, _: &()| count + 1);

clicks.emit(());
clicks.emit(());
clicks.emit(());
assert_eq!(*counter.get(), 3);
```

### Event Filtering

```rust
use orlando_transducers::stream::Stream;

let events = Stream::new();

// Only process error events
let errors = events.filter(|e: &Event| e.level == Level::Error);
errors.subscribe(|e| {
    eprintln!("ERROR: {}", e.message);
});

// Only process first 100 events
let limited = events.take(100);
limited.subscribe(|e| {
    log_event(e);
});
```

### Stream Merging

```rust
use orlando_transducers::stream::Stream;

let keyboard = Stream::new();
let mouse = Stream::new();

// Merge into a unified input stream
let input = keyboard.merge(&mouse);
input.subscribe(|event| {
    handle_input(event);
});

keyboard.emit(InputEvent::KeyPress('a'));
mouse.emit(InputEvent::Click(100, 200));
// Both arrive at the merged subscriber
```

### Transform Pipeline on Stream

```rust
let raw_messages = Stream::new();

// Build a processing pipeline on the stream
let processed = raw_messages
    .map(|msg: String| msg.trim().to_lowercase())
    .filter(|msg: &String| !msg.is_empty());

processed.subscribe(|msg| {
    println!("Processed: {}", msg);
});

raw_messages.emit("  Hello World  ".into());
// prints: Processed: hello world
```

## Stream-Signal Bridge: `.fold()`

The `.fold()` method is the key bridge between discrete events (Stream) and continuous state (Signal).

### Running Average

```rust
use orlando_transducers::signal::Signal;
use orlando_transducers::stream::Stream;

let measurements = Stream::new();
let stats = Signal::new((0.0_f64, 0_u32)); // (sum, count)

stats.fold(&measurements, (0.0, 0), |state, value: &f64| {
    (state.0 + value, state.1 + 1)
});

let average = stats.map(|(sum, count)| {
    if count > 0 { sum / count as f64 } else { 0.0 }
});

measurements.emit(10.0);
measurements.emit(20.0);
measurements.emit(30.0);
assert_eq!(*average.get(), 20.0);
```

### State Machine

```rust
use orlando_transducers::signal::Signal;
use orlando_transducers::stream::Stream;

#[derive(Clone, Debug, PartialEq)]
enum AppState {
    Loading,
    Ready,
    Error(String),
}

let actions = Stream::new();
let state = Signal::new(AppState::Loading);

state.fold(&actions, AppState::Loading, |current, action: &Action| {
    match (current, action) {
        (AppState::Loading, Action::DataLoaded) => AppState::Ready,
        (_, Action::Error(msg)) => AppState::Error(msg.clone()),
        (_, Action::Reset) => AppState::Loading,
        (state, _) => state,
    }
});

actions.emit(Action::DataLoaded);
assert_eq!(*state.get(), AppState::Ready);
```

## Subscription Lifecycle

Subscriptions are automatically cleaned up when dropped:

```rust
let counter = Signal::new(0);

{
    let _sub = counter.subscribe(|val| {
        println!("Value: {}", val);
    });

    counter.set(1);  // prints: Value: 1
    counter.set(2);  // prints: Value: 2
}
// _sub dropped here, subscription is cleaned up

counter.set(3);  // no output - subscriber is gone
```

For streams:

```rust
let events = Stream::new();

let sub = events.subscribe(|e| handle(e));

// Explicitly unsubscribe when done
drop(sub);
```
