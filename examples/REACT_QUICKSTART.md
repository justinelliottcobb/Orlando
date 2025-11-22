# React Quick Start Guide for Orlando Transducers

## Installation

```bash
npm install orlando-transducers
```

**For Vite users**, also install the WASM plugin:

```bash
npm install vite-plugin-wasm
```

See [Build Tool Compatibility](#build-tool-compatibility) for configuration.

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

## Immutable State Updates with Lenses

**NEW in v0.4.0!** Orlando now includes functional lenses for clean, type-safe state updates.

### Basic Lens Usage

```jsx
import { useState } from 'react';
import { lens, lensPath } from 'orlando-transducers';

function UserProfile() {
  const [user, setUser] = useState({
    name: 'Alice',
    email: 'alice@example.com',
    preferences: {
      theme: 'dark',
      notifications: true
    }
  });

  const nameLens = lens('name');
  const themeLens = lensPath(['preferences', 'theme']);

  const updateName = (newName) => {
    setUser(nameLens.set(user, newName));
  };

  const toggleTheme = () => {
    setUser(themeLens.over(user, theme =>
      theme === 'dark' ? 'light' : 'dark'
    ));
  };

  return (
    <div>
      <input
        value={user.name}
        onChange={(e) => updateName(e.target.value)}
      />
      <button onClick={toggleTheme}>
        Theme: {user.preferences.theme}
      </button>
    </div>
  );
}
```

### Lens Benefits over Manual Spreading

```jsx
// ❌ Traditional approach - verbose and error-prone
const updateCity = (newCity) => {
  setUser({
    ...user,
    address: {
      ...user.address,
      city: newCity
    }
  });
};

// ✅ With lenses - clean and safe
const cityLens = lensPath(['address', 'city']);
const updateCity = (newCity) => {
  setUser(cityLens.set(user, newCity));
};
```

### Complex State Management Example

```jsx
import { useState } from 'react';
import { lens, lensPath, optional } from 'orlando-transducers';

function ShoppingCart() {
  const [cart, setCart] = useState({
    items: [
      { id: 1, name: 'Widget', quantity: 2, price: 10 },
      { id: 2, name: 'Gadget', quantity: 1, price: 20 }
    ],
    discount: null
  });

  const itemsLens = lens('items');
  const discountLens = optional('discount');

  // Update quantity for a specific item
  const updateQuantity = (itemId, newQuantity) => {
    const updated = cart.items.map(item =>
      item.id === itemId
        ? lens('quantity').set(item, newQuantity)
        : item
    );
    setCart(itemsLens.set(cart, updated));
  };

  // Apply discount (optional field)
  const applyDiscount = (discountCode) => {
    setCart(discountLens.set(cart, { code: discountCode, percent: 10 }));
  };

  // Remove discount
  const removeDiscount = () => {
    const { discount, ...rest } = cart;
    setCart(rest);
  };

  const total = cart.items.reduce((sum, item) =>
    sum + (item.price * item.quantity), 0
  );

  const discountAmount = discountLens.get(cart)
    ? total * (cart.discount.percent / 100)
    : 0;

  return (
    <div>
      {cart.items.map(item => (
        <div key={item.id}>
          {item.name} × {item.quantity}
          <button onClick={() => updateQuantity(item.id, item.quantity + 1)}>
            +
          </button>
        </div>
      ))}
      <p>Subtotal: ${total}</p>
      {discountAmount > 0 && <p>Discount: -${discountAmount}</p>}
      <p>Total: ${total - discountAmount}</p>
    </div>
  );
}
```

### Combining Lenses with Transducers

```jsx
import { useState, useMemo } from 'react';
import { lens, Pipeline } from 'orlando-transducers';

function ProductList({ products }) {
  const [search, setSearch] = useState('');
  const [minPrice, setMinPrice] = useState(0);

  const nameLens = lens('name');
  const priceLens = lens('price');

  const filtered = useMemo(() => {
    return new Pipeline()
      .filter(product =>
        nameLens.get(product).toLowerCase().includes(search.toLowerCase())
      )
      .filter(product => priceLens.get(product) >= minPrice)
      .map(product => ({
        ...product,
        displayPrice: `$${priceLens.get(product).toFixed(2)}`
      }))
      .toArray(products);
  }, [products, search, minPrice]);

  return (
    <div>
      <input
        type="text"
        placeholder="Search products..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
      />
      <input
        type="number"
        placeholder="Min price"
        value={minPrice}
        onChange={(e) => setMinPrice(Number(e.target.value))}
      />
      <ul>
        {filtered.map(product => (
          <li key={product.id}>
            {product.name} - {product.displayPrice}
          </li>
        ))}
      </ul>
    </div>
  );
}
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

Requires the WASM plugin:

```bash
npm create vite@latest my-app -- --template react
cd my-app
npm install orlando-transducers vite-plugin-wasm
```

Add to `vite.config.js`:

```javascript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [react(), wasm()],
});
```

Then run:

```bash
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
