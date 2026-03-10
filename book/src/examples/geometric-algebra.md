# Geometric Algebra

Orlando provides operations on multivector coefficient arrays for geometric algebra computations. These work on plain `&[f64]` (Rust) or `Float64Array` (JavaScript), making them lightweight and integration-friendly.

## Concepts

In geometric algebra, a **multivector** is represented as an array of coefficients, one for each basis blade. For an algebra with `n` dimensions, there are `2^n` basis blades organized by **grade**:

- **Grade 0**: Scalar (1 blade)
- **Grade 1**: Vectors (`n` blades)
- **Grade 2**: Bivectors (`n choose 2` blades)
- **Grade k**: k-vectors (`n choose k` blades)
- **Grade n**: Pseudoscalar (1 blade)

## JavaScript API

### Grade Inspection

```javascript
import init, {
  bladeGrade,
  bladesAtGradeCount,
  gradeIndices,
  gradeMask,
  hasGrade,
  isPureGrade,
} from 'orlando-transducers';

await init();

// What grade is blade index 3? (index 3 = e12, which is grade 2)
bladeGrade(3);  // 2

// How many bivectors in 3D? (3 choose 2 = 3)
bladesAtGradeCount(3, 2);  // 3

// Which indices hold grade-1 (vector) components in 3D?
gradeIndices(3, 1);  // [1, 2, 4] (e1, e2, e3)
```

### Grade Extraction and Projection

```javascript
import {
  gradeExtract,
  gradeProject,
  gradeProjectMax,
} from 'orlando-transducers';

// 3D algebra: 2^3 = 8 coefficients
// Layout: [scalar, e1, e2, e12, e3, e13, e23, e123]
const mv = new Float64Array([1, 2, 3, 4, 5, 6, 7, 8]);

// Extract just the vector (grade 1) part
const vectors = gradeExtract(3, 1, mv);
// vectors: [2, 3, 5] (coefficients of e1, e2, e3)

// Project onto grade 1 (zero out everything else)
const projected = gradeProject(3, 1, mv);
// projected: [0, 2, 3, 0, 5, 0, 0, 0]

// Project onto grades 0 and 1 (scalar + vector)
const lowGrade = gradeProjectMax(3, 1, mv);
// lowGrade: [1, 2, 3, 0, 5, 0, 0, 0]
```

### Grade Analysis

```javascript
// Which grades have non-zero components?
const mask = gradeMask(3, mv);
// mask is a bitmask: bit k set if grade k is present

// Check specific grade
hasGrade(3, 2, mv);  // true (has bivector components)

// Is this a pure-grade multivector?
isPureGrade(3, mv);  // false (multiple grades present)

const pureVector = new Float64Array([0, 1, 0, 0, 0, 0, 0, 0]);
isPureGrade(3, pureVector);  // true (only grade 1)
```

### Component Access

```javascript
import { componentGet, componentSet } from 'orlando-transducers';

const mv = new Float64Array([0, 0, 0, 0, 0, 0, 0, 0]);

// Set the e1 component (index 1)
const updated = componentSet(mv, 1, 3.14);
// updated: [0, 3.14, 0, 0, 0, 0, 0, 0]

// Get the e1 component
componentGet(updated, 1);  // 3.14
```

### Norms

```javascript
import { mvNorm, mvNormSquared, mvNormalize } from 'orlando-transducers';

const v = new Float64Array([0, 3, 4, 0, 0, 0, 0, 0]);

mvNormSquared(v);   // 25 (3^2 + 4^2)
mvNorm(v);          // 5

const unit = mvNormalize(v);
// unit: [0, 0.6, 0.8, 0, 0, 0, 0, 0]
mvNorm(unit);       // 1.0
```

### Algebraic Transformations

```javascript
import { mvReverse, gradeInvolution } from 'orlando-transducers';

const mv = new Float64Array([1, 2, 3, 4, 5, 6, 7, 8]);

// Reversion: sign flip depends on grade
// grade k gets factor (-1)^(k(k-1)/2)
const reversed = mvReverse(3, mv);

// Grade involution: negate odd grades
const involuted = gradeInvolution(3, mv);
```

## Rust API

All operations work on `&[f64]` coefficient slices:

```rust
use orlando_transducers::geometric_optics::*;

// Grade of basis blade at index 5 (= e13 in 3D, grade 2)
assert_eq!(blade_grade(5), 2);

// Extract vector components from a multivector
let mv = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
let vectors = grade_extract(3, 1, &mv);
assert_eq!(vectors, vec![2.0, 3.0, 5.0]);

// Project onto a single grade
let projected = grade_project(3, 2, &mv);
// Only bivector components survive

// Normalize
let v = vec![0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0];
let unit = normalize(&v);
assert!((norm(&unit) - 1.0).abs() < 1e-10);
```

### Using with Transducer Pipelines

```rust
use orlando_transducers::iter_ext::PipelineBuilder;
use orlando_transducers::geometric_optics::*;

// Process a stream of multivectors: normalize, then extract vector parts
let multivectors: Vec<Vec<f64>> = get_multivectors();

let unit_vectors: Vec<Vec<f64>> = PipelineBuilder::new()
    .map(|mv: Vec<f64>| normalize(&mv))
    .filter(|mv: &Vec<f64>| is_pure_grade(3, mv))
    .map(|mv: Vec<f64>| grade_extract(3, 1, &mv))
    .run(multivectors.into_iter());
```

## API Reference

| Function | Description |
|----------|-------------|
| `bladeGrade(index)` | Grade of a basis blade (popcount of index) |
| `bladesAtGradeCount(dim, grade)` | Number of blades at a grade (binomial coefficient) |
| `gradeIndices(dim, grade)` | Coefficient indices for a grade |
| `gradeExtract(dim, grade, mv)` | Extract coefficients at a grade |
| `gradeProject(dim, grade, mv)` | Zero out all other grades |
| `gradeProjectMax(dim, maxGrade, mv)` | Keep grades up to max |
| `gradeMask(dim, mv)` | Bitmask of present grades |
| `hasGrade(dim, grade, mv)` | Check for non-zero grade |
| `isPureGrade(dim, mv)` | Check single-grade multivector |
| `componentGet(mv, index)` | Get single coefficient |
| `componentSet(mv, index, value)` | Set single coefficient |
| `mvNorm(mv)` | Euclidean norm |
| `mvNormSquared(mv)` | Squared norm |
| `mvNormalize(mv)` | Normalize to unit length |
| `mvReverse(dim, mv)` | Reversion |
| `gradeInvolution(dim, mv)` | Grade involution |
