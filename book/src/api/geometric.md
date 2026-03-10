# Geometric Optics API

Operations on multivector coefficient arrays for geometric algebra. These work on plain `&[f64]` (Rust) or `Float64Array` (JavaScript).

In geometric algebra, a multivector with `n` dimensions has `2^n` coefficients, one per basis blade, organized by **grade** (scalar = 0, vectors = 1, bivectors = 2, etc.).

## Grade Inspection

### `bladeGrade(index)` / `blade_grade(index)`

Compute the grade of a basis blade from its index (popcount).

```javascript
bladeGrade(0);  // 0 (scalar)
bladeGrade(1);  // 1 (e1)
bladeGrade(3);  // 2 (e12)
bladeGrade(7);  // 3 (e123)
```

### `bladesAtGradeCount(dimension, grade)` / `blades_at_grade_count(dimension, grade)`

Number of basis blades at a given grade (binomial coefficient).

```javascript
bladesAtGradeCount(3, 0);  // 1 (scalar)
bladesAtGradeCount(3, 1);  // 3 (vectors: e1, e2, e3)
bladesAtGradeCount(3, 2);  // 3 (bivectors: e12, e13, e23)
bladesAtGradeCount(3, 3);  // 1 (pseudoscalar: e123)
```

### `gradeIndices(dimension, grade)` / `grade_indices(dimension, grade)`

Get coefficient array indices for all blades at a given grade.

```javascript
gradeIndices(3, 1);  // [1, 2, 4] (indices of e1, e2, e3)
gradeIndices(3, 2);  // [3, 5, 6] (indices of e12, e13, e23)
```

## Grade Extraction and Projection

### `gradeExtract(dimension, grade, mv)` / `grade_extract(dimension, grade, coefficients)`

Extract only the coefficients at a specific grade.

```javascript
const mv = new Float64Array([1, 2, 3, 4, 5, 6, 7, 8]);
gradeExtract(3, 1, mv);  // [2, 3, 5] (vector components)
```

### `gradeProject(dimension, grade, mv)` / `grade_project(dimension, grade, coefficients)`

Project onto a single grade, zeroing all others. Returns a full-size multivector.

```javascript
const mv = new Float64Array([1, 2, 3, 4, 5, 6, 7, 8]);
gradeProject(3, 1, mv);  // [0, 2, 3, 0, 5, 0, 0, 0]
```

### `gradeProjectMax(dimension, maxGrade, mv)` / `grade_project_max(dimension, max_grade, coefficients)`

Project onto all grades up to and including `maxGrade`.

```javascript
gradeProjectMax(3, 1, mv);  // [1, 2, 3, 0, 5, 0, 0, 0] (scalar + vector)
```

## Grade Analysis

### `gradeMask(dimension, mv)` / `grade_mask(dimension, coefficients)`

Bitmask indicating which grades have non-zero components. Bit `k` is set if grade `k` is present.

```javascript
const pure_vector = new Float64Array([0, 1, 0, 0, 0, 0, 0, 0]);
gradeMask(3, pure_vector);  // 0b010 = 2 (only grade 1)
```

### `hasGrade(dimension, grade, mv)` / `has_grade(dimension, grade, coefficients)`

Check if a specific grade has non-zero components.

```javascript
hasGrade(3, 1, mv);  // true
hasGrade(3, 3, mv);  // true (e123 component is 8)
```

### `isPureGrade(dimension, mv)` / `is_pure_grade(dimension, coefficients)`

Check if the multivector has components at only one grade.

```javascript
const pure = new Float64Array([0, 1, 2, 0, 3, 0, 0, 0]);
isPureGrade(3, pure);  // true (only grade 1)

const mixed = new Float64Array([1, 1, 0, 0, 0, 0, 0, 0]);
isPureGrade(3, mixed);  // false (grade 0 + grade 1)
```

## Component Access

### `componentGet(mv, bladeIndex)` / `component_get(coefficients, blade_index)`

Get a single coefficient by blade index.

```javascript
componentGet(mv, 1);  // coefficient of e1
```

### `componentSet(mv, bladeIndex, value)` / `component_set(coefficients, blade_index, value)`

Set a single coefficient. Returns a new array.

```javascript
const updated = componentSet(mv, 1, 3.14);  // set e1 to 3.14
```

## Norms

### `mvNorm(mv)` / `norm(coefficients)`

Euclidean norm (magnitude) of a multivector.

```javascript
const v = new Float64Array([0, 3, 4, 0, 0, 0, 0, 0]);
mvNorm(v);  // 5
```

### `mvNormSquared(mv)` / `norm_squared(coefficients)`

Squared Euclidean norm (avoids square root).

```javascript
mvNormSquared(v);  // 25
```

### `mvNormalize(mv)` / `normalize(coefficients)`

Normalize to unit length.

```javascript
const unit = mvNormalize(v);  // [0, 0.6, 0.8, 0, 0, 0, 0, 0]
mvNorm(unit);                 // 1.0
```

## Algebraic Transformations

### `mvReverse(dimension, mv)` / `reverse(dimension, coefficients)`

Reversion: grade-dependent sign reversal. Grade `k` gets factor `(-1)^(k(k-1)/2)`.

```javascript
const reversed = mvReverse(3, mv);
```

### `gradeInvolution(dimension, mv)` / `grade_involution(dimension, coefficients)`

Grade involution: negate odd-grade components.

```javascript
const involuted = gradeInvolution(3, mv);
```

## Rust Usage

```rust
use orlando_transducers::geometric_optics::*;

let mv = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

// Grade operations
let vectors = grade_extract(3, 1, &mv);     // [2.0, 3.0, 5.0]
let projected = grade_project(3, 2, &mv);   // bivector projection
let mask = grade_mask(3, &mv);              // bitmask of present grades

// Norms
let n = norm(&mv);
let unit = normalize(&mv);

// Transformations
let rev = reverse(3, &mv);
let inv = grade_involution(3, &mv);
```
