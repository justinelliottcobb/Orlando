# Profunctor Optics API

Orlando's optics use a **profunctor encoding** via [Karpal](https://crates.io/crates/karpal-optics) for principled composition and cross-type conversions.

## Profunctor Constraints

Each optic type corresponds to a constraint on a profunctor:

| Optic | Constraint | Meaning |
|-------|-----------|---------|
| **Lens** | `Strong` | Focus through products (structs/tuples) |
| **Prism** | `Choice` | Focus through sums (enums/variants) |
| **Iso** | `Profunctor` | Only needs `dimap` (weakest) |
| **Traversal** | `Traversing` | Focus through collections |

## `transform()` Method

Each optic exposes its profunctor encoding:

```rust
use orlando_transducers::optics::{Lens, Prism, Iso, Traversal};

// Lens -> Strong profunctor
let strong = lens.transform();

// Prism -> Choice profunctor
let choice = prism.transform();

// Iso -> Profunctor (weakest constraint)
let prof = iso.transform();

// Traversal -> Traversing profunctor
let traversing = traversal.transform();
```

## Karpal Profunctor Traits

Re-exported from Karpal for use with Orlando's optics.

### `Profunctor`

The base trait. Supports `dimap(f, g)` for mapping over both input and output.

```rust
use orlando_transducers::profunctor::Profunctor;

// dimap transforms both sides of a profunctor
// p.dimap(f, g) where f: B -> A, g: C -> D gives P<B, D> from P<A, C>
```

### `Strong`

Extends `Profunctor` with product operations. Used by Lens.

```rust
use orlando_transducers::profunctor::Strong;

// first(): P<A, B> -> P<(A, C), (B, C)>
// second(): P<A, B> -> P<(C, A), (C, B)>
```

### `Choice`

Extends `Profunctor` with sum operations. Used by Prism.

```rust
use orlando_transducers::profunctor::Choice;

// left(): P<A, B> -> P<Either<A, C>, Either<B, C>>
// right(): P<A, B> -> P<Either<C, A>, Either<C, B>>
```

### `Traversing`

Extends `Strong` with collection operations. Used by Traversal.

```rust
use orlando_transducers::profunctor::Traversing;

// wander(): applies a profunctor across multiple foci
```

## Concrete Profunctor Types

| Type | Description | Use Case |
|------|-------------|----------|
| `FnP<A, B>` | Function arrow `A -> B` | Getting and setting (modify) |
| `ForgetF<R, A, B>` | Forgets `B`, extracts `R` from `A` | Read-only access (getters, folds) |
| `TaggedF<A, B>` | Forgets `A`, produces `B` | Write-only access (review/construct) |

## Re-exported Types

```rust
use orlando_transducers::profunctor::{
    Profunctor, Strong, Choice, Traversing,
    FnP, ForgetF, TaggedF, Monoid,
};

// Also available from lib.rs:
use orlando_transducers::{Getter, Setter, Review};
```

## Composition with `then()`

Compose optics while preserving profunctor constraints:

```rust
// Lens + Lens = Lens (both Strong)
let user_city = address_lens.then(&city_lens);

// Fold + Fold = Fold
let all_names = users_fold.then(&name_fold);

// Traversal + Traversal = Traversal (both Traversing)
let nested = outer.then(&inner);
```

## Cross-Type Conversions

The hierarchy flows from specific to general:

```
Iso (Profunctor — weakest constraint)
 ├── Lens (Strong)
 └── Prism (Choice)
        ├── Traversal (Traversing)
        └── Fold (read-only)
```

```rust
let traversal = lens.to_traversal();
let fold = lens.to_fold();
let fold = prism.to_fold();
let lens = iso.as_lens();
let prism = iso.as_prism();
let fold = traversal.as_fold();
```

## Fold Aggregation

Folds support rich queries over focused values:

```rust
let items = Fold::fold_of(|v: &Vec<i32>| v.clone());

items.any(&data, |x| *x > 10);      // bool
items.all(&data, |x| *x > 0);       // bool
items.find(&data, |x| *x > 5);      // Option<i32>
items.is_empty(&data);               // bool
items.length(&data);                 // usize
items.first(&data);                  // Option<i32>
```

## Storage Model

Orlando uses `Rc<dyn Fn>` for optic closures, enabling:

- **Cloning** - Optics can be freely cloned and shared
- **Composition** - Both sides of a composition can reference the same optic
- **`ComposedLens<S, A>`** is simply a type alias for `Lens<S, A>`

The `Rc` overhead is negligible, and WASM is single-threaded so `Send`/`Sync` are not required.
