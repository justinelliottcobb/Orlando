# Migration Guide: From Array Methods to Orlando Transducers

A practical guide for converting JavaScript array operations to Orlando transducers.

## Table of Contents

- [Why Migrate?](#why-migrate)
- [Basic Conversions](#basic-conversions)
- [Common Patterns](#common-patterns)
- [Performance Gotchas](#performance-gotchas)
- [Advanced Patterns](#advanced-patterns)
- [Troubleshooting](#troubleshooting)

## Why Migrate?

### Array Methods Create Intermediate Arrays

```javascript
// ❌ Traditional approach - creates 2 intermediate arrays
const result = data
  .map(x => x * 2)        // Intermediate array 1
  .filter(x => x > 10)    // Intermediate array 2
  .slice(0, 5);           // Final result
```

**Problems:**
- Memory allocation for each step
- Full iteration even if you only need first N results
- Garbage collection overhead

### Orlando Processes in a Single Pass

```javascript
// ✅ Orlando approach - single pass, no intermediates
import init, { Pipeline } from 'orlando-transducers';
await init();

const pipeline = new Pipeline()
  .map(x => x * 2)
  .filter(x => x > 10)
  .take(5);

const result = pipeline.toArray(data);
```

**Benefits:**
- No intermediate allocations
- Early termination (stops after collecting 5 elements)
- Single pass over data
- WASM-powered performance

## Basic Conversions

### Map

**Before (Array):**
```javascript
const doubled = numbers.map(x => x * 2);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .map(x => x * 2);

const doubled = pipeline.toArray(numbers);
```

---

### Filter

**Before (Array):**
```javascript
const evens = numbers.filter(x => x % 2 === 0);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .filter(x => x % 2 === 0);

const evens = pipeline.toArray(numbers);
```

---

### Map + Filter

**Before (Array):**
```javascript
const result = numbers
  .map(x => x * 2)
  .filter(x => x > 10);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .map(x => x * 2)
  .filter(x => x > 10);

const result = pipeline.toArray(numbers);
```

---

### Take (slice)

**Before (Array):**
```javascript
const first5 = numbers.slice(0, 5);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .take(5);

const first5 = pipeline.toArray(numbers);
```

**💡 Performance Win:** Orlando stops processing after 5 elements. Array methods process everything first, then slice.

---

### Drop (slice)

**Before (Array):**
```javascript
const skip3 = numbers.slice(3);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .drop(3);

const skip3 = pipeline.toArray(numbers);
```

---

### Find First

**Before (Array):**
```javascript
const first = numbers.find(x => x > 100);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .filter(x => x > 100)
  .take(1);

const result = pipeline.toArray(numbers);
const first = result[0]; // or undefined
```

**💡 Performance Win:** Orlando stops immediately after finding the first match.

---

### Reduce (Sum)

**Before (Array):**
```javascript
const sum = numbers.reduce((acc, x) => acc + x, 0);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .map(x => x); // or apply transformations

const sum = pipeline.reduce(
  numbers,
  (acc, x) => acc + x,
  0
);
```

---

## Common Patterns

### Pagination

**Before (Array):**
```javascript
function paginate(data, page, pageSize) {
  const start = (page - 1) * pageSize;
  return data.slice(start, start + pageSize);
}

const page2 = paginate(users, 2, 20);
```

**After (Orlando):**
```javascript
function paginate(data, page, pageSize) {
  return new Pipeline()
    .drop((page - 1) * pageSize)
    .take(pageSize)
    .toArray(data);
}

const page2 = paginate(users, 2, 20);
```

**💡 Performance Win:** Orlando only processes the exact slice needed, not the entire array.

---

### Data Transformation Pipeline

**Before (Array):**
```javascript
const activeCompanyEmails = users
  .filter(user => user.active)
  .map(user => ({
    id: user.id,
    email: user.email.toLowerCase()
  }))
  .filter(user => user.email.endsWith('@company.com'))
  .map(user => user.email)
  .slice(0, 100);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .filter(user => user.active)
  .map(user => ({
    id: user.id,
    email: user.email.toLowerCase()
  }))
  .filter(user => user.email.endsWith('@company.com'))
  .map(user => user.email)
  .take(100);

const activeCompanyEmails = pipeline.toArray(users);
```

**💡 Performance Win:**
- Single pass (no intermediate arrays)
- Early termination (stops at 100 emails)
- WASM-powered execution

---

### Search with Multiple Filters

**Before (Array):**
```javascript
const searchProducts = (products, filters) => {
  return products
    .filter(p => p.category === filters.category)
    .filter(p => p.price >= filters.minPrice)
    .filter(p => p.price <= filters.maxPrice)
    .filter(p => p.rating >= filters.minRating)
    .filter(p => p.inStock)
    .slice(0, filters.limit || 20);
};
```

**After (Orlando):**
```javascript
const searchProducts = (products, filters) => {
  const pipeline = new Pipeline()
    .filter(p => p.category === filters.category)
    .filter(p => p.price >= filters.minPrice)
    .filter(p => p.price <= filters.maxPrice)
    .filter(p => p.rating >= filters.minRating)
    .filter(p => p.inStock)
    .take(filters.limit || 20);

  return pipeline.toArray(products);
};
```

---

### Analytics Aggregation

**Before (Array):**
```javascript
// Calculate total revenue from purchases
const purchases = events
  .filter(e => e.type === 'purchase')
  .map(e => e.amount);

const totalRevenue = purchases.reduce((sum, amt) => sum + amt, 0);
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .filter(e => e.type === 'purchase')
  .map(e => e.amount);

const totalRevenue = pipeline.reduce(
  events,
  (sum, amt) => sum + amt,
  0
);
```

---

### Top N with Sorting

**Before (Array):**
```javascript
const top10 = products
  .filter(p => p.inStock)
  .sort((a, b) => b.sales - a.sales)
  .slice(0, 10);
```

**After (Orlando):**
```javascript
// Note: Orlando doesn't have built-in sort (sorting requires seeing all data)
// For this pattern, sort BEFORE the pipeline or use a hybrid approach

const sorted = products
  .filter(p => p.inStock)
  .sort((a, b) => b.sales - a.sales);

const top10 = new Pipeline()
  .take(10)
  .toArray(sorted);

// Or use array sort, then Orlando for rest of pipeline
const top10 = new Pipeline()
  .filter(p => p.inStock)
  .toArray(products)
  .sort((a, b) => b.sales - a.sales)
  .slice(0, 10);
```

**⚠️ Note:** Transducers are best for operations that don't require seeing all data at once. For sorting, use array methods or sort before/after the pipeline.

---

## Performance Gotchas

### 1. Small Datasets (<100 elements)

**Array methods may be faster!**

```javascript
// For small data, array methods have less overhead
const small = [1, 2, 3, 4, 5];

// This is fine (overhead is negligible)
const result = small.map(x => x * 2).filter(x => x > 5);

// Orlando overhead may not be worth it for tiny datasets
```

**Rule of thumb:** Use Orlando for datasets >1000 elements or complex pipelines.

---

### 2. Single Operation

**Array methods are simpler for single operations:**

```javascript
// ❌ Overkill for single operation
const doubled = new Pipeline()
  .map(x => x * 2)
  .toArray(numbers);

// ✅ Just use array method
const doubled = numbers.map(x => x * 2);
```

**Use Orlando when:** You have 2+ operations, especially with early termination.

---

### 3. Need All Data Anyway

**If processing everything, Orlando advantage is smaller:**

```javascript
// If you need all 1M results anyway, Orlando is still faster but less dramatic
const allDoubled = new Pipeline()
  .map(x => x * 2)
  .toArray(oneMillion);

// vs
const allDoubled = oneMillion.map(x => x * 2);

// Orlando still wins (no intermediate arrays), but margin is smaller
```

**Biggest wins:** Early termination scenarios (take, takeWhile, find first).

---

## Advanced Patterns

### Reusable Pipelines

**Before (Array):**
```javascript
// Have to repeat the chain
const activeUsers1 = users1.filter(u => u.active).map(u => u.email);
const activeUsers2 = users2.filter(u => u.active).map(u => u.email);
```

**After (Orlando):**
```javascript
// Define once, reuse many times
const activeEmailPipeline = new Pipeline()
  .filter(u => u.active)
  .map(u => u.email);

const activeUsers1 = activeEmailPipeline.toArray(users1);
const activeUsers2 = activeEmailPipeline.toArray(users2);
const activeUsers3 = activeEmailPipeline.toArray(users3);
```

---

### Debugging with Tap

**Before (Array):**
```javascript
const result = data
  .map(x => {
    console.log('Input:', x);
    return x * 2;
  })
  .filter(x => {
    console.log('After map:', x);
    return x > 10;
  });
```

**After (Orlando):**
```javascript
const pipeline = new Pipeline()
  .tap(x => console.log('Input:', x))
  .map(x => x * 2)
  .tap(x => console.log('After map:', x))
  .filter(x => x > 10)
  .tap(x => console.log('After filter:', x));

const result = pipeline.toArray(data);
```

---

### Conditional Pipelines

**Before (Array):**
```javascript
let result = data.map(x => x * 2);

if (needsFiltering) {
  result = result.filter(x => x > 10);
}

if (limit) {
  result = result.slice(0, limit);
}
```

**After (Orlando):**
```javascript
let pipeline = new Pipeline()
  .map(x => x * 2);

if (needsFiltering) {
  pipeline = pipeline.filter(x => x > 10);
}

if (limit) {
  pipeline = pipeline.take(limit);
}

const result = pipeline.toArray(data);
```

---

## Troubleshooting

### "Pipeline is not iterable"

**Problem:**
```javascript
// ❌ Won't work
for (const item of pipeline) {
  console.log(item);
}
```

**Solution:**
Pipelines are not iterables. Use `.toArray()` to execute:

```javascript
// ✅ Correct
const result = pipeline.toArray(data);
for (const item of result) {
  console.log(item);
}
```

---

### "Cannot read property of undefined"

**Problem:**
```javascript
const pipeline = new Pipeline();
const result = pipeline.toArray(); // ❌ Missing source data
```

**Solution:**
Always provide source data to terminal operations:

```javascript
const result = pipeline.toArray(data); // ✅ Provide data
```

---

### Type Errors in TypeScript

**Problem:**
```javascript
const pipeline = new Pipeline()
  .map(x => x * 2)  // x is 'any'
  .filter(x => x.length > 0); // Runtime error if x is number
```

**Solution:**
Add type annotations to your functions:

```typescript
const pipeline = new Pipeline()
  .map((x: number) => x * 2)
  .filter((x: number) => x > 10);
```

---

### Performance Not Improving

**Check:**
1. **Dataset size:** Orlando shines on large datasets (>1000 elements)
2. **Early termination:** Are you using `take` or `takeWhile`?
3. **Complexity:** Single operations may not benefit much
4. **Initialization:** Are you reusing pipelines or creating new ones each time?

**Good scenario for Orlando:**
```javascript
// Large dataset + complex pipeline + early termination
const result = new Pipeline()
  .map(/* expensive operation */)
  .filter(/* complex condition */)
  .map(/* another transformation */)
  .take(10)  // Early termination!
  .toArray(millionItems);
```

**Not ideal for Orlando:**
```javascript
// Small dataset + single operation
const result = new Pipeline()
  .map(x => x * 2)
  .toArray([1, 2, 3, 4, 5]);
```

---

## Summary: When to Use Orlando

### ✅ Great for:
- Large datasets (>1000 elements)
- Complex pipelines (3+ operations)
- Early termination scenarios (take, takeWhile)
- Reusable transformation pipelines
- Performance-critical code
- Reducing memory allocations

### ⚠️ Consider array methods for:
- Small datasets (<100 elements)
- Single operations
- Prototyping / quick scripts
- When you need array methods not in Orlando (e.g., sort, reverse)

---

## Next Steps

- Read the [JavaScript API Documentation](./JAVASCRIPT.md)
- Try the [Interactive Demo](../../examples/index.html)
- Run [Performance Benchmarks](../../examples/performance.html)
- Explore [Real-World Examples](../../examples/data-processing.html)
