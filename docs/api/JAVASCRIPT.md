# Orlando JavaScript/TypeScript API Documentation

Complete API reference for using Orlando transducers in JavaScript and TypeScript applications.

## Installation

```bash
npm install orlando-transducers
```

Or use directly from a CDN:

```html
<script type="module">
  import init, { Pipeline } from './pkg/orlando.js';
  await init();
  // Use Pipeline...
</script>
```

## Quick Start

```javascript
import init, { Pipeline } from 'orlando-transducers';

// Initialize WASM module
await init();

// Create a pipeline
const pipeline = new Pipeline()
  .map(x => x * 2)
  .filter(x => x > 10)
  .take(5);

// Execute on data
const data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const result = pipeline.toArray(data);

console.log(result); // [12, 14, 16, 18, 20]
```

## Core Concepts

### Transducers vs Array Methods

**Traditional approach** (creates intermediate arrays):
```javascript
const result = data
  .map(x => x * 2)        // creates intermediate array
  .filter(x => x > 10)    // creates another intermediate array
  .slice(0, 5);           // final result
```

**Orlando approach** (single pass, no intermediates):
```javascript
const pipeline = new Pipeline()
  .map(x => x * 2)
  .filter(x => x > 10)
  .take(5);

const result = pipeline.toArray(data); // single pass!
```

### Performance Benefits

1. **No intermediate allocations** - Processes data in a single pass
2. **Early termination** - Stops processing as soon as possible
3. **Composable** - Build complex pipelines from simple operations
4. **WASM-powered** - Native performance via WebAssembly

## API Reference

### Pipeline Class

The main entry point for building transducer pipelines.

#### Constructor

```typescript
new Pipeline(): Pipeline
```

Creates a new empty pipeline.

```javascript
const pipeline = new Pipeline();
```

---

### Transformation Methods

All transformation methods return a new `Pipeline` instance, allowing for method chaining.

#### `map(fn)`

Transforms each value using the provided function.

```typescript
map(fn: (value: T) => U): Pipeline
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .map(x => x * 2)
  .map(x => x + 1);

pipeline.toArray([1, 2, 3]); // [3, 5, 7]
```

**Use cases:**
- Data transformation
- Property extraction
- Type conversion
- Calculations

---

#### `filter(predicate)`

Keeps only values that match the predicate.

```typescript
filter(predicate: (value: T) => boolean): Pipeline
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .filter(x => x % 2 === 0)
  .filter(x => x > 10);

pipeline.toArray([1, 5, 12, 20, 3]); // [12, 20]
```

**Use cases:**
- Filtering data
- Validation
- Conditional inclusion

---

#### `take(n)`

Takes the first `n` elements, then stops processing.

```typescript
take(n: number): Pipeline
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .filter(x => x % 2 === 0)
  .take(3);

// Only processes until 3 evens are found!
pipeline.toArray([1, 2, 3, 4, 5, 6, 7, 8]); // [2, 4, 6]
```

**Use cases:**
- Pagination
- Limiting results
- Top-N queries
- Early termination for performance

**Performance note:** This is where Orlando shines! It stops processing the moment it has enough elements.

---

#### `takeWhile(predicate)`

Takes elements while the predicate is true, then stops.

```typescript
takeWhile(predicate: (value: T) => boolean): Pipeline
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .takeWhile(x => x < 100);

pipeline.toArray([1, 5, 50, 200, 10]); // [1, 5, 50]
```

**Use cases:**
- Taking until a condition
- Reading until delimiter
- Streaming data processing

---

#### `drop(n)`

Skips the first `n` elements.

```typescript
drop(n: number): Pipeline
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .drop(3);

pipeline.toArray([1, 2, 3, 4, 5]); // [4, 5]
```

**Use cases:**
- Pagination (skip)
- Removing headers
- Offset-based queries

---

#### `dropWhile(predicate)`

Skips elements while the predicate is true.

```typescript
dropWhile(predicate: (value: T) => boolean): Pipeline
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .dropWhile(x => x < 10);

pipeline.toArray([1, 5, 12, 20, 3]); // [12, 20, 3]
```

**Use cases:**
- Skipping headers
- Removing prefixes
- Starting from a condition

---

#### `tap(fn)`

Performs side effects without modifying values.

```typescript
tap(fn: (value: T) => void): Pipeline
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .tap(x => console.log('Processing:', x))
  .map(x => x * 2)
  .tap(x => console.log('Result:', x));

pipeline.toArray([1, 2, 3]);
// Logs:
// Processing: 1
// Result: 2
// Processing: 2
// Result: 4
// Processing: 3
// Result: 6
```

**Use cases:**
- Debugging
- Logging
- Analytics
- Progress tracking

---

### Terminal Operations (Collectors)

Terminal operations execute the pipeline and return a result.

#### `toArray(source)`

Collects all results into an array.

```typescript
toArray(source: Array<T>): Array<U>
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .map(x => x * 2);

const result = pipeline.toArray([1, 2, 3]); // [2, 4, 6]
```

---

#### `reduce(source, reducer, initial)`

Custom reduction with a reducer function.

```typescript
reduce(source: Array<T>, 
       reducer: (acc: A, value: U) => A, 
       initial: A): A
```

**Example:**
```javascript
const pipeline = new Pipeline()
  .map(x => x * 2);

const sum = pipeline.reduce(
  [1, 2, 3, 4],
  (acc, x) => acc + x,
  0
);
console.log(sum); // 20
```

**Use cases:**
- Custom aggregations
- Building objects from arrays
- Complex reductions

---

### Multi-Input Operations

These standalone functions work with multiple arrays. They don't use the Pipeline API.

