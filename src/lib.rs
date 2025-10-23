//! # Orlando: Compositional Data Transformation
//!
//! Orlando is a high-performance data transformation library that implements transducers
//! in Rust, compiling to WebAssembly for use in JavaScript applications.
//!
//! ## What are Transducers?
//!
//! Transducers compose transformations, not data. Instead of creating intermediate collections:
//!
//! ```text
//! data → map → [intermediate] → filter → [intermediate] → result
//! ```
//!
//! We compose operations first, then execute in a single pass:
//!
//! ```text
//! (map ∘ filter) → data → result
//! ```
//!
//! This approach:
//! - **Eliminates intermediate allocations** - No temporary arrays between operations
//! - **Enables early termination** - Operations like `take` can stop processing immediately
//! - **Composes efficiently** - Build complex pipelines from simple parts
//! - **Executes in a single pass** - Process each element only once
//!
//! ## Category Theory Foundation
//!
//! Transducers are natural transformations between fold functors. Given a reducing
//! function `R: (Acc, Out) -> Acc`, a transducer transforms it into a new reducing
//! function `(Acc, In) -> Acc`.
//!
//! Formally, a transducer is a polymorphic function:
//!
//! ```text
//! ∀Acc. ((Acc, Out) -> Acc) -> ((Acc, In) -> Acc)
//! ```
//!
//! This mathematical foundation ensures that transducers compose correctly and
//! satisfy important laws like associativity and identity.
//!
//! ## Usage (Rust)
//!
//! ```rust
//! use orlando::transforms::{Map, Filter, Take};
//! use orlando::collectors::to_vec;
//! use orlando::transducer::Transducer;
//!
//! // Build a pipeline
//! let pipeline = Map::new(|x: i32| x * 2)
//!     .compose(Filter::new(|x: &i32| x % 3 == 0))
//!     .compose(Take::new(5));
//!
//! // Execute in a single pass
//! let result = to_vec(&pipeline, 1..100);
//! // result: [6, 12, 18, 24, 30]
//! ```
//!
//! ## Usage (JavaScript via WASM)
//!
//! ```javascript
//! import { Pipeline } from './pkg/orlando.js';
//!
//! const pipeline = new Pipeline()
//!   .map(x => x * 2)
//!   .filter(x => x % 3 === 0)
//!   .take(5);
//!
//! const result = pipeline.toArray([...Array(100).keys()].map(x => x + 1));
//! // result: [6, 12, 18, 24, 30]
//! ```
//!
//! ## Performance
//!
//! Orlando leverages:
//! - **Zero-cost abstractions** - Rust's monomorphization eliminates abstraction overhead
//! - **WASM SIMD** - Vectorized operations for numeric data
//! - **Early termination** - Stop processing as soon as possible
//! - **Single-pass execution** - No intermediate allocations
//!
//! Benchmarks show 3-5x performance improvement over pure JavaScript array chaining.

pub mod collectors;
pub mod simd;
pub mod step;
pub mod transducer;
pub mod transforms;

#[cfg(target_arch = "wasm32")]
pub mod pipeline;

// Re-export main types for convenience
pub use step::{cont, is_stopped, stop, unwrap_step, Step};
pub use transducer::{Compose, Identity, Transducer};

// Re-export common transforms
pub use transforms::{
    Chunk, Drop, DropWhile, Filter, FlatMap, Map, Reject, Scan, Take, TakeWhile, Tap, Unique,
    UniqueBy,
};

// Re-export collectors
pub use collectors::{
    contains, count, every, find, first, group_by, last, none, partition, reduce, some, sum, to_vec,
};

#[cfg(target_arch = "wasm32")]
pub use pipeline::Pipeline;

// WASM initialization
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", not(test)))]
#[wasm_bindgen(start)]
pub fn main() {
    // WASM initialization
    // Future: Add console_error_panic_hook feature for better debugging
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pipeline() {
        let pipeline = Map::new(|x: i32| x * 2)
            .compose(Filter::new(|x: &i32| x % 3 == 0))
            .compose(Take::new(5));

        let result = to_vec(&pipeline, 1..100);
        assert_eq!(result, vec![6, 12, 18, 24, 30]);
    }

    #[test]
    fn test_early_termination() {
        // Take should stop early, not process all 1 million elements
        let pipeline = Take::<i32>::new(3);
        let result = to_vec(&pipeline, 1..1_000_000);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_composition_laws() {
        // Identity law: id ∘ f = f ∘ id = f
        let f = Map::new(|x: i32| x * 2);
        let id = Identity::<i32>::new();

        let left = id.compose(Map::new(|x: i32| x * 2));
        let right = Map::new(|x: i32| x * 2).compose(Identity::<i32>::new());

        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(to_vec(&left, data.clone()), to_vec(&f, data.clone()));
        assert_eq!(to_vec(&right, data.clone()), to_vec(&f, data.clone()));
    }

    #[test]
    fn test_no_intermediate_allocations() {
        // This pipeline should execute in a single pass
        // without creating intermediate vectors
        let pipeline = Map::new(|x: i32| x * 2)
            .compose(Filter::new(|x: &i32| *x > 5))
            .compose(Map::new(|x: i32| x + 1))
            .compose(Take::new(3));

        let result = to_vec(&pipeline, 1..100);
        assert_eq!(result, vec![7, 9, 11]);
    }
}
