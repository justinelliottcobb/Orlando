# Hybrid Composition: Combining Transducers with Multi-Input Operations

## Overview

**Hybrid composition** is Orlando's architectural pattern for combining single-input transducers with multi-input helper functions. This pattern demonstrates Orlando's pragmatic flexibility: using the right abstraction for each job.

### The Core Principle

- **Transducers**: Best for single-input streaming transformations (map, filter, take, etc.)
- **Multi-Input Helpers**: Best for operations requiring multiple streams (merge, intersection, union, etc.)
- **Hybrid Composition**: Combine both for maximum expressiveness

## Why Hybrid Composition?

Traditional functional libraries force everything into one model:
- Pure transducer libraries can't handle multi-input operations elegantly
- Pure collection-based libraries create intermediate allocations
- Orlando combines the best of both worlds

## The Two Fundamental Patterns

### Pattern 1: Process → Combine

Transform multiple streams independently with transducers, then combine the results with multi-input helpers.

```rust
use orlando::{Map, Filter, to_vec, merge, intersection};

// Process each stream independently
let pipeline_a = Map::new(|x: i32| x * 2);
let pipeline_b = Filter::new(|x: &i32| x % 2 == 0);

let a_result = to_vec(&pipeline_a, vec![1, 2, 3, 4, 5]);
let b_result = to_vec(&pipeline_b, vec![1, 2, 3, 4, 5]);

// Combine the processed results
let merged = merge(vec![a_result, b_result]);
// merged: [2, 2, 4, 4, 6, 8, 10]
```

**When to use:**
- When each stream needs different transformations
- When you want to process streams in parallel before combining
- When combining is the final step

### Pattern 2: Combine → Process

Combine multiple streams first with multi-input helpers, then process the combined result with transducers.

```rust
use orlando::{Map, Filter, Compose, Transducer, to_vec, merge};

// Combine streams first
let stream1 = vec![1, 2, 3];
let stream2 = vec![4, 5, 6];
let merged = merge(vec![stream1, stream2]);

// Process the combined result
let pipeline = Map::new(|x: i32| x * 2)
    .compose(Filter::new(|x: &i32| *x > 5));

let result = to_vec(&pipeline, merged);
// result: [6, 8, 10, 12]
```

**When to use:**
- When the same transformation applies to all streams
- When you want to deduplicate or combine before processing
- When combining is an intermediate step

## Multi-Input Operations

Orlando provides 5 multi-input helpers for different use cases:

### 1. Merge - Round-Robin Interleaving

Interleaves elements from multiple streams in round-robin fashion.

```rust
use orlando::merge;

let a = vec![1, 2, 3];
let b = vec![4, 5, 6];
let c = vec![7, 8, 9];

let result = merge(vec![a, b, c]);
// result: [1, 4, 7, 2, 5, 8, 3, 6, 9]
```

**Hybrid Example:**
```rust
use orlando::{Map, to_vec, merge};

// Process each stream differently
let evens = to_vec(&Map::new(|x: i32| x * 2), 1..5);
let odds = to_vec(&Map::new(|x: i32| x * 2 + 1), 1..5);

let alternating = merge(vec![evens, odds]);
// alternating: [2, 3, 4, 5, 6, 7, 8, 9]
```

**Use Cases:**
- Round-robin scheduling
- Interleaving data sources
- Creating alternating patterns

### 2. Intersection - Common Elements

Returns elements that appear in both collections.

```rust
use orlando::intersection;

let a = vec![1, 2, 3, 4, 5];
let b = vec![3, 4, 5, 6, 7];

let common = intersection(a, b);
// common: [3, 4, 5]
```

**Hybrid Example:**
```rust
use orlando::{Map, Filter, to_vec, intersection};

// Process streams to find common transformed values
let pipeline_a = Map::new(|x: i32| x * 2);
let pipeline_b = Map::new(|x: i32| x + 5);

let a_processed = to_vec(&pipeline_a, 1..10);    // [2, 4, 6, 8, 10, 12, 14, 16, 18]
let b_processed = to_vec(&pipeline_b, 1..10);    // [6, 7, 8, 9, 10, 11, 12, 13, 14]

let common = intersection(a_processed, b_processed);
// common: [6, 8, 10, 12, 14]
```