#### `takeLast(array, n)`

Takes the last N elements from an array.

```typescript
takeLast(source: Array<T>, n: number): Array<T>
```

**Example:**
```javascript
import { takeLast } from 'orlando-transducers';

const result = takeLast([1, 2, 3, 4, 5], 3);
// result: [3, 4, 5]
```

**Use cases:**
- Get recent items (last N logs, events, etc.)
- Tail of a sequence
- "Show more" from end

**Note:** Unlike `take()`, this requires processing the entire array since it needs to know which elements are last.

---

#### `dropLast(array, n)`

Drops the last N elements from an array.

```typescript
dropLast(source: Array<T>, n: number): Array<T>
```

**Example:**
```javascript
import { dropLast } from 'orlando-transducers';

const result = dropLast([1, 2, 3, 4, 5], 2);
// result: [1, 2, 3]
```

**Use cases:**
- Remove trailing elements
- Trim recent history
- Keep all except last N

---

#### `aperture(array, size)`

Creates sliding windows of a given size.

```typescript
aperture(source: Array<T>, size: number): Array<Array<T>>
```

**Example:**
```javascript
import { aperture } from 'orlando-transducers';

const data = [1, 2, 3, 4, 5];
const windows = aperture(data, 3);
// windows: [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
```

**Use cases:**
- Moving averages
- N-gram analysis
- Sliding window algorithms
- Comparing adjacent elements

**Example - Moving average:**
```javascript
const numbers = [10, 20, 30, 40, 50];
const windows = aperture(numbers, 3);
const averages = windows.map(w => w.reduce((a, b) => a + b) / w.length);
// averages: [20, 30, 40]
```

---

#### `merge(arrays)`

Interleaves elements from multiple arrays in round-robin fashion.

```typescript
merge(arrays: Array<Array<T>>): Array<T>
```

**Example:**
```javascript
import { merge } from 'orlando-transducers';

const a = [1, 2, 3];
const b = [4, 5, 6];
const result = merge([a, b]);
// result: [1, 4, 2, 5, 3, 6]
```

**Use cases:**
- Interleaving data streams
- Round-robin scheduling
- Combining event logs chronologically

---

#### `intersection(arrayA, arrayB)`

Returns elements that appear in both arrays.

```typescript
intersection(a: Array<T>, b: Array<T>): Array<T>
```

**Example:**
```javascript
import { intersection } from 'orlando-transducers';

const a = [1, 2, 3, 4];
const b = [3, 4, 5, 6];
const common = intersection(a, b);
// common: [3, 4]
```

**Use cases:**
- Finding common elements
- Set operations
- Filtering by membership

---

#### `difference(arrayA, arrayB)`

Returns elements in A that are not in B.

```typescript
difference(a: Array<T>, b: Array<T>): Array<T>
```

**Example:**
```javascript
import { difference } from 'orlando-transducers';

const a = [1, 2, 3, 4];
const b = [3, 4, 5, 6];
const uniqueToA = difference(a, b);
// uniqueToA: [1, 2]
```

**Use cases:**
- Finding new/removed items
- Exclusion lists
- Diff operations

---

#### `union(arrayA, arrayB)`

Returns all unique elements from both arrays.

```typescript
union(a: Array<T>, b: Array<T>): Array<T>
```

**Example:**
```javascript
import { union } from 'orlando-transducers';

const a = [1, 2, 3];
const b = [3, 4, 5];
const allUnique = union(a, b);
// allUnique: [1, 2, 3, 4, 5]
```

**Use cases:**
- Combining datasets
- Merging unique items
- Set union

---

#### `symmetricDifference(arrayA, arrayB)`

Returns elements in either array but not both.

```typescript
symmetricDifference(a: Array<T>, b: Array<T>): Array<T>
```

**Example:**
```javascript
import { symmetricDifference } from 'orlando-transducers';

const a = [1, 2, 3, 4];
const b = [3, 4, 5, 6];
const unique = symmetricDifference(a, b);
// unique: [1, 2, 5, 6]
```

**Use cases:**
- Finding differences
- XOR operations
- Change detection

---

### Logic Functions (Phase 3)

Predicate combinators for cleaner conditional logic.

#### `both(pred1, pred2)`

Combines two predicates with AND logic.

```typescript
both(p1: (value: T) => boolean, p2: (value: T) => boolean): (value: T) => boolean
```

**Example:**
```javascript
import { both } from 'orlando-transducers';

const isPositive = x => x > 0;
const isEven = x => x % 2 === 0;
const isPositiveEven = both(isPositive, isEven);

const result = [1, 2, 3, 4, -2].filter(isPositiveEven);
// result: [2, 4]
```

---

#### `either(pred1, pred2)`

Combines two predicates with OR logic.

```typescript
either(p1: (value: T) => boolean, p2: (value: T) => boolean): (value: T) => boolean
```

**Example:**
```javascript
import { either } from 'orlando-transducers';

const isSmall = x => x < 10;
const isLarge = x => x > 100;
const isExtreme = either(isSmall, isLarge);

const result = [5, 50, 105].filter(isExtreme);
// result: [5, 105]
```

---

#### `complement(predicate)`

Negates a predicate.

```typescript
complement(pred: (value: T) => boolean): (value: T) => boolean
```

**Example:**
```javascript
import { complement } from 'orlando-transducers';

const isEven = x => x % 2 === 0;
const isOdd = complement(isEven);

const result = [1, 2, 3, 4, 5].filter(isOdd);
// result: [1, 3, 5]
```

---

#### `allPass(predicates)`

Returns true if ALL predicates pass.

