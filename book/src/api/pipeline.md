# Pipeline (JavaScript API)

The `Pipeline` class is the main entry point for building transducer pipelines in JavaScript/TypeScript.

## Quick Start

```javascript
import init, { Pipeline } from 'orlando-transducers';
await init();

const pipeline = new Pipeline()
  .map(x => x * 2)
  .filter(x => x > 10)
  .take(5);

const result = pipeline.toArray(data);
```

## Transformation Methods

All transformation methods return a new `Pipeline` instance, allowing fluent method chaining.

### `map(fn)`

Transform each value using the provided function.

```typescript
map(fn: (value: T) => U): Pipeline
```

```javascript
new Pipeline()
  .map(x => x * 2)
  .map(x => x + 1)
  .toArray([1, 2, 3]); // [3, 5, 7]
```

### `filter(predicate)`

Keep only values that match the predicate.

```typescript
filter(predicate: (value: T) => boolean): Pipeline
```

```javascript
new Pipeline()
  .filter(x => x % 2 === 0)
  .filter(x => x > 10)
  .toArray([1, 5, 12, 20, 3]); // [12, 20]
```

### `take(n)`

Take the first `n` elements, then stop processing. This is where Orlando's early termination shines.

```typescript
take(n: number): Pipeline
```

```javascript
new Pipeline()
  .filter(x => x % 2 === 0)
  .take(3)
  .toArray([1, 2, 3, 4, 5, 6, 7, 8]); // [2, 4, 6]
```

### `takeWhile(predicate)`

Take elements while the predicate is true, then stop.

```typescript
takeWhile(predicate: (value: T) => boolean): Pipeline
```

```javascript
new Pipeline()
  .takeWhile(x => x < 100)
  .toArray([1, 5, 50, 200, 10]); // [1, 5, 50]
```

### `drop(n)`

Skip the first `n` elements.

```typescript
drop(n: number): Pipeline
```

```javascript
new Pipeline()
  .drop(3)
  .toArray([1, 2, 3, 4, 5]); // [4, 5]
```

### `dropWhile(predicate)`

Skip elements while the predicate is true.

```typescript
dropWhile(predicate: (value: T) => boolean): Pipeline
```

```javascript
new Pipeline()
  .dropWhile(x => x < 10)
  .toArray([1, 5, 12, 20, 3]); // [12, 20, 3]
```

### `flatMap(fn)`

Transform and flatten nested arrays.

```typescript
flatMap(fn: (value: T) => Array<U>): Pipeline
```

```javascript
new Pipeline()
  .flatMap(x => [x, x * 10])
  .toArray([1, 2, 3]); // [1, 10, 2, 20, 3, 30]
```

### `tap(fn)`

Perform side effects without modifying values. Useful for debugging.

```typescript
tap(fn: (value: T) => void): Pipeline
```

```javascript
new Pipeline()
  .tap(x => console.log('Processing:', x))
  .map(x => x * 2)
  .tap(x => console.log('Result:', x))
  .toArray([1, 2, 3]);
```

### `reject(predicate)`

Remove matching elements (inverse of `filter`).

```typescript
reject(predicate: (value: T) => boolean): Pipeline
```

```javascript
new Pipeline()
  .reject(x => x < 0)
  .toArray([-1, 2, -3, 4]); // [2, 4]
```

### `chunk(n)`

Group elements into arrays of size `n`.

```typescript
chunk(n: number): Pipeline
```

```javascript
new Pipeline()
  .chunk(3)
  .toArray([1, 2, 3, 4, 5, 6, 7]); // [[1,2,3], [4,5,6], [7]]
```

### `unique()`

Remove consecutive duplicate values.

```typescript
unique(): Pipeline
```

```javascript
new Pipeline()
  .unique()
  .toArray([1, 1, 2, 2, 3, 1]); // [1, 2, 3, 1]
```

### `scan(fn, initial)`

Accumulate values with intermediate results.

```typescript
scan(fn: (acc: A, value: T) => A, initial: A): Pipeline
```

```javascript
new Pipeline()
  .scan((sum, x) => sum + x, 0)
  .toArray([1, 2, 3, 4]); // [1, 3, 6, 10]
```

## Pipeline Enhancement Methods

### `pluck(key)`

Extract a single property from each object.

```javascript
new Pipeline()
  .pluck('name')
  .toArray([{ name: "Alice" }, { name: "Bob" }]); // ["Alice", "Bob"]
```

### `project(keys)`

Extract multiple properties from each object.

```javascript
new Pipeline()
  .project(['id', 'name'])
  .toArray(users); // [{ id: 1, name: "Alice" }, ...]
```

### `compact()`

Remove all falsy values (`null`, `undefined`, `false`, `0`, `''`, `NaN`).

```javascript
new Pipeline()
  .compact()
  .toArray([0, 1, null, 2, undefined, 3, '', 4]); // [1, 2, 3, 4]
```

### `flatten(depth)`

Flatten nested arrays to a given depth.

```javascript
new Pipeline()
  .flatten(2)
  .toArray([[1, [2]], [3, [4, [5]]]]); // [1, 2, 3, 4, [5]]
```

### `whereMatches(spec)`

Filter objects matching a specification pattern.

```javascript
new Pipeline()
  .whereMatches({ active: true, role: 'admin' })
  .toArray(users);
```

## Lens Pipeline Methods

### `viewLens(lens)`

Extract the focused value via a lens.

```javascript
const nameLens = lens('name');

new Pipeline()
  .viewLens(nameLens)
  .toArray(users); // ["Alice", "Bob", ...]
```

### `overLens(lens, fn)`

Transform values through a lens.

```javascript
const priceLens = lens('price');

new Pipeline()
  .overLens(priceLens, p => p * 0.9)
  .toArray(products); // each product with 10% discount
```

