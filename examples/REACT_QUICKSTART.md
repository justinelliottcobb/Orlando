# React Quick Start Guide for Orlando Transducers

## Installation

```bash
npm install orlando-transducers
```

## ⚡ Key Point: No Initialization Needed!

With the bundler target, Orlando auto-initializes when your app loads. **No `init()` function required!**

## Basic Import

```javascript
import { Pipeline } from 'orlando-transducers';
```

That's it! Just import and use.

## Minimal Working Example

```jsx
import { useState } from 'react';
import { Pipeline } from 'orlando-transducers';

function App() {
  const [result, setResult] = useState(null);

  const handleClick = () => {
    const pipeline = new Pipeline()
      .map(x => x * 2)
      .filter(x => x > 10)
      .take(5);

    const data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    setResult(pipeline.toArray(data));
  };

  return (
    <div>
      <button onClick={handleClick}>Process Data</button>
      {result && <pre>{JSON.stringify(result)}</pre>}
    </div>
  );
}
```

## Common Pipeline Operations

```javascript
const pipeline = new Pipeline()
  // Transform
  .map(x => x * 2)

  // Filter
  .filter(x => x > 10)

  // Take first N
  .take(5)

  // Extract property from objects
  .pluck('name')

  // Side effects (debugging)
  .tap(x => console.log(x));

// Execute pipeline
const result = pipeline.toArray(inputData);
```

## Working with Objects

```javascript
const data = [
  { id: 1, name: 'Alice', age: 30 },
  { id: 2, name: 'Bob', age: 25 },
  { id: 3, name: 'Charlie', age: 35 }
];

const pipeline = new Pipeline()
  .filter(person => person.age > 25)
  .pluck('name')
  .map(name => name.toUpperCase());

const result = pipeline.toArray(data);
// => ['ALICE', 'CHARLIE']
```

## Performance Example

```javascript
function DataProcessor() {
  const [stats, setStats] = useState(null);

  const processLargeDataset = () => {
    const data = Array.from({ length: 100000 }, (_, i) => i);

    const pipeline = new Pipeline()
      .map(x => x * 2)
      .filter(x => x % 3 === 0)
      .filter(x => x > 1000)
      .take(100);

    const start = performance.now();
    const result = pipeline.toArray(data);
    const duration = performance.now() - start;

    setStats({
      inputSize: data.length,
      outputSize: result.length,
      processingTime: duration.toFixed(2)
    });
  };

  return (
    <div>
      <button onClick={processLargeDataset}>
        Process 100K Items
      </button>
      {stats && (
        <div>
          <p>Processed {stats.inputSize} items in {stats.processingTime}ms</p>
          <p>Result: {stats.outputSize} items</p>
        </div>
      )}
    </div>
  );
}
```

## Real-World Example: Data Table Filtering

```jsx
import { useState, useMemo } from 'react';
import { Pipeline } from 'orlando-transducers';

function UserTable({ users }) {
  const [searchTerm, setSearchTerm] = useState('');
  const [minAge, setMinAge] = useState(0);

  const filteredUsers = useMemo(() => {
    const pipeline = new Pipeline()
      .filter(user =>
        user.name.toLowerCase().includes(searchTerm.toLowerCase())
      )
      .filter(user => user.age >= minAge)
      .map(user => ({
        ...user,
        displayName: user.name.toUpperCase()
      }));

    return pipeline.toArray(users);
  }, [users, searchTerm, minAge]);

  return (
    <div>
      <input
        type="text"
        placeholder="Search by name"
        value={searchTerm}
        onChange={e => setSearchTerm(e.target.value)}
      />
      <input
        type="number"
        placeholder="Min age"
        value={minAge}
        onChange={e => setMinAge(Number(e.target.value))}
      />

      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Age</th>
          </tr>
        </thead>
        <tbody>
          {filteredUsers.map(user => (
            <tr key={user.id}>
              <td>{user.displayName}</td>
              <td>{user.age}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <p>Showing {filteredUsers.length} users</p>
    </div>
  );
}
```