```typescript
allPass(predicates: Array<(value: T) => boolean>): (value: T) => boolean
```

**Example:**
```javascript
import { allPass } from 'orlando-transducers';

const isValid = allPass([
  user => user.age >= 18,
  user => user.email.includes('@'),
  user => user.verified
]);

const validUsers = users.filter(isValid);
```

---

#### `anyPass(predicates)`

Returns true if ANY predicate passes.

```typescript
anyPass(predicates: Array<(value: T) => boolean>): (value: T) => boolean
```

**Example:**
```javascript
import { anyPass } from 'orlando-transducers';

const hasDiscount = anyPass([
  user => user.isPremium,
  user => user.isStudent,
  user => user.couponCode
]);

const discountedUsers = users.filter(hasDiscount);
```

---

## Common Patterns

### Pagination

```javascript
function paginate(data, page, pageSize) {
  return new Pipeline()
    .drop(page * pageSize)
    .take(pageSize)
    .toArray(data);
}

const page2 = paginate([1,2,3,4,5,6,7,8,9,10], 1, 3);
// [4, 5, 6]
```

### Data Transformation Pipeline

```javascript
const processUsers = new Pipeline()
  .filter(user => user.active)
  .map(user => ({
    id: user.id,
    name: user.fullName,
    email: user.email.toLowerCase()
  }))
  .filter(user => user.email.endsWith('@company.com'))
  .take(100);

const activeCompanyUsers = processUsers.toArray(users);
```

### Find First Matching

```javascript
const findFirst = new Pipeline()
  .filter(x => x > 100)
  .take(1);

const result = findFirst.toArray(data);
const firstMatch = result[0]; // or undefined
```

### Debugging Pipeline

```javascript
const debugPipeline = new Pipeline()
  .tap(x => console.log('Input:', x))
  .map(x => x * 2)
  .tap(x => console.log('After map:', x))
  .filter(x => x > 10)
  .tap(x => console.log('After filter:', x));
```

### Combining Multiple Operations

```javascript
const complexPipeline = new Pipeline()
  .map(x => x.trim())                    // clean whitespace
  .filter(x => x.length > 0)             // remove empty
  .map(x => x.toLowerCase())             // normalize
  .filter(x => !x.startsWith('#'))       // remove comments
  .map(x => x.split('='))                // parse key=value
  .filter(([k, v]) => k && v)            // validate pairs
  .map(([k, v]) => ({ [k]: v }));        // to objects

const config = complexPipeline.toArray(lines);
```

## TypeScript Support

Orlando automatically generates TypeScript definitions. Import with full type safety:

```typescript
import init, { Pipeline } from 'orlando-transducers';

await init();

interface User {
  id: number;
  name: string;
  email: string;
  active: boolean;
}

interface UserDTO {
  id: number;
  displayName: string;
}

const pipeline = new Pipeline()
  .filter((user: User) => user.active)
  .map((user: User): UserDTO => ({
    id: user.id,
    displayName: user.name
  }))
  .take(10);

const users: User[] = [/* ... */];
const dtos: UserDTO[] = pipeline.toArray(users);
```

## Performance Tips

### 1. Use Early Termination

```javascript
// ❌ Processes all 1 million items
const bad = data
  .map(expensiveOperation)
  .slice(0, 10);

// ✅ Stops after 10 items
const good = new Pipeline()
  .map(expensiveOperation)
  .take(10)
  .toArray(data);
```

### 2. Filter Early

```javascript
// ❌ Maps all items, then filters
const bad = new Pipeline()
  .map(expensiveOperation)
  .filter(x => x.isValid);

// ✅ Filters first, then maps fewer items
const good = new Pipeline()
  .filter(x => x.isValid)
  .map(expensiveOperation);
```

### 3. Reuse Pipelines

```javascript
// Define once
const userProcessor = new Pipeline()
  .filter(user => user.active)
  .map(user => user.email);

// Reuse multiple times
const emails1 = userProcessor.toArray(users1);
const emails2 = userProcessor.toArray(users2);
```

### 4. Avoid Unnecessary Operations

```javascript
// ❌ Multiple passes
const bad = data
  .map(x => x * 2)
  .map(x => x + 1);

// ✅ Single pass
const good = new Pipeline()
  .map(x => (x * 2) + 1)
  .toArray(data);
```

## Browser Compatibility

Orlando uses WebAssembly and works in all modern browsers:

- ✅ Chrome 57+
- ✅ Firefox 52+
- ✅ Safari 11+
- ✅ Edge 16+

For older browsers, include a WASM polyfill.

## Error Handling

```javascript
try {
  await init(); // Initialize WASM
  
  const pipeline = new Pipeline()
    .map(x => {
      if (typeof x !== 'number') {
        throw new Error(`Expected number, got ${typeof x}`);
      }
      return x * 2;
    });
  
  const result = pipeline.toArray(data);
} catch (error) {
  console.error('Pipeline error:', error);
}
```

## Examples Repository

See the `/examples` directory for complete working examples:

- `examples/basic.html` - Basic usage
- `examples/pagination.html` - Pagination example
- `examples/data-processing.html` - Real-world data processing
- `examples/performance.html` - Performance comparison
- `examples/typescript/` - TypeScript examples

## Multi-Input Operations

Orlando provides powerful multi-input operations for combining and comparing arrays. These are standalone functions (not Pipeline methods) that enable hybrid composition patterns.

### `merge(arrays)`

Merges multiple arrays by interleaving their elements in round-robin fashion.

```typescript
merge(arrays: Array<Array<T>>): Array<T>
```

