# Orlando Project Context

> **Last Updated**: 2025-01-30
> **Current Version**: 0.4.0
> **Status**: Ready for release - PR #7 open

This document provides comprehensive context for future development sessions with Claude Code.

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Current State](#current-state)
3. [Architecture](#architecture)
4. [Development History](#development-history)
5. [Key Files and Structure](#key-files-and-structure)
6. [Testing Strategy](#testing-strategy)
7. [Build and Release Process](#build-and-release-process)
8. [Important Patterns](#important-patterns)
9. [Next Steps](#next-steps)
10. [Common Tasks](#common-tasks)

---

## Project Overview

**Orlando** is a high-performance transducer library written in Rust with WebAssembly bindings for JavaScript/TypeScript. It provides composable, single-pass data transformations with zero intermediate allocations.

### Key Features

- **Composable Transducers**: Build complex data pipelines from simple operations
- **Single-Pass Execution**: No intermediate arrays, process data in one pass
- **Early Termination**: Operations like `take()` stop processing immediately
- **WASM Performance**: 3-19x faster than native JavaScript arrays
- **Functional Lenses**: Clean, immutable updates to nested data structures (NEW in v0.4.0)
- **Streaming Lenses**: First lens library to integrate with transducers (UNIQUE)

### Technology Stack

- **Language**: Rust (1.75+)
- **WASM**: wasm-bindgen, wasm-pack
- **Testing**: proptest (property-based), wasm-bindgen-test
- **CI/CD**: GitHub Actions
- **Package Manager**: npm (for JavaScript distribution)

---

## Current State

### Version 0.4.0 (In PR #7)

**Status**: Ready for merge and release

**Major Addition**: Functional Optics (Lenses)

**Stats**:
- **Total Tests**: 243 passing (229 unit + 127 property + 64 integration + 111 doc)
- **Total Lines**: ~15,000+ (including tests and docs)
- **WASM Bundle Size**: ~50KB minified
- **API Surface**: 60+ operations

**Recent Changes** (4 commits):
1. `e002a69` - Bump version to 0.4.0 for Phase 6 development
2. `4167c41` - Add Phase 6a: Core optics implementation with Lens and Optional
3. `0045a89` - Add JavaScript bindings for optics with WASM tests
4. `ea8d022` - Update documentation for v0.4.0 optics release

### Git Branches

- **main**: Production releases (currently v0.3.0)
- **version-0.4.0**: Current development branch (PR #7 ready to merge)
- **version-0.3.0**: Previous release branch
- **version-0.2.0**: Older release branch

---

## Architecture

### High-Level Structure

```
Orlando
├── Rust Core (src/)           - Pure functional transducers
├── WASM Bindings (src/*_wasm) - JavaScript interop
├── Tests (tests/)             - Integration and WASM tests
└── Documentation (docs/)      - API reference and guides
```

### Core Concepts

#### 1. Transducers

Transducers are composable algorithmic transformations decoupled from input/output sources.

**Key trait**: `Transducer`
```rust
pub trait Transducer<A, B>: Clone {
    type Output;
    fn apply<R, F>(&self, source: impl Iterator<Item = A>, reducer: R, f: F) -> Self::Output
    where
        R: FnMut(Self::Output, B) -> ControlFlow<Self::Output, Self::Output>,
        F: Fn(A) -> B;
}
```

#### 2. Pipeline API (JavaScript)

The main user-facing API for building transducer chains:

```javascript
const pipeline = new Pipeline()
  .map(x => x * 2)
  .filter(x => x > 10)
  .take(5);

const result = pipeline.toArray(data);
```

#### 3. Functional Lenses (NEW in v0.4.0)

Composable, immutable accessors for nested data:

**Rust Types**:
- `Lens<S, A>` - Total focus with get/set/over
- `Optional<S, A>` - Partial focus for nullable fields

**JavaScript API**:
- `lens(property)` - Create simple property lens
- `lensPath(path)` - Create deep nested lens
- `optional(property)` - Create optional lens

**Lens Laws** (all verified via property tests):
1. **GetPut**: `set(obj, get(obj)) ≡ obj`
2. **PutGet**: `get(set(obj, val)) ≡ val`
3. **PutPut**: `set(set(obj, v1), v2) ≡ set(obj, v2)`

#### 4. Hybrid Composition

Orlando supports two composition patterns:
- **Pipeline composition**: Chained operations on single input
- **Multi-input operations**: Set operations (intersection, union, etc.)

```javascript
// Pipeline composition
const pipeline = new Pipeline().filter(x => x > 0).map(x => x * 2);

// Hybrid: combine processed streams
const stream1 = pipeline.toArray(data1);
const stream2 = pipeline.toArray(data2);
const merged = merge([stream1, stream2]);
```

---

## Development History

### Phase 1: Core Transducers (v0.1.0)
- Basic operations: map, filter, take, drop
- Flatmap, partition, find, reject
- Chunk, groupBy, zip
- **10 operations total**

### Phase 2a: Multi-Input Operations (v0.1.0)
- Set operations: merge, intersection, difference, union, symmetric difference
- Hybrid composition pattern
- **6 operations total**

### Phase 2b: Advanced Collectors (v0.1.0)
- Cartesian product, topK, reservoir sample
- PartitionBy, frequencies, zipLongest
- Interpose, unique
- **8 operations total**

### Phase 3: Logic Functions (v0.1.0)
- Predicate combinators: both, either, complement
- Multi-predicate: allPass, anyPass
- Conditional transducers: When, Unless, IfElse
- **8 operations total**

### Phase 4: Statistical Operations (v0.2.0 - Planned)
- Aggregations: product, mean, median
- Variance, standard deviation
- Min/max, quantile, mode
- **10 operations total**

### Phase 5: Collection Utilities (v0.2.0 - Planned)
- Sorting: sortBy, sortWith
- Generators: range, repeat, cycle, unfold
- Path operations: path, pathOr, evolve
- **10 operations total**

### Phase 6a: Functional Optics (v0.4.0 - CURRENT)
- Core lenses: Lens, Optional
- JavaScript bindings: lens(), lensPath(), optional()
- Lens laws verified via property tests
- **Streaming Lenses** - UNIQUE FEATURE
- 24 unit tests + 12 property tests + 14 WASM tests

### Phase 6b: Advanced Optics (Future)
- Prisms (partial optics for sum types)
- Traversals (multiple focus points)
- Isos (bidirectional transformations)

---

## Key Files and Structure

### Source Files

```
src/
├── lib.rs                    # Main library entry, module re-exports
├── transducer.rs             # Core Transducer trait
├── pipeline.rs               # Pipeline struct (not directly exposed to JS)
├── pipeline_wasm.rs          # JavaScript Pipeline API (2,444 lines)
├── operations/               # Transducer implementations
│   ├── map.rs               # Map transducer
│   ├── filter.rs            # Filter transducer
│   ├── take.rs              # Take transducer
│   ├── flatmap.rs           # FlatMap transducer
│   └── ... (20+ more)
├── collectors/              # Terminal operations
│   ├── to_vec.rs           # ToVec collector
│   ├── reduce.rs           # Reduce collector
│   ├── partition.rs        # Partition collector
│   └── ... (15+ more)
├── multi_input/            # Multi-input helpers
│   ├── merge.rs           # Merge operation
│   ├── intersection.rs    # Set intersection
│   └── ... (5+ more)
├── logic/                 # Logic functions
│   ├── predicates.rs     # Predicate combinators
│   └── conditionals.rs   # Conditional transducers
├── optics.rs             # Core lens implementation (948 lines) ⭐ NEW
└── optics_wasm.rs        # Lens JavaScript bindings (364 lines) ⭐ NEW
```

### Test Files

```
tests/
├── integration_tests.rs  # Integration tests (64 tests)
├── property_tests.rs     # Property-based tests (127 tests)
└── wasm_tests.rs        # WASM API tests (includes 14 optics tests) ⭐ UPDATED
```

### Documentation Files

```
docs/
├── api/
│   ├── JAVASCRIPT.md         # Full JavaScript API reference (2,762 lines) ⭐ UPDATED
│   └── RUST.md              # Rust API reference
├── HYBRID_COMPOSITION.md     # Hybrid composition guide
├── PHASE_6_OPTICS_IMPLEMENTATION_PLAN.md  # Optics implementation plan ⭐ NEW
└── ...
examples/
├── REACT_QUICKSTART.md       # React quick start (580 lines) ⭐ UPDATED
├── basic.html
├── advanced-collectors.html
└── ...
```

### Configuration Files

```
Cargo.toml                # Rust package config (version: 0.4.0)
package.json              # npm package config (version: 0.4.0)
wasm-pack.toml           # WASM build config
.github/workflows/       # CI/CD pipelines
.git/hooks/              # Pre-commit and pre-push hooks
```

---

## Testing Strategy

### Test Categories

1. **Unit Tests** (229 tests)
   - Inline in source files with `#[cfg(test)]`
   - Test individual operations in isolation
   - Run with: `cargo test`

2. **Property-Based Tests** (127 tests)
   - In `tests/property_tests.rs`
   - Use proptest to verify mathematical properties
   - **Lens laws verification** ⭐
   - Run with: `cargo test --test property_tests`

3. **Integration Tests** (64 tests)
   - In `tests/integration_tests.rs`
   - Test operation combinations
   - Run with: `cargo test --test integration_tests`

4. **Documentation Tests** (111 tests)
   - Embedded in doc comments
   - Ensure examples compile and run
   - Run with: `cargo test --doc`

5. **WASM Tests** (included in integration)
   - In `tests/wasm_tests.rs`
   - Test JavaScript API surface
   - **14 optics tests** ⭐
   - Run with: `wasm-pack test --node`

### Test Coverage Goals

- **All public APIs** must have tests
- **New features** require property tests where applicable
- **WASM bindings** must have corresponding wasm_tests.rs tests
- **Documentation examples** must be runnable (doctests)

### Running Tests

```bash
# All tests
cargo test --all-features

# Specific test suite
cargo test --test property_tests

# WASM tests
wasm-pack test --node

# With coverage
cargo tarpaulin --out Html
```

---

## Build and Release Process

### Development Build

```bash
# Rust native
cargo build

# WASM
wasm-pack build --target bundler

# With optimizations
wasm-pack build --release --target bundler
```

### Release Checklist

1. **Update version numbers**:
   - `Cargo.toml` (Rust)
   - `package.json` (npm)

2. **Update CHANGELOG.md**:
   - Add release date
   - Document all changes

3. **Run full test suite**:
   ```bash
   cargo test --all-features
   wasm-pack test --node
   cargo clippy --all-targets --all-features
   cargo fmt --check
   ```

4. **Build release artifacts**:
   ```bash
   wasm-pack build --release --target bundler
   ```

5. **Create git tag**:
   ```bash
   git tag -a v0.4.0 -m "Release v0.4.0: Functional Optics"
   git push origin v0.4.0
   ```

6. **Publish to npm**:
   ```bash
   cd pkg
   npm publish
   ```

### Git Hooks

Pre-commit hooks run automatically (`.git/hooks/pre-commit`):
- rustfmt check
- clippy (native and wasm32)
- unit tests
- doc tests
- integration tests
- WASM and native build checks

Pre-push hooks run automatically (`.git/hooks/pre-push`):
- All tests
- Property tests
- Integration tests
- Release build

---

## Important Patterns

### 1. WASM Binding Pattern

All JavaScript-exposed functions follow this pattern:

```rust
#[wasm_bindgen]
pub fn operation_name(args: JsValue) -> Result<JsValue, JsValue> {
    // 1. Convert JsValue to Rust types
    let rust_data = convert_from_js(args)?;

    // 2. Apply operation
    let result = rust_operation(rust_data);

    // 3. Convert back to JsValue
    Ok(convert_to_js(result))
}
```

### 2. Lens Implementation Pattern

Lenses use `Rc` for composition to handle ownership:

```rust
pub struct Lens<S, A> {
    get: Getter<S, A>,    // Type alias: Box<dyn Fn(&S) -> A>
    set: Setter<S, A>,    // Type alias: Box<dyn Fn(&S, A) -> S>
    _phantom: PhantomData<(S, A)>,
}

// Composition uses Rc for shared ownership
pub fn compose<B>(self, other: Lens<A, B>) -> Lens<S, B> {
    let self_rc_get = Rc::new(self.get);
    let self_rc_set = Rc::new(self.set);
    // ... compose with Rc clones
}
```

### 3. Type Aliases for Clippy

Complex types use aliases to satisfy clippy's `type_complexity` lint:

```rust
// Instead of: Box<dyn Fn(&S) -> A>
type Getter<S, A> = Box<dyn Fn(&S) -> A>;

// Instead of: Rc<dyn Fn(&JsValue) -> JsValue>
type JsGetter = Rc<dyn Fn(&JsValue) -> JsValue>;
```

### 4. Property Test Pattern

Use proptest for mathematical laws:

```rust
proptest! {
    #[test]
    fn lens_law_get_put(name in "\\PC+", age in 0u32..150) {
        let user = User { name: name.clone(), age };
        let lens = Lens::field(|u: &User| u.name.clone(), |u, n| {
            User { name: n, age: u.age }
        });

        let result = lens.set(&user, lens.get(&user));
        prop_assert_eq!(result, user);
    }
}
```

### 5. Immutable Updates (JavaScript)

All WASM lens operations use `Object.assign` for structural sharing:

```rust
let new_obj = Object::assign(&Object::new(), obj);
let _ = Reflect::set(&new_obj, &JsValue::from_str(&prop), &value);
new_obj.into()
```

---

## Next Steps

### Immediate (v0.4.0)

- ✅ Merge PR #7
- ✅ Create git tag v0.4.0
- ✅ Publish to npm
- ✅ Create GitHub release with notes

### Phase 4: Statistical Operations (v0.5.0)

Implement statistical functions (already in docs but need implementation):
- `product(array)` - Multiply all numbers
- `mean(array)` - Arithmetic mean
- `median(array)` - Median value
- `variance(array)` - Sample variance
- `stdDev(array)` - Standard deviation
- `min(array)` / `max(array)` - Min/max values
- `minBy(array, keyFn)` / `maxBy(array, keyFn)` - Min/max by key
- `quantile(array, p)` - P-th quantile
- `mode(array)` - Most frequent value

**Note**: These are documented in JAVASCRIPT.md but not yet implemented!

### Phase 5: Collection Utilities (v0.6.0)

Implement utility functions:
- `sortBy(array, keyFn)` - Sort by key function
- `sortWith(array, compareFn)` - Sort with comparator
- `reverse(array)` - Reverse order
- `range(start, end, step)` - Generate numeric sequence
- `repeat(value, n)` - Repeat value N times
- `cycle(array, n)` - Repeat array N times
- `unfold(seed, fn, limit)` - Generate sequence from function
- `path(obj, pathArray)` - Safe nested access
- `pathOr(obj, pathArray, default)` - Safe access with default
- `evolve(obj, transformations)` - Nested transformations

**Note**: These are documented in JAVASCRIPT.md but not yet implemented!

### Phase 6b: Advanced Optics (v0.7.0)

Extend optics with:
- **Prisms** - Partial optics for sum types/variants
- **Traversals** - Multiple focus points (e.g., all array elements)
- **Isos** - Bidirectional transformations
- Lens composition helpers
- Integration with Pipeline API

### Phase 7: Performance Optimizations

- SIMD optimizations for numeric operations
- Parallel processing for large datasets
- Benchmarking suite
- Memory profiling

---

## Common Tasks

### Adding a New Transducer

1. Create `src/operations/my_operation.rs`:
   ```rust
   #[derive(Clone)]
   pub struct MyOperation;

   impl<A: Clone> Transducer<A, A> for MyOperation {
       type Output = Vec<A>;
       fn apply<R, F>(&self, source: impl Iterator<Item = A>, mut reducer: R, f: F) -> Self::Output
       where
           R: FnMut(Self::Output, A) -> ControlFlow<Self::Output, Self::Output>,
           F: Fn(A) -> A,
       {
           // Implementation
       }
   }
   ```

2. Add to `src/lib.rs`:
   ```rust
   pub mod operations {
       pub mod my_operation;
       pub use my_operation::MyOperation;
   }
   ```

3. Add to Pipeline in `src/pipeline_wasm.rs`:
   ```rust
   #[wasm_bindgen]
   impl Pipeline {
       pub fn my_operation(&self) -> Pipeline {
           // Implementation
       }
   }
   ```

4. Add tests in the source file, property tests, and WASM tests

5. Document in `docs/api/JAVASCRIPT.md`

### Adding a New Lens Type

1. Add to `src/optics.rs`:
   ```rust
   pub struct MyLens<S, A> { /* ... */ }
   impl<S, A> MyLens<S, A> { /* methods */ }
   ```

2. Add WASM binding to `src/optics_wasm.rs`:
   ```rust
   #[wasm_bindgen]
   pub struct JsMyLens { /* ... */ }
   ```

3. Add property tests to verify laws

4. Add WASM tests to `tests/wasm_tests.rs`

5. Document in JAVASCRIPT.md with examples

### Debugging WASM Issues

1. Build with debug info:
   ```bash
   wasm-pack build --dev --target bundler
   ```

2. Use console logging from Rust:
   ```rust
   use web_sys::console;
   console::log_1(&format!("Debug: {:?}", value).into());
   ```

3. Test in Node.js:
   ```bash
   wasm-pack test --node
   ```

4. Check generated TypeScript definitions:
   ```bash
   cat pkg/orlando.d.ts
   ```

### Performance Profiling

1. Benchmark with criterion (if added):
   ```bash
   cargo bench
   ```

2. Profile WASM:
   ```bash
   wasm-pack build --release --profiling
   ```

3. Check bundle size:
   ```bash
   ls -lh pkg/orlando_bg.wasm
   ```

---

## Troubleshooting

### Common Issues

**Issue**: Clippy `type_complexity` warnings
- **Solution**: Create type aliases at module level

**Issue**: WASM tests fail with "JsValue doesn't implement X"
- **Solution**: Import `wasm_bindgen::JsCast` and use `.dyn_ref()` or `.as_ref()`

**Issue**: Lens composition ownership errors
- **Solution**: Use `Rc::new()` and `.clone()` for shared ownership

**Issue**: Pre-commit hook fails
- **Solution**: Run `cargo fmt --all` and `cargo clippy --fix --allow-dirty`

**Issue**: Property tests flaky
- **Solution**: Check for non-deterministic behavior, use `prop_assert!` instead of `assert!`

---

## Documentation Standards

### Code Comments

- All public items must have doc comments
- Use `///` for documentation
- Include examples in doc comments (they become tests)
- Reference related functions with `[function_name]`

### Example Format

```rust
/// Transforms each element using the provided function.
///
/// # Examples
///
/// ```
/// use orlando::Pipeline;
///
/// let pipeline = Pipeline::new().map(|x| x * 2);
/// let result = pipeline.to_array(vec![1, 2, 3]);
/// assert_eq!(result, vec![2, 4, 6]);
/// ```
pub fn map<F>(&self, f: F) -> Self { /* ... */ }
```

### Changelog Format

Follow [Keep a Changelog](https://keepachangelog.com/):

```markdown
## [0.4.0] - 2025-01-30

### Added
- Functional lenses with Lens and Optional types
- JavaScript bindings for lens operations

### Changed
- Updated README with optics examples

### Fixed
- None
```

---

## Project Philosophy

### Design Principles

1. **Composability First**: All operations should compose cleanly
2. **Zero-Cost Abstraction**: No runtime overhead for abstractions
3. **Lawful Behavior**: Mathematical properties should be verifiable
4. **JavaScript-Friendly**: WASM API should feel natural to JS developers
5. **Documentation-Driven**: Features should be documented with examples
6. **Test-Driven**: New features require comprehensive tests

### Performance Goals

- Single-pass data processing (no intermediate arrays)
- Early termination for operations like `take()`
- Memory efficiency (bounded memory usage)
- 3-10x faster than native JavaScript for large datasets
- WASM bundle size under 100KB

### API Design Goals

- Intuitive method names (match Ramda/Lodash where sensible)
- TypeScript-friendly (auto-generated .d.ts files)
- Functional purity (no side effects except in `tap()`)
- Error messages should be helpful
- Breaking changes only on major versions

---

## Resources

### Documentation
- [Main README](README.md) - Project overview
- [JavaScript API](docs/api/JAVASCRIPT.md) - Complete API reference
- [Rust API](docs/api/RUST.md) - Rust documentation
- [React Quickstart](examples/REACT_QUICKSTART.md) - React integration guide
- [Hybrid Composition](docs/HYBRID_COMPOSITION.md) - Advanced patterns

### External Resources
- [Transducers Explained](https://clojure.org/reference/transducers) - Clojure transducers
- [Optics Overview](https://www.schoolofhaskell.com/school/to-infinity-and-beyond/pick-of-the-week/a-little-lens-starter-tutorial) - Lens tutorial
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/) - WASM bindings

### Repository
- **GitHub**: https://github.com/justinelliottcobb/Orlando
- **npm**: https://www.npmjs.com/package/orlando-transducers
- **Issues**: https://github.com/justinelliottcobb/Orlando/issues

---

## Version History

- **v0.4.0** (Current) - Functional Optics (Lenses)
- **v0.3.0** (2025-01-24) - CI/CD pipeline and npm publishing
- **v0.2.0** - Statistical operations and collection utilities
- **v0.1.0** (2025-01-23) - Initial release with core transducers, multi-input ops, collectors, logic functions

---

## Contact & Contribution

For questions, issues, or contributions:
- Open an issue: https://github.com/justinelliottcobb/Orlando/issues
- Submit a PR: https://github.com/justinelliottcobb/Orlando/pulls

---

**This CONTEXT.md file should be updated**:
- After each major release
- When architectural decisions change
- When new development phases begin
- When important patterns are established

**Last Updated By**: Claude Code
**Next Review**: After v0.4.0 release
