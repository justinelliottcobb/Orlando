# Optics API

Orlando provides a complete hierarchy of functional optics for immutable, composable data access and transformation.

## Overview

| Optic | Focus | Read | Write | Use Case |
|-------|-------|------|-------|----------|
| **Lens** | Exactly one value | Yes | Yes | Object properties, nested fields |
| **Optional** | Zero or one value | Yes | Yes | Nullable fields, partial data |
| **Prism** | Zero or one (sum type) | Yes | Yes (construct) | Tagged unions, enum variants |
| **Iso** | Exactly one (bidirectional) | Yes | Yes | Unit conversions, encodings |
| **Traversal** | Zero or more values | Yes | Yes | Collections, arrays |
| **Fold** | Zero or more values | Yes | No | Read-only aggregation |

## Lens

Focus on exactly one part of a data structure with read/write access.

### JavaScript

```javascript
import { lens, lensPath } from 'orlando-transducers';

// Property lens
const nameLens = lens('name');
nameLens.get(user);                              // "Alice"
nameLens.set(user, "Bob");                       // { ...user, name: "Bob" }
nameLens.over(user, s => s.toUpperCase());       // { ...user, name: "ALICE" }

// Path lens for deep access
const cityLens = lensPath(['address', 'city']);
cityLens.get(user);                              // "NYC"
cityLens.set(user, "Boston");                    // deep immutable update

// Composition
const addressLens = lens('address');
const zipLens = lens('zip');
const userZipLens = addressLens.compose(zipLens);
userZipLens.get(user);                           // "10001"
```

### Rust

```rust
use orlando_transducers::optics::Lens;

let name_lens = Lens::new(
    |user: &User| user.name.clone(),
    |user: &User, name: String| User { name, ..user.clone() },
);

let name = name_lens.get(&user);
let updated = name_lens.set(&user, "Bob".into());
let shouted = name_lens.over(&user, |n| n.to_uppercase());

// Composition via then()
let user_city = address_lens.then(&city_lens);
```

### Lens Laws

All Orlando lenses satisfy:

1. **GetPut**: `set(s, get(s)) = s`
2. **PutGet**: `get(set(s, a)) = a`
3. **PutPut**: `set(set(s, a1), a2) = set(s, a2)`

## Optional

Like a Lens, but the focus may not exist. Safe for nullable or missing fields.

### JavaScript

```javascript
import { optional } from 'orlando-transducers';

const phoneLens = optional('phone');
phoneLens.get(user);                 // undefined (missing field)
phoneLens.getOr(user, "N/A");       // "N/A" (with default)
phoneLens.set(user, "555-0100");     // { ...user, phone: "555-0100" }
phoneLens.over(user, normalize);     // no-op if undefined
```

### Rust

```rust
use orlando_transducers::optics::Optional;

let phone = Optional::new(
    |u: &User| u.phone.clone(),
    |u: &User, p: String| User { phone: Some(p), ..u.clone() },
);

let val = phone.get_or(&user, "N/A".into());
```

## Prism

Focus on one variant of a sum type. Can both match (`preview`) and construct (`review`).

### JavaScript

```javascript
import { prism } from 'orlando-transducers';

const somePrism = prism(
  x => x.tag === 'Some' ? x.value : undefined,  // preview
  v => ({ tag: 'Some', value: v })                // review
);

somePrism.preview({ tag: 'Some', value: 42 });   // 42
somePrism.preview({ tag: 'None' });               // undefined
somePrism.review(42);                              // { tag: 'Some', value: 42 }
```

### Rust

```rust
use orlando_transducers::optics::Prism;

let some_prism = Prism::new(
    |opt: &Option<i32>| *opt,
    |val: i32| Some(val),
);

assert_eq!(some_prism.preview(&Some(42)), Some(42));
assert_eq!(some_prism.review(42), Some(42));
```

## Iso

Lossless, bidirectional conversion between two types.

### JavaScript

```javascript
import { iso } from 'orlando-transducers';

const tempIso = iso(
  c => c * 9/5 + 32,   // Celsius -> Fahrenheit
  f => (f - 32) * 5/9   // Fahrenheit -> Celsius
);

tempIso.to(100);        // 212
tempIso.from(212);      // 100
tempIso.reverse().to(212);  // 100
```

### Rust

```rust
use orlando_transducers::optics::Iso;

let celsius_fahrenheit = Iso::new(
    |c: &f64| c * 9.0 / 5.0 + 32.0,
    |f: &f64| (f - 32.0) * 5.0 / 9.0,
);

// Can be used as either a Lens or a Prism
let as_lens = celsius_fahrenheit.as_lens();
let as_prism = celsius_fahrenheit.as_prism();
```

## Traversal

Focus on zero or more values within a structure. Supports reading all and updating all.

### JavaScript

```javascript
import { traversal } from 'orlando-transducers';

const itemsTraversal = traversal(
  arr => arr,
  (arr, fn) => arr.map(fn)
);

itemsTraversal.getAll([1, 2, 3]);               // [1, 2, 3]
itemsTraversal.overAll([1, 2, 3], x => x * 2);  // [2, 4, 6]
itemsTraversal.setAll([1, 2, 3], 0);             // [0, 0, 0]
```

### Rust

```rust
use orlando_transducers::optics::Traversal;

let each = Traversal::new(
    |v: &Vec<i32>| v.clone(),
    |v: &Vec<i32>, f: &dyn Fn(&i32) -> i32| v.iter().map(f).collect(),
);

let doubled = each.over_all(&vec![1, 2, 3], |x| x * 2);  // [2, 4, 6]
```

## Fold

Read-only traversal for extracting and aggregating values.

### JavaScript

```javascript
import { fold } from 'orlando-transducers';

const valuesFold = fold(obj => Object.values(obj));

valuesFold.getAll({ a: 1, b: 2, c: 3 });  // [1, 2, 3]
valuesFold.isEmpty({});                     // true
valuesFold.length({ a: 1, b: 2 });          // 2
valuesFold.first({ a: 1, b: 2 });           // 1
```

### Rust

```rust
use orlando_transducers::optics::Fold;

let items = Fold::fold_of(|v: &Vec<i32>| v.clone());

items.any(&data, |x| *x > 10);     // true if any > 10
items.all(&data, |x| *x > 0);      // true if all > 0
items.find(&data, |x| *x > 5);     // Some(first > 5)
items.is_empty(&data);              // bool
items.length(&data);                // usize
items.first(&data);                 // Option<i32>
```

## Cross-Type Conversions

Optics can be widened to more general types:

| From | To | Method |
|------|-----|--------|
| Lens | Traversal | `.to_traversal()` |
| Lens | Fold | `.to_fold()` |
| Prism | Traversal | `.to_traversal()` |
| Prism | Fold | `.to_fold()` |
| Iso | Lens | `.as_lens()` |
| Iso | Prism | `.as_prism()` |
| Traversal | Fold | `.as_fold()` |

## Composition

All optics support composition for deeper access:

```javascript
// JavaScript
const userCity = addressLens.compose(cityLens);
```

```rust
// Rust
let user_city = address_lens.then(&city_lens);
let deep_fold = outer_fold.then(&inner_fold);
let nested = outer_traversal.then(&inner_traversal);
```