**Example:**
```javascript
import { merge } from 'orlando-transducers';

const a = [1, 2, 3];
const b = [4, 5, 6];
const c = [7, 8, 9];

const result = merge([a, b, c]);
// result: [1, 4, 7, 2, 5, 8, 3, 6, 9]
```

**Handles different lengths:**
```javascript
const a = [1, 2];
const b = [3, 4, 5, 6];
const result = merge([a, b]);
// result: [1, 3, 2, 4, 5, 6]
```

**Use cases:**
- Round-robin scheduling
- Interleaving data from multiple sources
- Creating alternating patterns

**Hybrid Composition Example:**
```javascript
// Process each stream differently, then merge
const pipeline1 = new Pipeline().map(x => x * 2);
const pipeline2 = new Pipeline().map(x => x + 10);

const stream1 = pipeline1.toArray([1, 2, 3]);
const stream2 = pipeline2.toArray([1, 2, 3]);

const merged = merge([stream1, stream2]);
// merged: [2, 11, 4, 12, 6, 13]
```

---

### `intersection(arrayA, arrayB)`

Returns elements that appear in both arrays.

```typescript
intersection(arrayA: Array<T>, arrayB: Array<T>): Array<T>
```

**Example:**
```javascript
import { intersection } from 'orlando-transducers';

const a = [1, 2, 3, 4, 5];
const b = [3, 4, 5, 6, 7];

const common = intersection(a, b);
// common: [3, 4, 5]
```

**Preserves order from first array:**
```javascript
const a = [5, 3, 4, 1];
const b = [1, 3, 5];
const result = intersection(a, b);
// result: [5, 3, 1] (order from a)
```

**Use cases:**
- Finding matching records across datasets
- Filtering by membership
- Database-style joins

---

### `difference(arrayA, arrayB)`

Returns elements in the first array but not in the second.

```typescript
difference(arrayA: Array<T>, arrayB: Array<T>): Array<T>
```

**Example:**
```javascript
import { difference } from 'orlando-transducers';

const a = [1, 2, 3, 4, 5];
const b = [3, 4, 5, 6, 7];

const uniqueToA = difference(a, b);
// uniqueToA: [1, 2]
```

**Use cases:**
- Finding new/deleted items
- Exclusion lists
- Data reconciliation

---

### `union(arrayA, arrayB)`

Returns all unique elements from both arrays.

```typescript
union(arrayA: Array<T>, arrayB: Array<T>): Array<T>
```

**Example:**
```javascript
import { union } from 'orlando-transducers';

const a = [1, 2, 3];
const b = [3, 4, 5];

const allUnique = union(a, b);
// allUnique: [1, 2, 3, 4, 5]
```

**Removes duplicates:**
```javascript
const a = [1, 2, 2, 3];
const b = [3, 4, 4, 5];
const result = union(a, b);
// result: [1, 2, 3, 4, 5]
```

**Use cases:**
- Combining datasets without duplicates
- Creating master lists
- Deduplication across sources

---

### `symmetricDifference(arrayA, arrayB)`

Returns elements that appear in exactly one array (not both).

```typescript
symmetricDifference(arrayA: Array<T>, arrayB: Array<T>): Array<T>
```

**Example:**
```javascript
import { symmetricDifference } from 'orlando-transducers';

const a = [1, 2, 3, 4];
const b = [3, 4, 5, 6];

const uniqueToEach = symmetricDifference(a, b);
// uniqueToEach: [1, 2, 5, 6]
```

**No overlap:**
```javascript
const a = [1, 2];
const b = [3, 4];
const result = symmetricDifference(a, b);
// result: [1, 2, 3, 4]
```

**Use cases:**
- Finding changed items
- XOR operations
- Detecting differences between versions

---

## Hybrid Composition Patterns

Combine transducers with multi-input operations for maximum flexibility.

### Pattern 1: Process → Combine

Process streams independently, then combine:

```javascript
const pipeline = new Pipeline()
  .filter(x => x > 0)
  .map(x => x * 2);

const stream1 = pipeline.toArray(data1);
const stream2 = pipeline.toArray(data2);

const combined = intersection(stream1, stream2);
```

### Pattern 2: Combine → Process

Combine first, then process:

```javascript
const merged = merge([data1, data2, data3]);

const pipeline = new Pipeline()
  .filter(x => x % 2 === 0)
  .take(10);

const result = pipeline.toArray(merged);
```

### Real-World Example: Finding Common Active Users

```javascript
// Get active users from both datasets
const activeInA = new Pipeline()
  .filter(user => user.active)
  .map(user => user.id)
  .toArray(usersA);

const activeInB = new Pipeline()
  .filter(user => user.active)
  .map(user => user.id)
  .toArray(usersB);

// Find users active in both systems
const activeInBoth = intersection(activeInA, activeInB);
```

For more patterns and examples, see the [Hybrid Composition Guide](../HYBRID_COMPOSITION.md).

---

## Advanced Collectors

Orlando provides specialized collector functions for complex data analysis and aggregation.

### `frequencies(array)`

Counts occurrences of each element in the array.

```typescript
frequencies(array: Array<T>): Map<T, number>
```

**Example:**
```javascript
import { frequencies } from 'orlando-transducers';

const data = ['apple', 'banana', 'apple', 'cherry', 'banana', 'apple'];
const counts = frequencies(data);

// counts: Map {
//   'apple' => 3,
//   'banana' => 2,
//   'cherry' => 1
// }
```

**With pipeline:**
```javascript
const pipeline = new Pipeline()
  .filter(word => word.length > 5)
  .map(word => word.toLowerCase());

const words = pipeline.toArray(text.split(' '));
const wordCounts = frequencies(words);
```

**Use cases:**
- Word frequency analysis
- Event counting
- Distribution analysis
- Histogram generation