### `filterLens(lens, predicate)`

Filter by lens-focused value.

```javascript
const ageLens = lens('age');

new Pipeline()
  .filterLens(ageLens, age => age >= 18)
  .toArray(users);
```

### `setLens(lens, value)`

Set a fixed value via a lens on every element.

```javascript
const statusLens = lens('status');

new Pipeline()
  .setLens(statusLens, 'published')
  .toArray(posts);
```

## Terminal Operations (Collectors)

These execute the pipeline and return a result.

### `toArray(source)`

Collect all results into an array.

```typescript
toArray(source: Array<T>): Array<U>
```

### `reduce(source, reducer, initial)`

Custom reduction with a reducer function.

```typescript
reduce(source: Array<T>, reducer: (acc: A, value: U) => A, initial: A): A
```

```javascript
const sum = new Pipeline()
  .map(x => x * 2)
  .reduce([1, 2, 3, 4], (acc, x) => acc + x, 0);
// sum: 20
```

## Standalone Collectors

These functions operate independently of the Pipeline:

| Function | Description | Example |
|----------|-------------|---------|
| `find(pipeline, data, pred)` | Find first matching element | `find(pipeline, data, x => x > 10)` |
| `partition(pipeline, data, pred)` | Split into [matching, non-matching] | `partition(pipeline, data, isValid)` |
| `groupBy(pipeline, data, keyFn)` | Group elements by key | `groupBy(pipeline, data, x => x.type)` |
| `frequencies(data)` | Count occurrences | `frequencies([1, 2, 2, 3])` |
| `topK(data, k)` | Get k largest elements | `topK(scores, 10)` |

## Standalone Functions

### Statistical Operations

| Function | Description | Example |
|----------|-------------|---------|
| `product(array)` | Multiply all numbers | `product([2, 3, 4])` = 24 |
| `mean(array)` | Arithmetic mean | `mean([1, 2, 3, 4, 5])` = 3 |
| `median(array)` | Middle value | `median([1, 2, 3, 4, 5])` = 3 |
| `min(array)` / `max(array)` | Min/max value | `max([1, 5, 3])` = 5 |
| `minBy(array, fn)` / `maxBy(array, fn)` | Min/max by key | `maxBy(users, u => u.score)` |
| `variance(array)` | Sample variance | `variance([2, 4, 6, 8])` |
| `stdDev(array)` | Standard deviation | `stdDev([2, 4, 6, 8])` |
| `quantile(array, p)` | P-th quantile | `quantile(data, 0.95)` |
| `mode(array)` | Most frequent value | `mode([1, 2, 2, 3])` = 2 |

### Collection Utilities

| Function | Description | Example |
|----------|-------------|---------|
| `sortBy(array, fn)` | Sort by key | `sortBy(users, u => u.age)` |
| `sortWith(array, cmp)` | Sort with comparator | `sortWith(nums, (a,b) => a - b)` |
| `reverse(array)` | Reverse order | `reverse([1, 2, 3])` = [3, 2, 1] |
| `range(start, end, step)` | Numeric sequence | `range(0, 10, 2)` = [0, 2, 4, 6, 8] |
| `repeat(value, n)` | Repeat value | `repeat('x', 3)` = ['x', 'x', 'x'] |
| `cycle(array, n)` | Cycle array | `cycle([1, 2], 3)` = [1, 2, 1, 2, 1, 2] |
| `unfold(seed, fn, limit)` | Generate from seed | `unfold(1, x => x * 2, 5)` |
| `path(obj, pathArr)` | Safe deep access | `path(user, ['profile', 'email'])` |
| `pathOr(obj, path, default)` | Path with default | `pathOr(config, ['port'], 8080)` |
| `evolve(obj, transforms)` | Nested transforms | `evolve(user, { age: n => n + 1 })` |

### Logic Functions

| Function | Description | Example |
|----------|-------------|---------|
| `both(p1, p2)` | AND combinator | `both(isPositive, isEven)` |
| `either(p1, p2)` | OR combinator | `either(isSmall, isLarge)` |
| `complement(pred)` | NOT combinator | `complement(isEven)` |
| `allPass(preds)` | All must pass | `allPass([isValid, isActive])` |
| `anyPass(preds)` | Any must pass | `anyPass([isZero, isDivisibleBy10])` |
| `When(pred, fn)` | Conditional transform | `new When(x => x > 0, x => x * 2)` |
| `Unless(pred, fn)` | Inverse conditional | `new Unless(x => x > 0, _ => 0)` |
| `IfElse(pred, onTrue, onFalse)` | Branch | `new IfElse(x => x >= 0, double, halve)` |

### Multi-Input Operations

| Function | Description | Example |
|----------|-------------|---------|
| `merge(arrays)` | Interleave arrays | `merge([a, b, c])` |
| `zip(a, b)` | Combine into pairs | `zip([1,2], ['a','b'])` |
| `zipLongest(a, b, fill)` | Zip with fill | `zipLongest(a, b, null)` |
| `intersection(a, b)` | Common elements | `intersection(a, b)` |
| `union(a, b)` | Unique from both | `union(a, b)` |
| `difference(a, b)` | In a, not b | `difference(a, b)` |
| `symmetricDifference(a, b)` | In one, not both | `symmetricDifference(a, b)` |
| `cartesianProduct(a, b)` | All pairs | `cartesianProduct(colors, sizes)` |
| `takeLast(array, n)` | Last N elements | `takeLast([1,2,3,4,5], 3)` |
| `dropLast(array, n)` | Drop last N | `dropLast([1,2,3,4,5], 2)` |
| `aperture(array, n)` | Sliding windows | `aperture([1,2,3,4], 3)` |
