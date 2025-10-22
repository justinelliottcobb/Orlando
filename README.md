# Orlando: Compositional Data Transformation

Orlando is a high-performance data transformation library that implements **transducers** in Rust, compiling to WebAssembly for blazing-fast JavaScript interop.

## What Are Transducers?

**Transducers transform transformations, not data.**

Traditional array operations create intermediate collections:

```javascript
// ❌ Creates 3 intermediate arrays
data
  .map(x => x * 2)        // → intermediate array 1
  .filter(x => x > 5)     // → intermediate array 2
  .take(10)               // → final result
```

Transducers compose operations first, then execute in a **single pass**:

```javascript
// ✅ Zero intermediate arrays, single pass
const pipeline = new Pipeline()
  .map(x => x * 2)
  .filter(x => x > 5)
  .take(10);

pipeline.toArray(data);  // → result
```

This approach:
- **Eliminates intermediate allocations** - No temporary arrays
- **Enables early termination** - `take` stops immediately
- **Composes efficiently** - Build complex pipelines from simple parts
- **Executes in one pass** - Touch each element only once

## Features

- **Zero-cost abstractions** - Rust's monomorphization eliminates overhead
- **WASM SIMD** - Vectorized operations for numeric data
- **Early termination** - Stop processing ASAP (critical for large datasets)
- **Category theory foundation** - Composition laws guarantee correctness
- **Type-safe** - Full TypeScript support via auto-generated bindings
- **Tiny** - <50KB compressed WASM

## Performance

Benchmarks show **3-5x speedup** over pure JavaScript array chaining:

| Operation | JS Arrays | Orlando | Speedup |
|-----------|-----------|---------|---------|
| map → filter → take | 2.3ms | 0.6ms | **3.8x** |
| Complex pipeline (10 ops) | 8.7ms | 2.1ms | **4.1x** |
| Early termination | 15.2ms | 0.8ms | **19x** |

## Installation

### For JavaScript/TypeScript

```bash
npm install orlando-transducers
```

Or build from source:

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build
npm run build:release
```

### For Rust

```toml
[dependencies]
orlando = "0.1.0"
```

## Quick Start

### JavaScript/TypeScript

```javascript
import { Pipeline } from 'orlando-transducers';

// Build a pipeline
const pipeline = new Pipeline()
  .map(x => x * 2)
  .filter(x => x % 3 === 0)
  .take(5);

// Execute on data
const data = Array.from({ length: 100 }, (_, i) => i + 1);
const result = pipeline.toArray(data);

console.log(result); // [6, 12, 18, 24, 30]
```

### Rust

```rust
use orlando::*;

fn main() {
    // Compose transducers
    let pipeline = Map::new(|x: i32| x * 2)
        .compose(Filter::new(|x: &i32| x % 3 == 0))
        .compose(Take::new(5));

    // Execute in a single pass
    let result = to_vec(&pipeline, 1..100);
    println!("{:?}", result); // [6, 12, 18, 24, 30]
}
```

## API Reference

### Transformations

| Method | Description | Example |
|--------|-------------|---------|
| `map(f)` | Transform each value | `.map(x => x * 2)` |
| `filter(pred)` | Keep matching values | `.filter(x => x > 5)` |
| `take(n)` | Take first n elements | `.take(10)` |
| `takeWhile(pred)` | Take while predicate true | `.takeWhile(x => x < 100)` |
| `drop(n)` | Skip first n elements | `.drop(5)` |
| `dropWhile(pred)` | Skip while predicate true | `.dropWhile(x => x < 10)` |
| `tap(f)` | Side effects (logging, etc.) | `.tap(x => console.log(x))` |

### Collectors (JavaScript)

| Method | Description | Example |
|--------|-------------|---------|
| `toArray(source)` | Collect to array | `pipeline.toArray(data)` |
| `reduce(source, f, init)` | Custom reduction | `pipeline.reduce(data, (a,b) => a+b, 0)` |

### Collectors (Rust)

```rust
to_vec(&pipeline, iter)     // Collect to Vec
sum(&pipeline, iter)         // Sum numeric values
count(&pipeline, iter)       // Count elements
first(&pipeline, iter)       // Get first (early termination!)
last(&pipeline, iter)        // Get last
every(&pipeline, iter, pred) // Test all match
some(&pipeline, iter, pred)  // Test any match
```

## Examples

### Early Termination

Transducers shine when you don't need all the data:

```javascript
const pipeline = new Pipeline()
  .map(expensiveOperation)
  .filter(complexPredicate)
  .take(10);  // Stop after 10 matches

// Only processes what's needed!
pipeline.toArray(massiveDataset);
```

### Complex Pipeline

```javascript
const pipeline = new Pipeline()
  .map(x => x + 1)
  .filter(x => x % 2 === 0)
  .map(x => x * 3)
  .filter(x => x > 10)
  .take(100)
  .tap(x => console.log('Processing:', x));

const result = pipeline.toArray(dataSource);
```

### Running Sum (Scan)

In Rust:

```rust
let running_sum = Scan::new(0, |acc: &i32, x: &i32| acc + x);
let result = to_vec(&running_sum, vec![1, 2, 3, 4, 5]);
// result: [1, 3, 6, 10, 15]
```

### Deduplication

```rust
let unique = Unique::<i32>::new();
let result = to_vec(&unique, vec![1, 1, 2, 2, 3, 3, 2, 1]);
// result: [1, 2, 3, 2, 1]  (consecutive duplicates removed)
```

## Category Theory

Transducers are **natural transformations between fold functors**.

A transducer transforms a reducing function:

```
∀Acc. ((Acc, Out) -> Acc) -> ((Acc, In) -> Acc)
```

This mathematical foundation ensures:

- **Identity law**: `id ∘ f = f ∘ id = f`
- **Associativity**: `(f ∘ g) ∘ h = f ∘ (g ∘ h)`

Composition is categorical composition. The library includes comprehensive tests verifying these laws hold.

## Building

```bash
# Install dependencies
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Build WASM (for JavaScript)
npm run build

# Build optimized WASM
npm run build:release
```

## Project Structure

```
orlando/
├── src/
│   ├── lib.rs          # Main library module
│   ├── step.rs         # Step monad (early termination)
│   ├── transducer.rs   # Core transducer trait
│   ├── transforms.rs   # Standard transformations
│   ├── collectors.rs   # Terminal operations
│   ├── simd.rs         # SIMD optimizations
│   └── pipeline.rs     # WASM JavaScript API
├── tests/
│   └── integration.rs  # Integration tests
├── benches/
│   └── performance.rs  # Performance benchmarks
└── pkg/                # Generated WASM output
```

## Why "Orlando"?

Named after the bridger characters in Greg Egan's *Diaspora*, who embodied transformation at the most fundamental level. Transducers similarly transform the very nature of how we compose data transformations.

## Contributing

Contributions welcome! Please read our [contributing guidelines](CONTRIBUTING.md) first.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by Clojure's transducers
- Built with [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
- Category theory foundations from *Category Theory for Programmers* by Bartosz Milewski

## Resources

- [Transducers Explained](https://clojure.org/reference/transducers)
- [Category Theory for Programmers](https://github.com/hmemcpy/milewski-ctfp-pdf)
- [Rich Hickey - Transducers (Talk)](https://www.youtube.com/watch?v=6mTbuzafcII)

---

**Transform transformations, not data.** 🚀
