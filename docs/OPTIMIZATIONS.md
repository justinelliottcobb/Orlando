# Performance Optimizations

This document tracks performance optimizations applied to Orlando transducers.

## Pattern Matching Optimization (2025-01-22)

### Problem

The original code used helper functions (`is_stopped` + `unwrap`) to check and extract values from the `Step` enum:

```rust
// Before - function call overhead
for item in source {
    let step = transformed(acc, item);
    let is_stop = is_stopped(&step);  // Function call
    acc = step.unwrap();               // Another function call
    if is_stop {
        break;
    }
}
```

### Solution

Use direct pattern matching to eliminate function call overhead:

```rust
// After - zero-cost abstraction
for item in source {
    match transformed(acc, item) {
        Step::Continue(new_acc) => acc = new_acc,
        Step::Stop(final_acc) => {
            acc = final_acc;
            break;
        }
    }
}
```

### Impact

**Files Optimized:**
1. ✅ `src/collectors.rs` - `to_vec()` function (line 35-43)
2. ✅ `src/collectors.rs` - `reduce()` function (line 77-85)
3. ✅ `src/transforms.rs` - `Take` transducer (line 155-157)

**Performance Gains:**
- Eliminates 2 function calls per iteration in collectors
- For 100K elements: saves ~200K function calls
- Especially beneficial for early termination scenarios
- Compiler can better optimize direct pattern matching

**Benchmark Impact:**
- Hot path in all transducer operations
- Measurable improvement in high-iteration scenarios
- More idiomatic Rust code

### Files Already Optimal

These implementations already use optimal patterns:
- ✅ `Map` transducer - Direct reducer call
- ✅ `Filter` transducer - Simple conditional with cont/reducer
- ✅ `TakeWhile` transducer - Predicate-based stop
- ✅ `Drop` transducer - Counter-based logic
- ✅ `DropWhile` transducer - State flag with cont
- ✅ `Unique` transducer - Direct state comparison
- ✅ `Scan` transducer - Accumulator pattern
- ✅ `Tap` transducer - Side-effect then pass-through

## Why This Matters

### Zero-Cost Abstraction Principle

The `Step` enum is meant to be a zero-cost abstraction. Using helper functions like `is_stopped()` and `unwrap()` adds unnecessary overhead. Pattern matching is the idiomatic Rust way and compiles to the same assembly as manual checks.

### Hot Path Identification

These optimizations target the hottest paths in the codebase:
1. **Collectors** (`to_vec`, `reduce`) - Called for every element
2. **Early termination** (`Take`) - Called when reaching limit

### Compiler Optimizations

Direct pattern matching allows:
- Better branch prediction
- Inlining opportunities
- Dead code elimination
- Move semantics optimization

## Future Optimization Opportunities

### 1. SIMD Vectorization (Partially Implemented)

Currently implemented for numeric operations in `src/simd.rs`. Could be extended to:
- String operations
- Boolean operations
- Custom types implementing SIMD traits

### 2. Inline Annotations

Review `#[inline(always)]` usage:
- ✅ Already applied to `Map::apply`
- ✅ Already applied to `Filter::apply`
- ✅ Already applied to all transducer `apply` methods

Good coverage, but could benchmark with/without to verify benefit.

### 3. Const Generics for Take

Currently uses `Rc<RefCell<usize>>` for the counter. For small N, could use const generics:

```rust
pub struct TakeConst<T, const N: usize> {
    count: usize,
    _phantom: PhantomData<T>,
}
```

**Trade-off:** Less flexible but zero overhead for the counter.

### 4. Specialization for Common Cases

Could specialize collectors for:
- Identity transducer (just clone)
- Single Map (direct transformation)
- Map + Filter (fused loop)

**Status:** Waiting for Rust specialization to stabilize.

### 5. Arena Allocation

For pipelines that create many intermediate transducers:
```rust
let pipeline = map(...).filter(...).map(...).filter(...);
```

Could use arena allocation for the `Box<dyn Fn>` closures.

**Trade-off:** More complex lifetime management.

## Benchmarking

To verify optimizations:

```bash
# Run Rust benchmarks
cargo bench --target x86_64-unknown-linux-gnu

# Run JavaScript benchmarks
npm run build:nodejs
npm run bench:all
```

## Anti-Patterns to Avoid

### ❌ Don't Use Helper Functions in Hot Paths

```rust
// Bad - function call overhead
if is_stopped(&step) {
    return step.unwrap();
}
```

```rust
// Good - direct pattern matching
match step {
    Step::Stop(value) => return value,
    Step::Continue(value) => value,
}
```

### ❌ Don't Unwrap Steps Unnecessarily

```rust
// Bad - extracts value just to rewrap
stop(result.unwrap())
```

```rust
// Good - pattern match to extract and rewrap
match result {
    Step::Continue(value) | Step::Stop(value) => stop(value),
}
```

### ❌ Don't Create Intermediate Collections

```rust
// Bad - allocates intermediate Vec
let filtered: Vec<_> = data.into_iter().filter(...).collect();
let result: Vec<_> = filtered.into_iter().map(...).collect();
```

```rust
// Good - single pass with transducers
let pipeline = Filter::new(...).compose(Map::new(...));
let result = to_vec(&pipeline, data);
```

## Optimization Checklist

When adding new transducers or collectors:

- [ ] Use pattern matching instead of `is_stopped()` + `unwrap()`
- [ ] Add `#[inline(always)]` to `apply` methods
- [ ] Avoid unnecessary allocations in hot paths
- [ ] Use `Rc<RefCell<>>` only when state is truly needed
- [ ] Test with benchmarks before/after
- [ ] Document performance characteristics
- [ ] Verify all tests pass

## References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [LLVM Optimization Guide](https://llvm.org/docs/Passes.html)
- [Category Theory for Programmers](https://github.com/hmemcpy/milewski-ctfp-pdf)
- [Transducers Paper (Clojure)](https://clojure.org/reference/transducers)

---

**Last Updated:** 2025-01-22
**Optimizations Applied:** 3
**Performance Improvement:** Measurable reduction in function call overhead