**Use Cases:**
- Finding matching records across datasets
- Filtering by membership in another set
- Database-style joins

### 3. Difference - Exclusion

Returns elements in the first collection but not the second.

```rust
use orlando::difference;

let a = vec![1, 2, 3, 4, 5];
let b = vec![3, 4, 5, 6, 7];

let unique_to_a = difference(a, b);
// unique_to_a: [1, 2]
```

**Hybrid Example:**
```rust
use orlando::{Filter, to_vec, difference};

// Find elements that pass filter A but not filter B
let filter_a = Filter::new(|x: &i32| *x > 5);
let filter_b = Filter::new(|x: &i32| *x % 2 == 0);

let data = 1..20;
let passed_a = to_vec(&filter_a, data.clone());  // [6, 7, 8, 9, 10, ...]
let passed_b = to_vec(&filter_b, data);          // [2, 4, 6, 8, 10, ...]

let only_in_a = difference(passed_a, passed_b);
// only_in_a: [7, 9, 11, 13, 15, 17, 19]
```

**Use Cases:**
- Finding new/deleted items
- Exclusion lists
- Data reconciliation

### 4. Union - Unique Combined Elements

Returns all unique elements from both collections.

```rust
use orlando::union;

let a = vec![1, 2, 3];
let b = vec![3, 4, 5];

let all_unique = union(a, b);
// all_unique: [1, 2, 3, 4, 5]
```

**Hybrid Example:**
```rust
use orlando::{Map, Unique, to_vec, union};

// Combine unique values from multiple processed streams
let pipeline = Map::new(|x: i32| x % 5);

let stream_a = to_vec(&pipeline, 1..10);   // [1, 2, 3, 4, 0, 1, 2, 3, 4]
let stream_b = to_vec(&pipeline, 5..15);   // [0, 1, 2, 3, 4, 0, 1, 2, 3, 4]

let all_mods = union(stream_a, stream_b);
// all_mods: [1, 2, 3, 4, 0]
```

**Use Cases:**
- Combining datasets without duplicates
- Creating master lists
- Deduplication across sources

### 5. Symmetric Difference - Unique to Each

Returns elements that appear in exactly one collection (not both).

```rust
use orlando::symmetric_difference;

let a = vec![1, 2, 3, 4];
let b = vec![3, 4, 5, 6];

let unique_to_each = symmetric_difference(a, b);
// unique_to_each: [1, 2, 5, 6]
```

**Hybrid Example:**
```rust
use orlando::{Filter, to_vec, symmetric_difference};

// Find elements that pass exactly one of two filters
let filter_positive = Filter::new(|x: &i32| *x > 0);
let filter_even = Filter::new(|x: &i32| *x % 2 == 0);

let data = -5..5;
let positives = to_vec(&filter_positive, data.clone());  // [1, 2, 3, 4]
let evens = to_vec(&filter_even, data);                  // [-4, -2, 0, 2, 4]

let exclusive = symmetric_difference(positives, evens);
// exclusive: [1, 3, -4, -2, 0]
```

**Use Cases:**
- Finding changed items
- XOR operations
- Detecting differences between versions

## Real-World Examples

### Example 1: Data Pipeline with Multiple Sources

```rust
use orlando::{Map, Filter, to_vec, merge};

// Process logs from different servers
let process_logs = Map::new(|log: String| parse_log(log))
    .compose(Filter::new(|entry: &LogEntry| entry.level == "ERROR"));

let server1_errors = to_vec(&process_logs, server1_logs);
let server2_errors = to_vec(&process_logs, server2_logs);
let server3_errors = to_vec(&process_logs, server3_logs);

// Merge all errors chronologically (assuming merge preserves time order)
let all_errors = merge(vec![server1_errors, server2_errors, server3_errors]);
```

