# Rust API

Orlando is a first-class Rust crate with ergonomic iterator extensions, reactive primitives, and a fluent builder API.

## Core Transducers

The fundamental building blocks for data transformation pipelines.

### The Transducer Trait

```rust
use orlando_transducers::{Transducer, Map, Filter, Take, Compose};

// Transducers compose transformations, not data
let pipeline = Map::new(|x: i32| x * 2)
    .compose(Filter::new(|x: &i32| *x > 10))
    .compose(Take::new(5));

// Execute with a collector
let result = orlando_transducers::to_vec(&pipeline, 1..100);
// result: [12, 14, 16, 18, 20]
```

### Available Transducers

| Type | Description | Constructor |
|------|-------------|-------------|
| `Map<F>` | Transform each element | `Map::new(\|x\| x * 2)` |
| `Filter<P>` | Keep elements matching predicate | `Filter::new(\|x: &i32\| *x > 5)` |
| `Take` | Take first N elements (early termination) | `Take::new(10)` |
| `TakeWhile<P>` | Take while predicate holds | `TakeWhile::new(\|x: &i32\| *x < 100)` |
| `Drop` | Skip first N elements | `Drop::new(5)` |
| `DropWhile<P>` | Skip while predicate holds | `DropWhile::new(\|x: &i32\| *x < 10)` |
| `FlatMap<F>` | Transform and flatten | `FlatMap::new(\|x\| vec![x, x*2])` |
| `Reject<P>` | Remove matching elements | `Reject::new(\|x: &i32\| *x < 0)` |
| `Chunk` | Group into fixed-size chunks | `Chunk::new(3)` |
| `Unique` | Remove consecutive duplicates | `Unique::new()` |
| `Scan<F, S>` | Accumulate with intermediate results | `Scan::new(0, \|acc, x\| acc + x)` |

### Collectors

Terminal operations that execute a pipeline:

```rust
use orlando_transducers::*;

let pipeline = Map::new(|x: i32| x * 2);

let vec_result = to_vec(&pipeline, 1..=5);     // [2, 4, 6, 8, 10]
let total = sum(&pipeline, 1..=5);              // 30
let n = count(&pipeline, 1..=5);                // 5
let head = first(&pipeline, 1..=5);             // Some(2)
let tail = last(&pipeline, 1..=5);              // Some(10)
let all_pos = every(&pipeline, 1..=5, |x| *x > 0);  // true
let has_ten = some(&pipeline, 1..=5, |x| *x == 10);  // true
```

### Logic Combinators

```rust
use orlando_transducers::logic::{When, Unless, IfElse};

// When: transform only when predicate is true
let double_positive = When::new(|x: &i32| *x > 0, |x: i32| x * 2);

// Unless: transform only when predicate is false
let zero_negative = Unless::new(|x: &i32| *x > 0, |_: i32| 0);

// IfElse: branch on condition
let classify = IfElse::new(
    |x: &i32| *x >= 0,
    |x: i32| x * 2,     // positive: double
    |x: i32| x.abs(),   // negative: absolute value
);
```

## TransduceExt Trait

Extension trait that adds `.transduce()` to any iterator:

```rust
use orlando_transducers::iter_ext::TransduceExt;
use orlando_transducers::{Map, Filter, Take};

let result: Vec<i32> = (1..100)
    .transduce(
        Map::new(|x: i32| x * 2)
            .compose(Filter::new(|x: &i32| *x > 10))
            .compose(Take::new(5))
    );

assert_eq!(result, vec![12, 14, 16, 18, 20]);
```

The `TransducedIterator` returned by `.transduce()` is a lazy iterator adapter - it processes elements on demand and supports early termination.

## PipelineBuilder

Fluent builder API for constructing transducer pipelines without manual composition:

```rust
use orlando_transducers::iter_ext::PipelineBuilder;

let result = PipelineBuilder::new()
    .map(|x: i32| x * 2)
    .filter(|x: &i32| *x > 10)
    .take(5)
    .run(1..100);

assert_eq!(result, vec![12, 14, 16, 18, 20]);
```

### Available Builder Methods