---

### `partitionBy(array, keyFn)`

Splits array into consecutive groups where keyFn returns the same value.

```typescript
partitionBy<T, K>(array: Array<T>, keyFn: (value: T) => K): Array<Array<T>>
```

**Example:**
```javascript
import { partitionBy } from 'orlando-transducers';

const numbers = [1, 1, 2, 3, 3, 3, 4, 5, 5];
const groups = partitionBy(numbers, x => x);
// groups: [[1, 1], [2], [3, 3, 3], [4], [5, 5]]

const data = [
  { type: 'A', value: 1 },
  { type: 'A', value: 2 },
  { type: 'B', value: 3 },
  { type: 'B', value: 4 }
];
const byType = partitionBy(data, item => item.type);
// byType: [[{type:'A', value:1}, {type:'A', value:2}],
//          [{type:'B', value:3}, {type:'B', value:4}]]
```

**Use cases:**
- Grouping consecutive similar items
- Run-length encoding
- Chunking by property changes
- Log file analysis

---

### `topK(array, k, [compareFn])`

Returns the k largest elements (maintains relative order).

```typescript
topK<T>(array: Array<T>, k: number, compareFn?: (a: T, b: T) => number): Array<T>
```

**Example:**
```javascript
import { topK } from 'orlando-transducers';

const scores = [85, 92, 78, 95, 88, 72, 99, 81];
const topThree = topK(scores, 3);
// topThree: [99, 95, 92]

// Custom comparison
const users = [
  { name: 'Alice', score: 85 },
  { name: 'Bob', score: 92 },
  { name: 'Charlie', score: 88 }
];
const topUsers = topK(users, 2, (a, b) => a.score - b.score);
// topUsers: [{name: 'Bob', score: 92}, {name: 'Charlie', score: 88}]
```

**Use cases:**
- Leaderboards
- Top performers
- High scores
- Best matches

---

### `reservoirSample(array, k)`

Random sampling with uniform probability (reservoir sampling algorithm).

```typescript
reservoirSample<T>(array: Array<T>, k: number): Array<T>
```

**Example:**
```javascript
import { reservoirSample } from 'orlando-transducers';

const largeDataset = Array.from({ length: 10000 }, (_, i) => i);
const sample = reservoirSample(largeDataset, 100);
// sample: 100 randomly selected items with uniform probability
```

**Use cases:**
- Statistical sampling
- Random selection from large datasets
- A/B testing
- Data subset creation

---

### `cartesianProduct(arrayA, arrayB)`

Returns all possible pairs from two arrays.

```typescript
cartesianProduct<T, U>(arrayA: Array<T>, arrayB: Array<U>): Array<[T, U]>
```

**Example:**
```javascript
import { cartesianProduct } from 'orlando-transducers';

const colors = ['red', 'blue'];
const sizes = ['S', 'M', 'L'];

const combinations = cartesianProduct(colors, sizes);
// combinations: [
//   ['red', 'S'], ['red', 'M'], ['red', 'L'],
//   ['blue', 'S'], ['blue', 'M'], ['blue', 'L']
// ]
```

**Use cases:**
- Product variant generation
- Combinatorial analysis
- Test case generation
- Grid coordinates

---

### `zipLongest(arrayA, arrayB, [fillValue])`

Like zip, but continues until the longer array is exhausted, filling missing values.

```typescript
zipLongest<T, U>(arrayA: Array<T>, arrayB: Array<U>, fillValue?: any): Array<[T | any, U | any]>
```

**Example:**
```javascript
import { zipLongest } from 'orlando-transducers';

const a = [1, 2, 3];
const b = ['a', 'b'];

const result = zipLongest(a, b, null);
// result: [[1, 'a'], [2, 'b'], [3, null]]

const result2 = zipLongest(a, b, undefined);
// result2: [[1, 'a'], [2, 'b'], [3, undefined]]
```

**Use cases:**
- Handling arrays of different lengths
- Data alignment
- Missing value handling
- Table formatting

---

## Statistical Operations

Orlando provides efficient statistical analysis operations for numeric data.

### `product(array)`

Multiplies all numbers in an array.

```typescript
product(array: Array<number>): number
```

**Example:**
```javascript
import { product } from 'orlando-transducers';

const numbers = [2, 3, 4];
const result = product(numbers);
// result: 24

const pipeline = new Pipeline().filter(x => x > 0);
const filtered = pipeline.toArray([1, -2, 3, 4]);
const prod = product(filtered);
// prod: 12
```

**Use cases:**
- Mathematical calculations
- Compound growth rates
- Probability calculations

---

### `mean(array)`

Calculates the arithmetic mean (average) of an array.

```typescript
mean(array: Array<number>): number | undefined
```

**Example:**
```javascript
import { mean } from 'orlando-transducers';

const scores = [85, 92, 78, 95, 88];
const average = mean(scores);
// average: 87.6

// Returns undefined for empty arrays
mean([]); // undefined
```

**Use cases:**
- Performance metrics
- Grade calculations
- Statistical analysis

---

### `median(array)`

Finds the median (middle value) of an array.

```typescript
median(array: Array<number>): number | undefined
```

**Example:**
```javascript
import { median } from 'orlando-transducers';

const odd = [1, 3, 5, 7, 9];
median(odd); // 5

const even = [1, 2, 3, 4];
median(even); // 2.5

median([]); // undefined
```

**Use cases:**
- Robust averaging (less affected by outliers)
- Salary distributions
- Performance baselines

---

### `variance(array)`

Calculates the sample variance.

```typescript
variance(array: Array<number>): number | undefined
```

