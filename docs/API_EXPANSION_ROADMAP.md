# Orlando API Expansion Roadmap

## Goal: Achieve 1:1+ Feature Parity with Ramda

**Current Status:** 45 operations (14 transducers + 30 collectors + 1 JS helper)
**Ramda List Operations:** ~80+
**Target:** 50+ operations (comprehensive coverage)

**Phase 1 Status:** ✅ COMPLETE (10/10 operations, 171 tests)
**Phase 2a Status:** ✅ COMPLETE (6/6 operations, 35 tests)
**Phase 2b Status:** ✅ COMPLETE (10/10 operations, 96 tests)
**Phase 3 Status:** ✅ COMPLETE (8/8 operations, 42 tests)

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

## Phase 2a: Multi-Input Operations & Hybrid Composition (6 operations)

### **Priority: HIGH** - Extend beyond single-input transducer model

**Architectural Innovation:** Phase 1's Zip/ZipWith revealed that Orlando benefits from supporting operations outside the single-input transducer model. These multi-input helpers enable powerful hybrid compositions.

#### 1. **Merge** (Helper) ⭐⭐⭐
```rust
pub fn merge<T>(streams: Vec<impl Iterator<Item = T>>) -> Vec<T>
// Interleave multiple streams
// [1,2,3] + [4,5,6] -> [1,4,2,5,3,6]
```

**Why critical:** Round-robin combination of multiple data sources.

**Hybrid Composition:**
```rust
// Pre-process streams, then merge
let stream_a = Map::new(|x: i32| x * 2);
let stream_b = Filter::new(|x: &i32| x % 2 == 0);

let a_processed = to_vec(&stream_a, 1..10);
let b_processed = to_vec(&stream_b, 1..20);
let result = merge(vec![a_processed.into_iter(), b_processed.into_iter()]);
```

#### 2. **Intersection** (Helper) ⭐⭐⭐
```rust
pub fn intersection<T: Eq + Hash>(a: Vec<T>, b: Vec<T>) -> Vec<T>
// Set intersection - elements in both A and B
// [1,2,3,4] ∩ [3,4,5,6] -> [3,4]
```

**Why critical:** Common set operation, useful for filtering by membership.

#### 3. **Difference** (Helper) ⭐⭐
```rust
pub fn difference<T: Eq + Hash>(a: Vec<T>, b: Vec<T>) -> Vec<T>
// Set difference - elements in A but not B
// [1,2,3,4] - [3,4,5,6] -> [1,2]
```

**Why important:** Exclusion filtering, data reconciliation.

#### 4. **Union** (Helper) ⭐⭐
```rust
pub fn union<T: Eq + Hash>(a: Vec<T>, b: Vec<T>) -> Vec<T>
// Set union - unique elements from both A and B
// [1,2,3] ∪ [3,4,5] -> [1,2,3,4,5]
```

**Why important:** Combine unique elements from multiple sources.

#### 5. **SymmetricDifference** (Helper) ⭐
```rust
pub fn symmetric_difference<T: Eq + Hash>(a: Vec<T>, b: Vec<T>) -> Vec<T>
// Elements in A or B but not both
// [1,2,3,4] ⊕ [3,4,5,6] -> [1,2,5,6]
```

**Why useful:** Find non-overlapping elements.

#### 6. **Hybrid Composition Pattern** (Documentation) ⭐⭐⭐
Document the pattern of composing transducers with multi-input helpers:

```rust
// Pattern 1: Process then combine
let pipeline_a = Map::new(|x: i32| x * 2)
    .compose(Filter::new(|x: &i32| *x > 5));
let pipeline_b = Map::new(|x: i32| x + 10);

let a_results = to_vec(&pipeline_a, 1..20);
let b_results = to_vec(&pipeline_b, 1..10);
let combined = intersection(a_results, b_results);

// Pattern 2: Combine then process
let merged = merge(vec![stream1, stream2]);
let pipeline = Filter::new(|x: &i32| *x % 2 == 0)
    .compose(Take::new(10));
let result = to_vec(&pipeline, merged);
```

**Why critical:** Demonstrates Orlando's flexibility - transducers where they fit, helpers where they don't.

---

## Phase 2b: High-Value Operations (10 operations)

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

## Phase 6: Optics & Data Access Patterns (5-8 operations)

### **Priority: FUTURE** - Functional optics for data manipulation