## Custom Reusable Pipelines

```javascript
// Create reusable pipeline configurations
const createAdultFilter = () =>
  new Pipeline().filter(person => person.age >= 18);

const createNameExtractor = () =>
  new Pipeline()
    .pluck('name')
    .map(name => name.trim())
    .filter(name => name.length > 0);

function MyComponent({ people }) {
  // Compose pipelines
  const adultNames = createAdultFilter()
    .pluck('name')
    .toArray(people);

  const processedNames = createNameExtractor()
    .map(name => name.toUpperCase())
    .toArray(people);

  return (
    <div>
      <h3>Adult Names: {adultNames.join(', ')}</h3>
      <h3>Processed: {processedNames.join(', ')}</h3>
    </div>
  );
}
```

## TypeScript Support

```typescript
import { Pipeline } from 'orlando-transducers';

interface Person {
  name: string;
  age: number;
}

const data: Person[] = [
  { name: 'Alice', age: 30 },
  { name: 'Bob', age: 25 }
];

// Pipeline works with TypeScript
const pipeline = new Pipeline()
  .filter((p: Person) => p.age > 25)
  .pluck('name');

const names: any[] = pipeline.toArray(data);
```

## Build Tool Compatibility

### ✅ Vite (Recommended)

Works out of the box!

```bash
npm create vite@latest my-app -- --template react
cd my-app
npm install orlando-transducers
npm run dev
```

### ✅ Next.js

Add to `next.config.js`:

```javascript
module.exports = {
  webpack: (config) => {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
    };
    return config;
  },
};
```

### ⚠️ Create React App

May require additional webpack configuration. Consider using Vite instead.

## Common Questions

### Q: Do I need to initialize anything?

**No!** Just import and use:

```javascript
import { Pipeline } from 'orlando-transducers';
// Ready to use immediately
```

### Q: Can I use it in useEffect/useMemo?

**Yes!** It works anywhere:

```javascript
const result = useMemo(() => {
  const pipeline = new Pipeline().map(x => x * 2);
  return pipeline.toArray(data);
}, [data]);
```

### Q: Does it work with React Server Components?

Currently only client-side. Mark components with `'use client'`:

```javascript
'use client';
import { Pipeline } from 'orlando-transducers';
```

### Q: How do I handle errors?

Wrap pipeline operations in try-catch:

```javascript
try {
  const pipeline = new Pipeline().map(x => x * 2);
  const result = pipeline.toArray(data);
} catch (error) {
  console.error('Pipeline failed:', error);
}
```

## Performance Tips

1. **Create pipelines once, reuse them**:
   ```javascript
   // Good - create once
   const pipeline = useMemo(() =>
     new Pipeline().map(x => x * 2),
     []
   );

   // Avoid - creates new pipeline every render
   const result = new Pipeline().map(x => x * 2).toArray(data);
   ```

2. **Use early termination** with `take()`:
   ```javascript
   // Only processes until it gets 10 items
   const pipeline = new Pipeline()
     .filter(x => x > 100)
     .take(10);
   ```

3. **Chain filters efficiently** (most restrictive first):
   ```javascript
   // Good - eliminates most items first
   new Pipeline()
     .filter(x => x > 1000)      // Removes 90%
     .filter(x => x % 2 === 0);  // Removes 50% of remainder

   // Less efficient
   new Pipeline()
     .filter(x => x % 2 === 0)   // Removes 50%
     .filter(x => x > 1000);     // Removes 90% of remainder
   ```

## Resources

- [npm package](https://www.npmjs.com/package/orlando-transducers)
- [GitHub repository](https://github.com/justinelliottcobb/Orlando)
- [Examples](https://github.com/justinelliottcobb/Orlando/tree/main/examples)

## Need Help?

File an issue: https://github.com/justinelliottcobb/Orlando/issues