### Example 2: Finding Common Users Across Datasets

```rust
use orlando::{Map, to_vec, intersection};

// Extract user IDs from different datasets
let extract_ids = Map::new(|record: Record| record.user_id);

let active_users = to_vec(&extract_ids, activity_data);
let premium_users = to_vec(&extract_ids, billing_data);

// Find users who are both active and premium
let active_premium = intersection(active_users, premium_users);
```

### Example 3: Combining Filtered Results

```rust
use orlando::{Filter, Take, to_vec, union};

// Get top items from multiple categories
let top_tech = to_vec(
    &Filter::new(|item: &Item| item.category == "tech")
        .compose(Take::new(10)),
    all_items.clone()
);

let top_books = to_vec(
    &Filter::new(|item: &Item| item.category == "books")
        .compose(Take::new(10)),
    all_items
);

// Combine top items from both categories
let featured = union(top_tech, top_books);
```

### Example 4: Complex Multi-Stage Pipeline

```rust
use orlando::{Map, Filter, Unique, to_vec, intersection, difference};

// Stage 1: Process multiple data sources
let normalize = Map::new(|s: String| s.to_lowercase().trim());
let filter_valid = Filter::new(|s: &String| !s.is_empty());

let source_a = to_vec(&normalize.compose(filter_valid.clone()), raw_data_a);
let source_b = to_vec(&normalize.compose(filter_valid), raw_data_b);

// Stage 2: Find common valid entries
let common = intersection(source_a.clone(), source_b.clone());

// Stage 3: Find entries unique to source A
let unique_a = difference(source_a, common.clone());

// Stage 4: Process common entries further
let process_common = Map::new(|s: String| s.to_uppercase())
    .compose(Unique::new());

let final_result = to_vec(&process_common, common);
```

## Performance Considerations

### Memory Efficiency

**Single-Pass Transducers:**
- ✅ No intermediate allocations
- ✅ Constant memory (for most operations)
- ✅ Stream processing

**Multi-Input Helpers:**
- ⚠️ Materialize results (require Vec or HashSet)
- ⚠️ O(n) or O(n+m) memory
- ✅ Optimized for their specific use case

### When to Use Each Pattern

**Process → Combine** (Pattern 1):
- ✅ Best when each stream needs different transformations
- ✅ Allows parallel processing of streams
- ✅ Clear separation of concerns

**Combine → Process** (Pattern 2):
- ✅ Best when same transformation applies to all
- ✅ Reduces code duplication
- ✅ Better for subsequent transducer compositions

### Optimization Tips

1. **Minimize multi-input operations:**
   ```rust
   // Less efficient: multiple combines
   let merged1 = merge(vec![a, b]);
   let merged2 = merge(vec![merged1, c]);

   // More efficient: single combine
   let merged = merge(vec![a, b, c]);
   ```

2. **Filter early:**
   ```rust
   // Better: filter before combining
   let a_filtered = to_vec(&filter, stream_a);
   let b_filtered = to_vec(&filter, stream_b);
   let combined = merge(vec![a_filtered, b_filtered]);

   // vs. filtering after (processes more data)
   let combined = merge(vec![stream_a, stream_b]);
   let filtered = to_vec(&filter, combined);
   ```

3. **Use the right set operation:**
   ```rust
   // If you only need membership testing, use HashSet directly
   use std::collections::HashSet;

   let set_b: HashSet<_> = b.into_iter().collect();
   let filtered: Vec<_> = a.into_iter()
       .filter(|x| set_b.contains(x))
       .collect();
   ```

## Architectural Benefits

### 1. Flexibility

Orlando doesn't force every operation into the transducer model. Multi-input operations are standalone helpers because that's the right abstraction.

### 2. Composability

