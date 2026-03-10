# Optics Composition Patterns

Patterns for combining Orlando's optics with transducer pipelines for expressive, immutable data transformations.

## Streaming Lenses: Optics + Transducers

Orlando uniquely combines lenses with transducer pipelines. No other lens library offers this.

### Extract, Filter, Transform

```javascript
import init, { Pipeline, lens, lensPath } from 'orlando-transducers';
await init();

const users = [
  { name: "Alice", profile: { email: "alice@company.com", verified: true }},
  { name: "Bob", profile: { email: "bob@gmail.com", verified: false }},
  { name: "Carol", profile: { email: "carol@company.com", verified: true }},
];

const emailLens = lensPath(['profile', 'email']);
const verifiedLens = lensPath(['profile', 'verified']);

// Pipeline with lens-based extraction
const companyEmails = new Pipeline()
  .filterLens(verifiedLens, v => v === true)   // filter by lens value
  .viewLens(emailLens)                          // extract via lens
  .filter(email => email.endsWith('@company.com'))
  .toArray(users);

// Result: ["alice@company.com", "carol@company.com"]
```

### Batch Updates via Pipeline

```javascript
const products = [
  { id: 1, name: "Widget", price: 10, category: "tools" },
  { id: 2, name: "Gadget", price: 20, category: "tools" },
  { id: 3, name: "Doohickey", price: 15, category: "accessories" },
];

const priceLens = lens('price');

// Apply 20% discount to all items via pipeline
const discounted = new Pipeline()
  .overLens(priceLens, price => price * 0.8)
  .toArray(products);

// Each product has price * 0.8, originals unchanged
```

### Selective Updates

```javascript
const categoryLens = lens('category');
const priceLens = lens('price');

// Discount only tools
const discountTools = new Pipeline()
  .map(product => {
    if (categoryLens.get(product) === 'tools') {
      return priceLens.over(product, price => price * 0.8);
    }
    return product;
  })
  .toArray(products);
```

## Redux-Style State Management

### Lens-Based Reducers

```javascript
import { lens, lensPath } from 'orlando-transducers';

const state = {
  user: {
    profile: { name: "Alice", email: "alice@example.com" },
    preferences: { theme: "dark", notifications: true },
  },
  cart: { items: [], total: 0 },
};

// Define lenses for each slice of state
const nameLens = lensPath(['user', 'profile', 'name']);
const themeLens = lensPath(['user', 'preferences', 'theme']);
const cartLens = lens('cart');
const totalLens = lensPath(['cart', 'total']);

// Reducers become simple lens operations
function reducer(state, action) {
  switch (action.type) {
    case 'SET_NAME':
      return nameLens.set(state, action.payload);

    case 'TOGGLE_THEME':
      return themeLens.over(state, theme =>
        theme === 'dark' ? 'light' : 'dark'
      );

    case 'SET_TOTAL':
      return totalLens.set(state, action.payload);

    default:
      return state;
  }
}
```

### Multiple Updates

```javascript
// Chain lens operations for multiple immutable updates
const newState = themeLens.set(
  nameLens.set(state, "Alicia"),
  "light"
);

// Original state unchanged
console.log(state.user.profile.name);       // "Alice"
console.log(newState.user.profile.name);    // "Alicia"
console.log(newState.user.preferences.theme); // "light"
```

## Deep Nested Access with lensPath

```javascript
const config = {
  database: {
    primary: {
      host: "db.example.com",
      port: 5432,
      credentials: {
        username: "admin",
        password: "secret",
      },
    },
  },
};

const dbHostLens = lensPath(['database', 'primary', 'host']);
const dbPortLens = lensPath(['database', 'primary', 'port']);
const dbUserLens = lensPath(['database', 'primary', 'credentials', 'username']);

dbHostLens.get(config);              // "db.example.com"
dbPortLens.set(config, 5433);        // new config with updated port
dbUserLens.over(config, u => u.toUpperCase());  // new config with "ADMIN"
```

## Optional Fields

```javascript
import { optional } from 'orlando-transducers';

const phoneLens = optional('phone');
const bioLens = optional('bio');

const users = [
  { name: "Alice", phone: "555-0100" },
  { name: "Bob" },  // no phone
  { name: "Carol", phone: "555-0102", bio: "Developer" },
];

// Safe extraction with defaults
const pipeline = new Pipeline()
  .map(user => ({
    name: user.name,
    phone: phoneLens.getOr(user, "N/A"),
    bio: bioLens.getOr(user, "No bio provided"),
  }))
  .toArray(users);

// Result:
// [
//   { name: "Alice", phone: "555-0100", bio: "No bio provided" },
//   { name: "Bob", phone: "N/A", bio: "No bio provided" },
//   { name: "Carol", phone: "555-0102", bio: "Developer" },
// ]
```

## Prism Patterns: Sum Types

```javascript
import { prism } from 'orlando-transducers';

// API response: { status: "success", data: ... } or { status: "error", message: ... }
const successPrism = prism(
  resp => resp.status === 'success' ? resp.data : undefined,
  data => ({ status: 'success', data })
);

const errorPrism = prism(
  resp => resp.status === 'error' ? resp.message : undefined,
  message => ({ status: 'error', message })
);

const responses = [
  { status: 'success', data: { id: 1 } },
  { status: 'error', message: 'Not found' },
  { status: 'success', data: { id: 2 } },
];

// Extract only successful data
const pipeline = new Pipeline()
  .map(resp => successPrism.preview(resp))
  .filter(data => data !== undefined)
  .toArray(responses);

// Result: [{ id: 1 }, { id: 2 }]
```

## Rust: Optics Composition

```rust
use orlando_transducers::optics::{Lens, Fold, Traversal};

// Compose lenses for deep access
let department_lens = Lens::new(
    |company: &Company| company.department.clone(),
    |company: &Company, dept: Department| Company { department: dept, ..company.clone() },
);

let employees_traversal = Traversal::new(
    |dept: &Department| dept.employees.clone(),
    |dept: &Department, f: &dyn Fn(&Employee) -> Employee| {
        Department {
            employees: dept.employees.iter().map(f).collect(),
            ..dept.clone()
        }
    },
);

let name_lens = Lens::new(
    |emp: &Employee| emp.name.clone(),
    |emp: &Employee, name: String| Employee { name, ..emp.clone() },
);

// Company -> Department -> [Employee] -> name
// Use traversal to update all employee names
let dept = employees_traversal.over_all(&department, |emp| {
    name_lens.over(emp, |n| n.to_uppercase())
});
```

## Iso Patterns: Unit Conversions

```rust
use orlando_transducers::optics::Iso;

let meters_feet = Iso::new(
    |m: &f64| m * 3.28084,
    |f: &f64| f / 3.28084,
);

let celsius_fahrenheit = Iso::new(
    |c: &f64| c * 9.0 / 5.0 + 32.0,
    |f: &f64| (f - 32.0) * 5.0 / 9.0,
);

// Isos are reversible
let feet_meters = meters_feet.reverse();
assert_eq!(feet_meters.to(&3.28084), 1.0);

// Isos can be used as lenses
let as_lens = celsius_fahrenheit.as_lens();
let f = as_lens.get(&100.0);  // 212.0
```
