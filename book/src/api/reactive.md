# Reactive Primitives API

Orlando provides `Signal` and `Stream` types for reactive programming. These are currently **Rust-only** APIs.

## Signal\<T\>

A time-varying value with automatic change propagation. When a source signal changes, all derived signals update automatically.

### `Signal::new(value)`

Create a signal with an initial value.

```rust
use orlando_transducers::signal::Signal;

let counter = Signal::new(0_i32);
```

### `.get()`

Get the current value. Returns `Ref<T>` (smart pointer).

```rust
let val = counter.get();
assert_eq!(*val, 0);
```

### `.set(value)`

Set a new value, notifying all subscribers.

```rust
counter.set(42);
assert_eq!(*counter.get(), 42);
```

### `.update(f)`

Update the value by applying a function.

```rust
counter.update(|n| n + 1);
assert_eq!(*counter.get(), 43);
```

### `.subscribe(f)`

Subscribe to value changes. Returns a `Subscription` that unsubscribes when dropped.

```rust
let _sub = counter.subscribe(|val| {
    println!("Counter is now: {}", val);
});

counter.set(10);  // prints: Counter is now: 10
```

### `.map(f)`

Create a derived signal that auto-updates when the source changes.

```rust
let celsius = Signal::new(100.0_f64);
let fahrenheit = celsius.map(|c| c * 9.0 / 5.0 + 32.0);

assert_eq!(*fahrenheit.get(), 212.0);

celsius.set(0.0);
assert_eq!(*fahrenheit.get(), 32.0);  // auto-updated
```

### `.combine(other, f)`

Combine two signals into a derived signal.

```rust
let width = Signal::new(800_u32);
let height = Signal::new(600_u32);

let area = width.combine(&height, |w, h| w * h);
assert_eq!(*area.get(), 480_000);

width.set(1920);
assert_eq!(*area.get(), 1_152_000);  // auto-updated
```

### `.fold(stream, init, f)`

Fold a stream's events into this signal's value.

```rust
use orlando_transducers::stream::Stream;

let counter = Signal::new(0_i32);
let clicks = Stream::new();

counter.fold(&clicks, 0, |count, _: &()| count + 1);

clicks.emit(());
clicks.emit(());
assert_eq!(*counter.get(), 2);
```

## Stream\<T\>

A push-based event stream for discrete events.

### `Stream::new()`

Create an empty stream.

```rust
use orlando_transducers::stream::Stream;

let events = Stream::<String>::new();
```

### `.emit(value)`

Push a value to all subscribers.

```rust
events.emit("hello".into());
```

### `.subscribe(f)`

Listen for events. Returns `StreamSubscription` that unsubscribes when dropped.

```rust
let _sub = events.subscribe(|msg| {
    println!("Received: {}", msg);
});

events.emit("test".into());  // prints: Received: test
```

### `.map(f)`

Transform each event.

```rust
let raw = Stream::new();
let upper = raw.map(|s: String| s.to_uppercase());

upper.subscribe(|s| println!("{}", s));
raw.emit("hello".into());  // prints: HELLO
```

### `.filter(pred)`

Only pass events matching the predicate.

```rust
let numbers = Stream::new();
let evens = numbers.filter(|n: &i32| n % 2 == 0);

evens.subscribe(|n| println!("Even: {}", n));
numbers.emit(1);  // nothing
numbers.emit(2);  // prints: Even: 2
numbers.emit(3);  // nothing
numbers.emit(4);  // prints: Even: 4
```

### `.take(n)`

Take only the first `n` events, then stop.

```rust
let events = Stream::new();
let first3 = events.take(3);

first3.subscribe(|v| println!("{}", v));
events.emit(1);  // prints: 1
events.emit(2);  // prints: 2
events.emit(3);  // prints: 3
events.emit(4);  // nothing (taken 3 already)
```

### `.merge(other)`

Merge two streams into one.

```rust
let keyboard = Stream::new();
let mouse = Stream::new();

let input = keyboard.merge(&mouse);
input.subscribe(|event| handle(event));

keyboard.emit(KeyEvent::Press('a'));
mouse.emit(MouseEvent::Click(100, 200));
// Both arrive at the merged subscriber
```

### `.fold(init, f)`

Fold events into a Signal, bridging discrete events to continuous state.

```rust
let measurements = Stream::new();
let sum = measurements.fold(0.0_f64, |acc, val: &f64| acc + val);

measurements.emit(10.0);
measurements.emit(20.0);
assert_eq!(*sum.get(), 30.0);
```

## Subscription Lifecycle

Subscriptions are cleaned up automatically when dropped:

```rust
let sig = Signal::new(0);

{
    let _sub = sig.subscribe(|v| println!("{}", v));
    sig.set(1);  // prints: 1
}
// _sub dropped — subscription removed

sig.set(2);  // no output
```

Explicit cleanup:

```rust
let sub = stream.subscribe(|e| handle(e));
drop(sub);  // explicitly unsubscribe
```
