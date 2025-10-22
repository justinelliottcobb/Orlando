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

## Next Steps

- Check out the [Migration Guide](./MIGRATION.md) for converting from array methods
- See [Performance Benchmarks](./BENCHMARKS.md) for detailed comparisons
- Read the [Main README](../../README.md) for project overview

---

**Questions?** [Open an issue](https://github.com/yourusername/orlando/issues) on GitHub.