| Method | Description |
|--------|-------------|
| `.map(f)` | Transform each element |
| `.filter(pred)` | Keep matching elements |
| `.take(n)` | Take first N elements |
| `.run(iter)` | Execute pipeline on an iterator, collecting to `Vec` |

## Signal\<T\>

A time-varying value with automatic change propagation. Signals form the foundation of reactive programming in Orlando.

```rust
use orlando_transducers::signal::Signal;

// Create a signal with an initial value
let celsius = Signal::new(0.0_f64);

// Derived signal that auto-updates when source changes
let fahrenheit = celsius.map(|c| c * 9.0 / 5.0 + 32.0);

assert_eq!(*fahrenheit.get(), 32.0);

celsius.set(100.0);
assert_eq!(*fahrenheit.get(), 212.0);  // automatically updated
```

### Signal Methods

| Method | Description |
|--------|-------------|
| `Signal::new(value)` | Create a signal with initial value |
| `.get()` | Get current value (returns `Ref<T>`) |
| `.set(value)` | Set new value, notifying all subscribers |
| `.update(f)` | Update value via function |
| `.subscribe(f)` | Subscribe to changes, returns `Subscription` |
| `.map(f)` | Create a derived signal |
| `.combine(other, f)` | Combine two signals into one |
| `.fold(stream, init, f)` | Fold a stream into this signal |

### Subscriptions

```rust
let counter = Signal::new(0);
let mut log = Vec::new();

let _sub = counter.subscribe(|val| {
    println!("Counter changed to: {}", val);
});

counter.set(1);  // prints: Counter changed to: 1
counter.set(2);  // prints: Counter changed to: 2
// Subscription is dropped when _sub goes out of scope
```

### Combining Signals

```rust
let width = Signal::new(10.0_f64);
let height = Signal::new(5.0_f64);

let area = width.combine(&height, |w, h| w * h);
assert_eq!(*area.get(), 50.0);

width.set(20.0);
assert_eq!(*area.get(), 100.0);  // auto-updated
```

## Stream\<T\>

A push-based event stream for discrete event processing.

```rust
use orlando_transducers::stream::Stream;

let clicks = Stream::new();

// Transform events
let doubled = clicks.map(|x: i32| x * 2);

// Subscribe to processed events
doubled.subscribe(|val| println!("Got: {}", val));

clicks.emit(21);  // prints: Got: 42
```

### Stream Methods

| Method | Description |
|--------|-------------|
| `Stream::new()` | Create an empty stream |
| `.emit(value)` | Push a value to all subscribers |
| `.subscribe(f)` | Listen for events, returns `StreamSubscription` |
| `.map(f)` | Transform each event |
| `.filter(pred)` | Only pass matching events |
| `.take(n)` | Take first N events then stop |
| `.merge(other)` | Merge two streams |
| `.fold(init, f)` | Fold into a Signal |

### Stream-Signal Bridge

The `.fold()` method bridges discrete events into continuous signal values:

```rust
use orlando_transducers::signal::Signal;
use orlando_transducers::stream::Stream;

let counter = Signal::new(0);
let increments = Stream::new();

// Each stream event updates the signal
counter.fold(&increments, 0, |acc, _| acc + 1);

increments.emit(());  // counter is now 1
increments.emit(());  // counter is now 2
```

## Multi-Input Operations

Standalone functions for combining multiple collections:

```rust
use orlando_transducers::{merge, intersection, difference, union, symmetric_difference};

let a = vec![1, 2, 3, 4];
let b = vec![3, 4, 5, 6];

let merged = merge(vec![a.clone(), b.clone()]);      // [1, 3, 2, 4, 3, 5, 4, 6]
let common = intersection(a.clone(), b.clone());       // [3, 4]
let unique_a = difference(a.clone(), b.clone());       // [1, 2]
let all = union(a.clone(), b.clone());                 // [1, 2, 3, 4, 5, 6]
let exclusive = symmetric_difference(a, b);            // [1, 2, 5, 6]
```

## Statistical Functions

```rust
use orlando_transducers::collectors::*;

let data = vec![2.0, 4.0, 6.0, 8.0];

let avg = mean(&data);          // 5.0
let mid = median(&data);        // 5.0
let var = variance(&data);      // 6.666...
let dev = std_dev(&data);       // 2.581...
let p95 = quantile(&data, 0.95);
```
