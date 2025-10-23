# Orlando API Expansion Roadmap

## Goal: Achieve 1:1+ Feature Parity with Ramda

**Current Status:** 18 operations (10 transducers + 8 collectors)
**Ramda List Operations:** ~80+
**Target:** 50+ operations (comprehensive coverage)

## Classification: Transducer vs. Collector vs. Helper

### Transducers (Streaming, Composable)
Operations that can process elements one-at-a-time without seeing the whole collection.

### Collectors (Terminal Operations)
Operations that consume the stream and produce a final result.

### Helpers (Utilities)
Operations that don't fit the transducer model (sorting, reversing, etc.).

---

## Phase 1: Critical Missing Operations (10 operations)

### **Priority: HIGH** - Core functional programming operations

#### 1. **FlatMap / Chain** (Transducer) ⭐⭐⭐
```rust
pub struct FlatMap<F, In, Out> {
    f: Rc<F>,
    _phantom: PhantomData<(In, Out)>,
}

// Usage
let pipeline = FlatMap::new(|x: i32| vec![x, x * 2, x * 3]);
// [1, 2] -> [1, 2, 3, 2, 4, 6]
```

**Why critical:** Essential for working with nested data, monadic operations.

**JavaScript API:**
```javascript
new Pipeline()
  .flatMap(x => [x, x * 2])
  .toArray([1, 2, 3])  // [1, 2, 2, 4, 3, 6]
```

#### 2. **Partition** (Collector) ⭐⭐⭐
```rust
pub fn partition<T, U, Iter, P>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    predicate: P
) -> (Vec<U>, Vec<U>)
```

**Why critical:** Common pattern for splitting data into pass/fail groups.

**JavaScript API:**
```javascript
const [evens, odds] = pipeline.partition(data, x => x % 2 === 0);
```

#### 3. **Reject** (Transducer) ⭐⭐
```rust
pub struct Reject<P, T> {
    predicate: Rc<P>,
    _phantom: PhantomData<T>,
}
// Inverse of filter - keeps elements that DON'T match
```

**Why important:** More intuitive than `filter(x => !predicate(x))` for exclusion logic.

#### 4. **Find** (Collector) ⭐⭐⭐
```rust
pub fn find<T, U, Iter, P>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    predicate: P
) -> Option<U>
```

**Why critical:** Common pattern, benefits from early termination.

#### 5. **Chunk / SplitEvery** (Transducer) ⭐⭐
```rust
pub struct Chunk<T> {
    size: usize,
    buffer: Rc<RefCell<Vec<T>>>,
}

// Usage: [1,2,3,4,5,6] with chunk(2) -> [[1,2], [3,4], [5,6]]
```

**Why important:** Batch processing, pagination, windowing.

#### 6. **Zip / ZipWith** (Helper or Transducer) ⭐⭐⭐
```rust
pub fn zip<T, U>(a: Vec<T>, b: Vec<U>) -> Vec<(T, U)>
pub fn zip_with<T, U, V, F>(a: Vec<T>, b: Vec<U>, f: F) -> Vec<V>
```

**Why critical:** Combining parallel data streams.

**Challenge:** Doesn't fit single-input transducer model. May need special implementation.

#### 7. **TakeLast / DropLast** (Helper) ⭐⭐
```rust
pub fn take_last<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    n: usize
) -> Vec<U>
```