**Example:**
```javascript
import { variance } from 'orlando-transducers';

const data = [2, 4, 6, 8, 10];
const v = variance(data);
// v: 10.0

variance([5]); // undefined (need at least 2 values)
```

**Use cases:**
- Measuring data spread
- Quality control
- Risk assessment

---

### `stdDev(array)`

Calculates the standard deviation (square root of variance).

```typescript
stdDev(array: Array<number>): number | undefined
```

**Example:**
```javascript
import { stdDev } from 'orlando-transducers';

const data = [2, 4, 6, 8, 10];
const sd = stdDev(data);
// sd: 3.16...

// Useful for measuring consistency
const player1Scores = [50, 52, 48, 51, 49];
const player2Scores = [30, 70, 20, 80, 10];
stdDev(player1Scores); // ~1.58 (consistent)
stdDev(player2Scores); // ~27.39 (variable)
```

**Use cases:**
- Data consistency measurement
- Outlier detection
- Statistical analysis

---

### `min(array)` / `max(array)`

Finds the minimum or maximum value.

```typescript
min(array: Array<number>): number | undefined
max(array: Array<number>): number | undefined
```

**Example:**
```javascript
import { min, max } from 'orlando-transducers';

const scores = [85, 92, 78, 95, 88];
min(scores); // 78
max(scores); // 95

min([]); // undefined
```

---

### `minBy(array, keyFn)` / `maxBy(array, keyFn)`

Finds the element with minimum or maximum key value.

```typescript
minBy<T>(array: Array<T>, keyFn: (value: T) => number): T | undefined
maxBy<T>(array: Array<T>, keyFn: (value: T) => number): T | undefined
```

**Example:**
```javascript
import { minBy, maxBy } from 'orlando-transducers';

const users = [
  { name: 'Alice', score: 85 },
  { name: 'Bob', score: 92 },
  { name: 'Charlie', score: 78 }
];

const lowest = minBy(users, u => u.score);
// lowest: { name: 'Charlie', score: 78 }

const highest = maxBy(users, u => u.score);
// highest: { name: 'Bob', score: 92 }
```

**Use cases:**
- Finding extremes in object arrays
- Best/worst performer
- Price comparisons

---

### `quantile(array, p)`

Calculates the p-th quantile (0 ≤ p ≤ 1) using linear interpolation.

```typescript
quantile(array: Array<number>, p: number): number | undefined
```

**Example:**
```javascript
import { quantile } from 'orlando-transducers';

const data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

quantile(data, 0.25); // First quartile (Q1): 3.25
quantile(data, 0.5);  // Median (Q2): 5.5
quantile(data, 0.75); // Third quartile (Q3): 7.75
quantile(data, 0.95); // 95th percentile: 9.55

quantile(data, 1.5); // undefined (p out of range)
```

**Use cases:**
- Percentile calculations
- Performance SLAs (p95, p99)
- Outlier detection

---

### `mode(array)`

Finds the most frequently occurring value.

```typescript
mode(array: Array<number>): number | undefined
```

**Example:**
```javascript
import { mode } from 'orlando-transducers';

const data = [1, 2, 2, 3, 3, 3, 4];
mode(data); // 3 (appears most often)

const tie = [1, 1, 2, 2];
mode(tie); // 1 (returns first mode if tied)

mode([]); // undefined
```

**Use cases:**
- Finding common values
- Survey analysis
- Pattern detection

---

## Collection Utilities

Non-streaming utility operations for sorting, reversing, and generating sequences.

### `sortBy(array, keyFn)`

Sorts elements by the result of a key function.

```typescript
sortBy<T, K>(array: Array<T>, keyFn: (value: T) => K): Array<T>
```

**Example:**
```javascript
import { sortBy } from 'orlando-transducers';

const users = [
  { name: 'Charlie', age: 30 },
  { name: 'Alice', age: 25 },
  { name: 'Bob', age: 35 }
];

const byAge = sortBy(users, u => u.age);
// [{ name: 'Alice', age: 25 }, { name: 'Charlie', age: 30 }, { name: 'Bob', age: 35 }]

const byName = sortBy(users, u => u.name);
// [{ name: 'Alice', ... }, { name: 'Bob', ... }, { name: 'Charlie', ... }]
```

**Use cases:**
- Sorting objects by property
- Custom ordering
- Normalized sorting

---

### `sortWith(array, compareFn)`

Sorts with a custom comparator function.

```typescript
sortWith<T>(array: Array<T>, compareFn: (a: T, b: T) => number): Array<T>
```

**Example:**
```javascript
import { sortWith } from 'orlando-transducers';

const numbers = [3, 1, 4, 1, 5];

// Ascending
const asc = sortWith(numbers, (a, b) => a - b);
// [1, 1, 3, 4, 5]

// Descending
const desc = sortWith(numbers, (a, b) => b - a);
// [5, 4, 3, 1, 1]

// Complex comparison
const items = [
  { priority: 1, name: 'b' },
  { priority: 2, name: 'a' },
  { priority: 1, name: 'a' }
];
const sorted = sortWith(items, (a, b) => {
  if (a.priority !== b.priority) return a.priority - b.priority;
  return a.name.localeCompare(b.name);
});
```

**Use cases:**
- Multi-level sorting
- Custom ordering logic
- Complex comparisons

---

### `reverse(array)`

Reverses the order of elements.

```typescript
reverse<T>(array: Array<T>): Array<T>
```