You can freely mix and match:
```rust
// Transducer composition
let pipeline = map.compose(filter).compose(take);

// Helper composition
let combined = intersection(union(a, b), c);

// Hybrid composition
let result = to_vec(&pipeline, merge(vec![x, y]));
```

### 3. Clarity

The code reflects the data flow:
```rust
// Clear: process each differently, then combine
let processed_a = to_vec(&pipeline_a, source_a);
let processed_b = to_vec(&pipeline_b, source_b);
let result = merge(vec![processed_a, processed_b]);
```

### 4. Testability

Each stage can be tested independently:
```rust
#[test]
fn test_processing() {
    let result = to_vec(&pipeline, test_data);
    assert_eq!(result, expected);
}

#[test]
fn test_combining() {
    let result = merge(vec![stream_a, stream_b]);
    assert_eq!(result, expected);
}
```

## Comparison with Other Libraries

### vs. Ramda/Lodash (JavaScript)

**Ramda/Lodash:**
- Create intermediate arrays between operations
- Limited multi-input operations
- No early termination

**Orlando:**
- ✅ Zero intermediate allocations (transducers)
- ✅ Rich set of multi-input operations
- ✅ Early termination built-in
- ✅ Hybrid composition for best of both

### vs. Pure Transducer Libraries

**Pure Transducers:**
- Can't elegantly handle multi-input operations
- Have to work around single-input limitation

**Orlando:**
- ✅ Single-input transducers where they fit
- ✅ Multi-input helpers where they're needed
- ✅ Pragmatic flexibility

### vs. Rust Iterators

**Rust Iterators:**
- Excellent for single-stream processing
- Limited multi-stream support (zip only)
- No JavaScript/WASM integration

**Orlando:**
- ✅ Similar performance to iterators
- ✅ Rich multi-stream operations
- ✅ JavaScript/WASM ready
- ✅ Category theory foundation

## Best Practices

### 1. Choose the Right Pattern

```rust
// ✅ Good: Same transformation for both
let merged = merge(vec![a, b]);
let result = to_vec(&pipeline, merged);

// ❌ Wasteful: Different transformations
let merged = merge(vec![a, b]);
let result = to_vec(&pipeline_a, merged);  // Can't differentiate!
```

### 2. Minimize Materialization

```rust
// ✅ Good: Single materialization
let processed = to_vec(&pipeline, data);
let result = intersection(processed, reference);

// ❌ Wasteful: Multiple materializations
let intermediate1 = to_vec(&map, data);
let intermediate2 = to_vec(&filter, intermediate1);
let result = intersection(intermediate2, reference);

// ✅ Better: Compose transducers first
let pipeline = map.compose(filter);
let processed = to_vec(&pipeline, data);
let result = intersection(processed, reference);
```

### 3. Document Complex Flows

```rust
// Complex hybrid composition? Add comments!

// Stage 1: Extract and normalize user data from both sources
let extract = Map::new(|record| record.user_id.to_lowercase());
let users_a = to_vec(&extract, source_a);
let users_b = to_vec(&extract, source_b);

// Stage 2: Find users in both systems (intersection)
let common_users = intersection(users_a.clone(), users_b.clone());

// Stage 3: Find users only in system A (difference)
let a_only = difference(users_a, common_users);
```

## Future Enhancements

Potential additions to the hybrid composition toolkit:

1. **Zip** - Already implemented as a multi-input helper
2. **Concat** - Append streams sequentially (vs. merge which interleaves)
3. **Cartesian Product** - All pairs from two streams
4. **GroupBy Multiple** - Group by multiple key functions
5. **Partition Multiple** - Split into N groups based on predicates

## Conclusion

Hybrid composition is what makes Orlando unique:

- **Not dogmatic** - Uses the right tool for each job
- **Highly expressive** - Combine single and multi-input operations freely
- **Performance-conscious** - Minimize allocations where possible
- **Practical** - Solves real-world problems elegantly

By combining transducers with multi-input helpers, Orlando achieves what neither approach can do alone: **efficient, composable, and expressive data transformation** for both Rust and JavaScript.