**Challenge:** Requires buffering entire stream (can't be pure transducer).

#### 8. **GroupBy** (Collector) ⭐⭐⭐
```rust
pub fn group_by<T, U, K, Iter, F>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    key_fn: F
) -> HashMap<K, Vec<U>>
```

**Why critical:** Common aggregation pattern.

#### 9. **Pluck** (Transducer for JS) ⭐⭐
```javascript
// Extract property from objects
new Pipeline()
  .pluck('name')  // x => x.name
  .toArray(users)
```

**Note:** In Rust, this is just `Map::new(|x| x.field)`. Mainly a JavaScript convenience.

#### 10. **None** (Collector) ⭐
```rust
pub fn none<T, U, Iter, P>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    predicate: P
) -> bool
// Inverse of some - true if NO elements match
```

---

## Phase 2: High-Value Operations (10 operations)

### **Priority: MEDIUM** - Commonly used utilities

#### 11. **Aperture / Window** (Transducer) ⭐⭐
```rust
pub struct Aperture<T> {
    size: usize,
    buffer: Rc<RefCell<VecDeque<T>>>,
}

// [1,2,3,4,5] with aperture(3) -> [[1,2,3], [2,3,4], [3,4,5]]
```

**Why useful:** Sliding window analysis, moving averages.

#### 12. **Intersperse** (Transducer) ⭐
```rust
pub struct Intersperse<T> {
    separator: T,
    first: Rc<RefCell<bool>>,
}

// [1,2,3] with separator ',' -> [1, ',', 2, ',', 3]
```

#### 13. **Concat / Append / Prepend** (Collectors) ⭐
```rust
pub fn concat<T, U, Iter1, Iter2>(
    transducer: &impl Transducer<T, U>,
    source1: Iter1,
    source2: Iter2
) -> Vec<U>
```

#### 14. **Contains / Includes** (Collector) ⭐⭐
```rust
pub fn contains<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    value: U
) -> bool
```

#### 15. **StartsWith / EndsWith** (Collectors) ⭐
```rust
pub fn starts_with<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    prefix: Vec<U>
) -> bool
```

#### 16. **Tail / Init** (Helpers) ⭐
```rust
pub fn tail<T>(vec: Vec<T>) -> Vec<T>  // All but first
pub fn init<T>(vec: Vec<T>) -> Vec<T>  // All but last
```

#### 17. **Nth** (Collector) ⭐
```rust
pub fn nth<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    n: usize
) -> Option<U>
```

#### 18. **FindIndex / FindLastIndex** (Collectors) ⭐
```rust
pub fn find_index<T, U, Iter, P>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    predicate: P
) -> Option<usize>
```

#### 19. **SplitAt / SplitWhen** (Collectors) ⭐
```rust
pub fn split_at<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    n: usize
) -> (Vec<U>, Vec<U>)
```

#### 20. **UniqWith** (Transducer) ⭐
```rust
pub struct UniqWith<F, T> {
    comparator: Rc<F>,
    seen: Rc<RefCell<Vec<T>>>,
}
// Custom equality comparator
```

---

## Phase 3: Aggregation & Math (5 operations)

### **Priority: MEDIUM** - Statistical operations

#### 21. **Product** (Collector)
```rust
pub fn product<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter
) -> U
where U: Mul<Output = U> + From<u8>
```

#### 22. **Mean** (Collector)
```rust
pub fn mean<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter
) -> f64
```

#### 23. **Median** (Collector)
```rust
pub fn median<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter
) -> Option<f64>
```

#### 24. **Min / Max** (Collectors)
```rust
pub fn min<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter
) -> Option<U>
where U: Ord
```

#### 25. **MinBy / MaxBy** (Collectors)
```rust
pub fn min_by<T, U, K, Iter, F>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    key_fn: F
) -> Option<U>
where K: Ord
```

---

## Phase 4: Advanced Operations (10 operations)

### **Priority: LOW** - Nice-to-have, less common

#### 26-30. **Set Operations** (Collectors)
- `union` - Combine unique elements
- `intersection` - Common elements
- `difference` - Elements in A but not B
- `symmetric_difference` - Elements in A or B but not both
- `cartesian_product` - All pairs

#### 31-32. **Sorting** (Helpers)
- `sort_by` - Sort by key function
- `sort_with` - Sort with custom comparator

**Note:** Sorting doesn't fit transducer model (requires full collection).

#### 33. **Reverse** (Helper)
```rust
pub fn reverse<T>(vec: Vec<T>) -> Vec<T>
```

**Note:** Requires full collection, can't stream.

#### 34. **Unfold** (Generator)
```rust
pub fn unfold<T, F>(seed: T, f: F) -> impl Iterator<Item = T>
```

#### 35. **Repeat / Cycle** (Generators)
```rust
pub fn repeat<T>(value: T, n: usize) -> Vec<T>
pub fn cycle<T>(vec: Vec<T>, n: usize) -> Vec<T>
```

---

## Phase 5: JavaScript-Specific Enhancements (5 operations)

### **Priority: MEDIUM** - Better DX for JS users

#### 36. **Pluck** (JS convenience)
```javascript
pipeline.pluck('name')  // Cleaner than .map(x => x.name)
```

#### 37. **Project** (JS convenience)
```javascript
pipeline.project(['id', 'name'])  // Extract multiple fields
```

#### 38. **Compact** (Transducer)
```javascript
pipeline.compact()  // Remove null/undefined/false/0/''
```

#### 39. **Flatten with depth** (Transducer)
```javascript
pipeline.flatten(2)  // Flatten nested arrays to depth 2
```

#### 40. **Where** (Filter shorthand)
```javascript
pipeline.where({ active: true, role: 'admin' })
// Same as: filter(x => x.active === true && x.role === 'admin')
```

---

## Implementation Priorities

### **Must Have (Start Here)** - 10 operations
1. ✅ FlatMap
2. ✅ Partition
3. ✅ Find
4. ✅ Reject
5. ✅ Chunk
6. ✅ GroupBy
7. ✅ None
8. ✅ Contains
9. ✅ Zip/ZipWith
10. ✅ Pluck (JS)

### **Should Have (Next Sprint)** - 10 operations
11. Aperture
12. TakeLast/DropLast
13. Concat
14. Intersperse
15. Nth
16. FindIndex
17. SplitAt
18. UniqWith
19. StartsWith/EndsWith
20. Tail/Init

### **Nice to Have (Future)** - 15 operations
21-35. Aggregation, set operations, sorting, etc.

### **JS Enhancements (Ongoing)** - 5 operations
36-40. Developer experience improvements for JavaScript users

---

## Implementation Strategy

### 1. Transducers (Pure Streaming)
Implement as new structs in `src/transforms.rs`:
```rust
pub struct FlatMap<F, In, Out> { ... }
pub struct Reject<P, T> { ... }
pub struct Chunk<T> { ... }
// etc.
```

### 2. Collectors (Terminal Operations)
Add to `src/collectors.rs`:
```rust
pub fn partition<T, U>(...) -> (Vec<U>, Vec<U>) { ... }
pub fn find<T, U>(...) -> Option<U> { ... }
pub fn group_by<T, U, K>(...) -> HashMap<K, Vec<U>> { ... }
// etc.
```

### 3. JavaScript API
Update `src/pipeline.rs` with fluent methods:
```rust
#[wasm_bindgen]
impl Pipeline {
    pub fn flat_map(&self, f: &Function) -> Pipeline { ... }
    pub fn reject(&self, pred: &Function) -> Pipeline { ... }
    pub fn chunk(&self, size: usize) -> Pipeline { ... }
    // etc.
}
```

### 4. Property Tests
Add for each new operation in `tests/property_tests.rs`:
```rust
proptest! {
    fn test_flatmap_flattens(vec in ...) { ... }
    fn test_partition_splits(vec in ...) { ... }
    fn test_chunk_sizes(vec in ...) { ... }
}
```

### 5. Documentation
- Update `docs/api/JAVASCRIPT.md`
- Add examples to README
- Create Ramda migration guide

---

## Architectural Considerations

### Challenge: Multi-Input Operations

**Problem:** Transducers work on single input streams. Operations like `zip`, `concat` require multiple inputs.

**Solutions:**

1. **Special helpers** (not transducers):
```rust
pub fn zip<T, U>(a: Vec<T>, b: Vec<U>) -> Vec<(T, U)>
```

2. **Currying**:
```rust
pub fn zip_with<T, U>(other: Vec<U>) -> impl Transducer<T, (T, U)>
```

3. **Builder pattern**:
```rust
Pipeline::zip([stream1, stream2, stream3])
    .map(|(a, b, c)| ...)
```

### Challenge: Operations Requiring Full Collection

Some operations can't stream (sorting, reversing, take_last):

**Solutions:**

1. **Helpers** (not transducers):
```rust
pub fn sort_by<T, F>(vec: Vec<T>, key_fn: F) -> Vec<T>
```

2. **Buffering transducers** (less efficient but composable):
```rust
pub struct TakeLast<T> {
    n: usize,
    buffer: Rc<RefCell<VecDeque<T>>>,
}
```

3. **Documentation** - Clearly mark which operations break streaming.

---

## Success Metrics

**Target:** 50+ operations (up from 18)

| Category | Current | Target |
|----------|---------|--------|
| Transducers | 10 | 25 |
| Collectors | 8 | 20 |
| Helpers | 0 | 5 |
| **Total** | **18** | **50** |

**Coverage Goals:**
- ✅ 100% of Ramda's high-frequency operations
- ✅ 80%+ of Ramda's list operations
- ✅ Property tests for all new operations
- ✅ JavaScript/TypeScript examples for all

---

## Timeline Estimate

**Phase 1 (Critical):** 2-3 weeks
- 10 must-have operations
- Full test coverage
- JavaScript API
- Documentation

**Phase 2 (High-Value):** 2 weeks
- 10 commonly used operations
- Test coverage
- Examples

**Phase 3 (Aggregation):** 1 week
- 5 math operations
- Test coverage

**Phase 4 (Advanced):** 2 weeks
- 10 nice-to-have operations
- Test coverage

**Phase 5 (JS Enhancements):** Ongoing
- Developer experience improvements
- Based on user feedback

**Total:** ~8-10 weeks for comprehensive Ramda parity

---

## Next Steps

1. ✅ Create this roadmap
2. ⬜ Implement Phase 1, Operation 1: FlatMap
3. ⬜ Add property tests for FlatMap
4. ⬜ Add JavaScript API for FlatMap
5. ⬜ Document FlatMap
6. ⬜ Repeat for remaining Phase 1 operations
7. ⬜ Create migration guide: Ramda → Orlando

---

**Last Updated:** 2025-01-22
**Status:** Planning phase complete, ready for implementation
**Priority:** Start with Phase 1 (10 critical operations)