**Example:**
```javascript
import { reverse } from 'orlando-transducers';

const data = [1, 2, 3, 4, 5];
const reversed = reverse(data);
// reversed: [5, 4, 3, 2, 1]

// With pipeline
const pipeline = new Pipeline().filter(x => x % 2 === 0);
const evens = pipeline.toArray([1, 2, 3, 4, 5, 6]);
const reversedEvens = reverse(evens);
// reversedEvens: [6, 4, 2]
```

**Use cases:**
- Reversing order
- Last-to-first processing
- Stack operations

---

### `range(start, end, step)`

Generates a numeric sequence from start to end (exclusive) with a given step.

```typescript
range(start: number, end: number, step: number): Array<number>
```

**Example:**
```javascript
import { range } from 'orlando-transducers';

range(0, 10, 1);    // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
range(0, 10, 2);    // [0, 2, 4, 6, 8]
range(10, 0, -1);   // [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]
range(0, 1, 0.1);   // [0, 0.1, 0.2, ..., 0.9]

// Use with pipeline
const pipeline = new Pipeline().filter(x => x % 3 === 0);
const divisibleBy3 = pipeline.toArray(range(0, 30, 1));
// [0, 3, 6, 9, 12, 15, 18, 21, 24, 27]
```

**Use cases:**
- Generating sequences
- Loop replacements
- Index generation

---

### `repeat(value, n)`

Repeats a value N times.

```typescript
repeat<T>(value: T, n: number): Array<T>
```

**Example:**
```javascript
import { repeat } from 'orlando-transducers';

repeat(0, 5);           // [0, 0, 0, 0, 0]
repeat('x', 3);         // ['x', 'x', 'x']
repeat([1, 2], 2);      // [[1, 2], [1, 2]]

// Initialize array with default values
const defaultScores = repeat(0, 10);
```

**Use cases:**
- Array initialization
- Padding
- Default values

---

### `cycle(array, n)`

Repeats an entire array N times.

```typescript
cycle<T>(array: Array<T>, n: number): Array<T>
```

**Example:**
```javascript
import { cycle } from 'orlando-transducers';

cycle([1, 2, 3], 2);         // [1, 2, 3, 1, 2, 3]
cycle(['a', 'b'], 3);        // ['a', 'b', 'a', 'b', 'a', 'b']

// Create repeating pattern
const colors = cycle(['red', 'blue', 'green'], 10);
// ['red', 'blue', 'green', 'red', 'blue', 'green', ...]
```

**Use cases:**
- Repeating patterns
- Round-robin scheduling
- Cyclic data

---

### `unfold(seed, fn, limit)`

Generates a sequence by repeatedly applying a function to a seed value.

```typescript
unfold<T>(seed: T, fn: (value: T) => T | undefined, limit: number): Array<T>
```

**Example:**
```javascript
import { unfold } from 'orlando-transducers';

// Fibonacci sequence
const fib = unfold(
  [0, 1],
  ([a, b]) => [b, a + b],
  10
);
// [[0, 1], [1, 1], [1, 2], [2, 3], [3, 5], [5, 8], ...]

// Powers of 2
const powersOf2 = unfold(1, x => x * 2, 8);
// [2, 4, 8, 16, 32, 64, 128, 256]

// Stops when function returns undefined
const countdown = unfold(5, x => x > 0 ? x - 1 : undefined, 10);
// [4, 3, 2, 1, 0]
```

**Use cases:**
- Generating sequences
- Recursive patterns
- Mathematical series

---

## Path Operations (JavaScript-Specific)

Safe navigation and transformation of nested objects.

### `path(obj, pathArray)`

Safely accesses nested properties using a path array.

```typescript
path(obj: Object, pathArray: Array<string>): any
```

**Example:**
```javascript
import { path } from 'orlando-transducers';

const user = {
  profile: {
    contact: {
      email: 'user@example.com'
    }
  }
};

path(user, ['profile', 'contact', 'email']);
// 'user@example.com'

path(user, ['profile', 'contact', 'phone']);
// undefined (safe - doesn't throw)

path(user, ['nonexistent', 'path']);
// undefined
```

**Use cases:**
- Safe property access
- Deep object navigation
- Optional chaining alternative

---

### `pathOr(obj, pathArray, defaultValue)`

Like `path`, but returns a default value if the path doesn't exist.

```typescript
pathOr(obj: Object, pathArray: Array<string>, defaultValue: any): any
```

**Example:**
```javascript
import { pathOr } from 'orlando-transducers';

const config = {
  server: {
    port: 3000
  }
};

pathOr(config, ['server', 'port'], 8080);
// 3000

pathOr(config, ['server', 'host'], 'localhost');
// 'localhost' (default)

pathOr(config, ['database', 'url'], 'mongodb://localhost');
// 'mongodb://localhost' (default)
```

**Use cases:**
- Configuration with defaults
- Fallback values
- Safe data access

---

### `evolve(obj, transformations)`

Applies transformations to nested object properties immutably.

```typescript
evolve(obj: Object, transformations: Object): Object
```

**Example:**
```javascript
import { evolve } from 'orlando-transducers';

const user = {
  name: 'john',
  age: 30,
  contact: {
    email: 'JOHN@EXAMPLE.COM'
  }
};

const normalized = evolve(user, {
  name: (s) => s.toUpperCase(),
  age: (n) => n + 1,
  contact: {
    email: (s) => s.toLowerCase()
  }
});

// normalized: {
//   name: 'JOHN',
//   age: 31,
//   contact: { email: 'john@example.com' }
// }

// Original is unchanged (immutable)
console.log(user.name); // 'john'
```

**Use cases:**
- Data normalization
- Immutable updates
- Nested transformations
- API response formatting

---

## Logic Functions

Orlando provides predicate combinators and conditional transducers for cleaner, more declarative conditional logic in pipelines.