**Rationale:** Lenses complement transducers by providing focused access to nested data structures. While transducers transform data flows, lenses enable immutable updates and focused traversals of complex structures.

#### 1. **Lens** (Core optic) ⭐⭐⭐
```rust
pub struct Lens<S, A> {
    get: Box<dyn Fn(&S) -> A>,
    set: Box<dyn Fn(&S, A) -> S>,
}

// Usage
let name_lens = Lens::new(
    |user: &User| user.name.clone(),
    |user: &User, name: String| User { name, ..user.clone() }
);

let user = User { name: "Alice".to_string(), age: 30 };
let updated = name_lens.set(&user, "Bob".to_string());
```

**JavaScript API:**
```javascript
import { lens } from 'orlando-transducers';

const nameLens = lens('name');
const updated = nameLens.set(user, 'Bob');
const name = nameLens.get(user); // 'Bob'
```

**Why critical:** Foundation for all optics. Enables focused immutable updates.

#### 2. **Optional** (Lens for Maybe values) ⭐⭐
```rust
pub struct Optional<S, A> {
    get: Box<dyn Fn(&S) -> Option<A>>,
    set: Box<dyn Fn(&S, A) -> S>,
}

// Usage: Access nested optional fields
let address_lens = Optional::new(
    |user: &User| user.address.as_ref().map(|a| a.clone()),
    |user: &User, addr: Address| User { address: Some(addr), ..user.clone() }
);
```

**Why important:** Safe access to optional/nullable fields without exceptions.

#### 3. **Prism** (Lens for sum types/enums) ⭐⭐
```rust
pub struct Prism<S, A> {
    preview: Box<dyn Fn(&S) -> Option<A>>,
    review: Box<dyn Fn(A) -> S>,
}

// Usage: Pattern match on enum variants
enum Shape { Circle(f64), Rectangle(f64, f64) }

let circle_prism = Prism::new(
    |shape: &Shape| match shape {
        Shape::Circle(r) => Some(*r),
        _ => None,
    },
    |r: f64| Shape::Circle(r)
);
```

**Why important:** Type-safe access to variant data in enums/tagged unions.

#### 4. **Traversal** (Lens for collections) ⭐⭐⭐
```rust
pub struct Traversal<S, A> {
    get_all: Box<dyn Fn(&S) -> Vec<A>>,
    set_all: Box<dyn Fn(&S, Vec<A>) -> S>,
}

// Usage: Access all elements matching a pattern
let all_active_users = Traversal::new(
    |users: &Vec<User>| users.iter().filter(|u| u.active).cloned().collect(),
    |users: &Vec<User>, active: Vec<User>| { /* merge logic */ }
);
```

**Why critical:** Bridge between lenses and collections. Works with transducers!

#### 5. **Iso (Isomorphism)** (Bidirectional conversion) ⭐
```rust
pub struct Iso<S, A> {
    to: Box<dyn Fn(S) -> A>,
    from: Box<dyn Fn(A) -> S>,
}

// Usage: Convert between equivalent representations
let celsius_fahrenheit = Iso::new(
    |c: f64| c * 9.0/5.0 + 32.0,
    |f: f64| (f - 32.0) * 5.0/9.0
);
```

**Why useful:** Lossless conversions between types.

### Integration with Transducers

Lenses naturally compose with transducers for powerful data pipelines:

```javascript
import { Pipeline, lens } from 'orlando-transducers';

// Extract nested property, then transform
const addressLens = lens(['address', 'street']);

const pipeline = new Pipeline()
  .map(addressLens.get)        // Extract street from user.address.street
  .filter(street => street.length > 0)
  .take(10)
  .toArray(users);

// Or: Batch update with transducers
const updatedUsers = new Pipeline()
  .map(user => addressLens.set(user, normalizeStreet(addressLens.get(user))))
  .toArray(users);
```

**Hybrid Composition Pattern:**
```rust
// Pattern 1: Lens → Transducer (extract then process)
let streets = to_vec(
    &Map::new(|user: User| street_lens.get(&user)),
    users
);

// Pattern 2: Transducer → Lens (filter then update)
let active_users = to_vec(&Filter::new(|u: &User| u.active), users);
let updated = active_users.iter()
    .map(|u| name_lens.set(u, normalize_name(name_lens.get(u))))
    .collect();
```

### Advanced: Lens Laws