### Predicate Combinators

Predicate combinators allow you to build complex predicates from simple building blocks.

#### `both(pred1, pred2)`

Combines two predicates with AND logic.

```typescript
both<T>(pred1: (value: T) => boolean, pred2: (value: T) => boolean): (value: T) => boolean
```

**Example:**
```javascript
import { both } from 'orlando-transducers';

const isPositive = x => x > 0;
const isEven = x => x % 2 === 0;
const isPositiveEven = both(isPositive, isEven);

const pipeline = new Pipeline()
  .filter(isPositiveEven);

pipeline.toArray([-2, -1, 0, 1, 2, 3, 4]); // [2, 4]
```

**Use cases:**
- Combining multiple conditions
- Complex validation rules
- Multi-criteria filtering

---

#### `either(pred1, pred2)`

Combines two predicates with OR logic.

```typescript
either<T>(pred1: (value: T) => boolean, pred2: (value: T) => boolean): (value: T) => boolean
```

**Example:**
```javascript
import { either } from 'orlando-transducers';

const isSmall = x => x < 10;
const isLarge = x => x > 100;
const isExtreme = either(isSmall, isLarge);

const pipeline = new Pipeline()
  .filter(isExtreme);

pipeline.toArray([5, 50, 150]); // [5, 150]
```

---

#### `complement(predicate)`

Negates a predicate (returns the opposite).

```typescript
complement<T>(pred: (value: T) => boolean): (value: T) => boolean
```

**Example:**
```javascript
import { complement } from 'orlando-transducers';

const isEven = x => x % 2 === 0;
const isOdd = complement(isEven);

const pipeline = new Pipeline()
  .filter(isOdd);

pipeline.toArray([1, 2, 3, 4, 5]); // [1, 3, 5]
```

---

#### `allPass(predicates)`

Returns true only if ALL predicates pass (short-circuits on first false).

```typescript
allPass<T>(predicates: Array<(value: T) => boolean>): (value: T) => boolean
```

**Example:**
```javascript
import { allPass } from 'orlando-transducers';

const validUser = allPass([
  user => user.age >= 18,
  user => user.email.includes('@'),
  user => user.verified === true,
  user => user.active === true
]);

const pipeline = new Pipeline()
  .filter(validUser);

const valid = pipeline.toArray(users);
```

**Use cases:**
- Multi-criteria validation
- Complex rule checking
- Access control
- Data quality checks

---

#### `anyPass(predicates)`

Returns true if ANY predicate passes (short-circuits on first true).

```typescript
anyPass<T>(predicates: Array<(value: T) => boolean>): (value: T) => boolean
```

**Example:**
```javascript
import { anyPass } from 'orlando-transducers';

const isSpecial = anyPass([
  x => x === 0,
  x => x % 10 === 0,
  x => x > 1000
]);

const pipeline = new Pipeline()
  .filter(isSpecial);

pipeline.toArray([0, 5, 50, 1500, 7]); // [0, 50, 1500]
```

---

### Conditional Transducers

Conditional transducers apply transformations based on predicates, allowing you to branch logic within a pipeline.

#### `When(predicate, transform)`

Applies transform only when predicate is true. Otherwise, value passes through unchanged.

**Note:** When is a transducer class that needs to be instantiated and used with collectors.

**Example:**
```javascript
import { When, toArray } from 'orlando-transducers';

const doubleIfPositive = new When(
  x => x > 0,
  x => x * 2
);

const result = toArray(doubleIfPositive, [-1, 2, -3, 4]);
// result: [-1, 4, -3, 8]
```

**Use cases:**
- Conditional normalization
- Selective transformation
- Data correction
- Format conversion

---

#### `Unless(predicate, transform)`

Applies transform only when predicate is false. Inverse of When.

**Example:**
```javascript
import { Unless, toArray } from 'orlando-transducers';

const zeroIfNegative = new Unless(
  x => x > 0,
  _ => 0
);

const result = toArray(zeroIfNegative, [-1, 2, -3, 4]);
// result: [0, 2, 0, 4]
```

---

#### `IfElse(predicate, onTrue, onFalse)`

Branches on condition - applies different transforms based on predicate.

**Example:**
```javascript
import { IfElse, toArray } from 'orlando-transducers';

const normalize = new IfElse(
  x => x >= 0,
  x => x * 2,      // double if positive
  x => x / 2       // halve if negative
);

const result = toArray(normalize, [-4, 3, -6, 5]);
// result: [-2, 6, -3, 10]
```

**Use cases:**
- Two-way data transformation
- A/B processing
- Type-based handling
- Status-dependent logic

---

### Composing Logic Functions

Logic functions compose beautifully for complex conditional logic:

```javascript
import { both, either, complement, allPass, When } from 'orlando-transducers';

// Complex predicate composition
const isPositiveEven = both(x => x > 0, x => x % 2 === 0);
const isNegativeOdd = both(x => x < 0, complement(x => x % 2 === 0));
const isSpecial = either(isPositiveEven, isNegativeOdd);

const pipeline = new Pipeline()
  .filter(isSpecial)
  .map(x => x * 10);

// Nested logic with When
const complexTransform = new When(
  allPass([
    x => x > 0,
    x => x < 100,
    x => x % 2 === 0
  ]),
  x => x * 2
);
```

---

## Next Steps

- Check out the [Hybrid Composition Guide](../HYBRID_COMPOSITION.md) for combining transducers with multi-input operations
- See the [Migration Guide](./MIGRATION.md) for converting from array methods
- Read the [Main README](../../README.md) for project overview

---

**Questions?** [Open an issue](https://github.com/yourusername/orlando/issues) on GitHub.