Lenses must satisfy three laws for correctness:

1. **Get-Put:** `set(s, get(s)) = s` (setting to current value is no-op)
2. **Put-Get:** `get(set(s, a)) = a` (get returns what was set)
3. **Put-Put:** `set(set(s, a), b) = set(s, b)` (last set wins)

Orlando will include property tests to verify these laws.

### Implementation Phases

**Phase 6a: Core Optics (3 operations)**
1. Lens (basic getter/setter)
2. Optional (for nullable fields)
3. Traversal (for collections)

**Phase 6b: Advanced Optics (3 operations)**
4. Prism (for sum types)
5. Iso (bidirectional conversions)
6. Fold (read-only traversal with aggregation)

**Phase 6c: Composition (2 operations)**
7. Compose lenses: `lens1.compose(lens2)`
8. Parallel lenses: `lens1.and(lens2)`

### Why Phase 6 Complements Transducers

| Aspect | Transducers | Lenses |
|--------|-------------|--------|
| **Purpose** | Stream transformation | Focused data access |
| **Direction** | Data flow (input → output) | Bidirectional (get/set) |
| **Composition** | Sequential pipeline | Nested composition |
| **Use Case** | Processing collections | Updating structures |
| **Strength** | Efficient iteration | Immutable updates |

**Together:** Lenses extract data, transducers transform it, lenses write it back.

```javascript
// Real-world example: Update all active user emails
const emailLens = lens('email');

const normalizedEmails = new Pipeline()
  .filter(user => user.active)
  .map(user => emailLens.set(user, user.email.toLowerCase()))
  .toArray(users);
```

---

## Implementation Priorities

### **Phase 1: Must Have** ✅ COMPLETE - 10 operations
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

### **Phase 2a: Multi-Input Operations** ✅ COMPLETE - 6 operations
1. ✅ Merge
2. ✅ Intersection
3. ✅ Difference
4. ✅ Union
5. ✅ SymmetricDifference
6. ✅ Hybrid Composition Pattern (docs)

### **Phase 2b: High-Value Operations** ✅ COMPLETE - 10 operations
11. ✅ CartesianProduct
12. ✅ TopK
13. ✅ ReservoirSample
14. ✅ PartitionBy
15. ✅ Frequencies
16. ✅ ZipLongest
17. ✅ Interpose (RepeatEach)
18. ✅ Unique/UniqueBy
19. ✅ Aperture/Window
20. ✅ TakeLast/DropLast

### **Phase 3: Logic Functions** ✅ COMPLETE - 8 operations
21. ✅ both (predicate AND)
22. ✅ either (predicate OR)
23. ✅ complement (predicate NOT)
24. ✅ all_pass (AND multiple predicates)
25. ✅ any_pass (OR multiple predicates)
26. ✅ When (conditional transform)
27. ✅ Unless (inverse conditional)
28. ✅ IfElse (branch on condition)

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

| Category | Phase 1 Start | Current (v0.2.0 ✅) | Target |
|----------|---------------|---------------------|--------|
| Transducers | 10 | 14 | 25 |
| Collectors | 8 | 30 | 20 |
| Helpers | 0 | 1 (JS Pluck) | 10 |
| **Total** | **18** | **45** | **50** |

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
2. ✅ Implement all Phase 1 operations (10/10)
3. ✅ Add property tests for all Phase 1 operations (171 total tests)
4. ✅ Add JavaScript API for Phase 1 operations
5. ✅ Document Phase 1 operations
6. ✅ Phase 2a: Multi-input operations (Merge, Intersection, Difference, Union, SymmetricDifference)
7. ✅ Phase 2b: High-value collectors (CartesianProduct, TopK, ReservoirSample, PartitionBy, Frequencies, ZipLongest, Aperture, TakeLast, DropLast)
8. ✅ Phase 3: Logic functions (both, either, complement, all_pass, any_pass, When, Unless, IfElse)
9. ⬜ Add hybrid composition documentation and examples to JavaScript docs
10. ⬜ Update JavaScript API documentation with Phase 2b and Phase 3 functions
11. ⬜ Create migration guide: Ramda → Orlando

---

**Last Updated:** 2025-11-03
**Status:** ✅ Phase 1, 2a, 2b (ALL 10/10), and 3 COMPLETE! (45 operations total)
**Priority:** Document Phase 2b and Phase 3 in JavaScript API docs
