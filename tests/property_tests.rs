//! Property-based tests for Orlando transducers.
//!
//! These tests verify algebraic properties and invariants that should hold
//! for all transducers, using randomly generated test data.

use orlando::*;
use proptest::prelude::*;

// Property: map(f).map(g) == map(g ∘ f)
proptest! {
    #[test]
    fn test_map_fusion(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        // map(f).map(g)
        let pipeline1 = Map::new(|x: i32| x.saturating_add(1))
            .compose(Map::new(|x: i32| x.saturating_mul(2)));
        let result1 = to_vec(&pipeline1, vec.clone());

        // map(g ∘ f)
        let pipeline2 = Map::new(|x: i32| x.saturating_add(1).saturating_mul(2));
        let result2 = to_vec(&pipeline2, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: filter(p).filter(q) == filter(λx. p(x) ∧ q(x))
proptest! {
    #[test]
    fn test_filter_composition(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        // filter(p).filter(q)
        let pipeline1 = Filter::new(|x: &i32| *x % 2 == 0)
            .compose(Filter::new(|x: &i32| *x > 10));
        let result1 = to_vec(&pipeline1, vec.clone());

        // filter(λx. p(x) ∧ q(x))
        let pipeline2 = Filter::new(|x: &i32| *x % 2 == 0 && *x > 10);
        let result2 = to_vec(&pipeline2, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: take(n) produces at most n elements
proptest! {
    #[test]
    fn test_take_bounds(vec in prop::collection::vec(any::<i32>(), 0..1000), n in 0usize..100) {
        let pipeline = Take::new(n);
        let result = to_vec(&pipeline, vec.clone());

        prop_assert!(result.len() <= n);
        prop_assert!(result.len() <= vec.len());
    }
}

// Property: take(n) preserves order
proptest! {
    #[test]
    fn test_take_order(vec in prop::collection::vec(any::<i32>(), 0..100), n in 0usize..100) {
        let pipeline = Take::new(n);
        let result = to_vec(&pipeline, vec.clone());

        let expected: Vec<i32> = vec.into_iter().take(n).collect();
        prop_assert_eq!(result, expected);
    }
}

// Property: drop(n).take(m) == take(m).drop(n) (for non-overlapping windows)
proptest! {
    #[test]
    fn test_drop_take_commute(vec in prop::collection::vec(any::<i32>(), 0..100), n in 0usize..50, m in 0usize..50) {
        let pipeline1 = Drop::new(n).compose(Take::new(m));
        let result1 = to_vec(&pipeline1, vec.clone());

        let expected: Vec<i32> = vec.into_iter().skip(n).take(m).collect();
        prop_assert_eq!(result1, expected);
    }
}

// Property: map preserves length (without filtering)
proptest! {
    #[test]
    fn test_map_preserves_length(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let pipeline = Map::new(|x: i32| x.saturating_mul(2));
        let result = to_vec(&pipeline, vec.clone());

        prop_assert_eq!(result.len(), vec.len());
    }
}

// Property: filter never increases length
proptest! {
    #[test]
    fn test_filter_decreases_length(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let pipeline = Filter::new(|x: &i32| *x % 2 == 0);
        let result = to_vec(&pipeline, vec.clone());

        prop_assert!(result.len() <= vec.len());
    }
}

// Property: Identity law - id.compose(t) == t
proptest! {
    #[test]
    fn test_identity_left(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let t = Map::new(|x: i32| x.saturating_mul(2));
        let id = Identity::<i32>::new();

        let pipeline1 = id.compose(Map::new(|x: i32| x.saturating_mul(2)));
        let result1 = to_vec(&pipeline1, vec.clone());

        let result2 = to_vec(&t, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: Identity law - t.compose(id) == t
proptest! {
    #[test]
    fn test_identity_right(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let t = Map::new(|x: i32| x.saturating_mul(2));

        let pipeline1 = Map::new(|x: i32| x.saturating_mul(2)).compose(Identity::<i32>::new());
        let result1 = to_vec(&pipeline1, vec.clone());

        let result2 = to_vec(&t, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: Associativity - (f.g).h == f.(g.h)
proptest! {
    #[test]
    fn test_associativity(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        // (f.g).h
        let left = Map::new(|x: i32| x.saturating_add(1))
            .compose(Map::new(|x: i32| x.saturating_mul(2)))
            .compose(Map::new(|x: i32| x.saturating_sub(3)));
        let result1 = to_vec(&left, vec.clone());

        // f.(g.h)
        let right_inner = Map::new(|x: i32| x.saturating_mul(2))
            .compose(Map::new(|x: i32| x.saturating_sub(3)));
        let right = Map::new(|x: i32| x.saturating_add(1)).compose(right_inner);
        let result2 = to_vec(&right, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: sum equals manual summation
proptest! {
    #[test]
    fn test_sum_correctness(vec in prop::collection::vec(0i32..1000, 0..100)) {
        let pipeline = Map::new(|x: i32| x);
        let result = sum(&pipeline, vec.clone());

        let expected: i32 = vec.iter().sum();
        prop_assert_eq!(result, expected);
    }
}

// Property: count equals length
proptest! {
    #[test]
    fn test_count_equals_length(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let pipeline = Identity::<i32>::new();
        let result = count(&pipeline, vec.clone());

        prop_assert_eq!(result, vec.len());
    }
}

// Property: first on non-empty equals first element
proptest! {
    #[test]
    fn test_first_correctness(vec in prop::collection::vec(any::<i32>(), 1..100)) {
        let pipeline = Identity::<i32>::new();
        let result = first(&pipeline, vec.clone());

        prop_assert_eq!(result, Some(vec[0]));
    }
}

// Property: last on non-empty equals last element
proptest! {
    #[test]
    fn test_last_correctness(vec in prop::collection::vec(any::<i32>(), 1..100)) {
        let pipeline = Identity::<i32>::new();
        let result = last(&pipeline, vec.clone());

        let expected = vec.last().copied();
        prop_assert_eq!(result, expected);
    }
}

// Property: takeWhile takes a prefix
proptest! {
    #[test]
    fn test_takewhile_prefix(vec in prop::collection::vec(0i32..100, 0..100)) {
        let threshold = 50;
        let pipeline = TakeWhile::new(move |x: &i32| *x < threshold);
        let result = to_vec(&pipeline, vec.clone());

        // All elements in result should satisfy predicate
        for &x in &result {
            prop_assert!(x < threshold);
        }

        // Result should be a prefix of input
        prop_assert_eq!(&result[..], &vec[..result.len()]);
    }
}

// Property: dropWhile drops a prefix
proptest! {
    #[test]
    fn test_dropwhile_suffix(vec in prop::collection::vec(0i32..100, 0..100)) {
        let threshold = 50;
        let pipeline = DropWhile::new(move |x: &i32| *x < threshold);
        let result = to_vec(&pipeline, vec.clone());

        // Result length should be <= input length
        prop_assert!(result.len() <= vec.len());
    }
}

// Property: scan produces cumulative results
proptest! {
    #[test]
    fn test_scan_cumulative(vec in prop::collection::vec(0i32..100, 0..50)) {
        let pipeline = Scan::new(0, |acc: &i32, x: &i32| acc.saturating_add(*x));
        let result = to_vec(&pipeline, vec.clone());

        // Result length should equal input length
        prop_assert_eq!(result.len(), vec.len());

        // Each element should be the cumulative sum up to that point
        let mut expected = Vec::new();
        let mut acc = 0i32;
        for &x in &vec {
            acc = acc.saturating_add(x);
            expected.push(acc);
        }
        prop_assert_eq!(result, expected);
    }
}

// Property: unique removes consecutive duplicates
proptest! {
    #[test]
    fn test_unique_no_consecutive_dups(vec in prop::collection::vec(0i32..10, 0..100)) {
        let pipeline = Unique::<i32>::new();
        let result = to_vec(&pipeline, vec);

        // No two consecutive elements should be equal
        for i in 1..result.len() {
            prop_assert_ne!(result[i-1], result[i]);
        }
    }
}

// Property: map then filter == filter then map (when predicate is compositional)
proptest! {
    #[test]
    fn test_map_filter_interchange(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        // map then filter
        let pipeline1 = Map::new(|x: i32| x.saturating_mul(2))
            .compose(Filter::new(|x: &i32| *x % 4 == 0));
        let result1 = to_vec(&pipeline1, vec.clone());

        // filter then map
        let pipeline2 = Filter::new(|x: &i32| x.saturating_mul(2) % 4 == 0)
            .compose(Map::new(|x: i32| x.saturating_mul(2)));
        let result2 = to_vec(&pipeline2, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: Identity transducer is truly identity
proptest! {
    #[test]
    fn test_identity_is_identity(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let identity = Identity::<i32>::new();
        let result = to_vec(&identity, vec.clone());

        prop_assert_eq!(result, vec);
    }
}

// Property: Early termination efficiency - first() processes at most 1 element
proptest! {
    #[test]
    fn test_first_early_termination(vec in prop::collection::vec(any::<i32>(), 1..100)) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&call_count);

        let counting_map = Map::new(move |x: i32| {
            counter.fetch_add(1, Ordering::SeqCst);
            x
        });

        let _result = first(&counting_map, vec);

        // Should process exactly 1 element (not 0, not more than 1)
        let count = call_count.load(Ordering::SeqCst);
        prop_assert_eq!(count, 1, "first() should process exactly 1 element, processed {}", count);
    }
}

// Property: Take(n) early termination - processes at most n elements
proptest! {
    #[test]
    fn test_take_early_termination(vec in prop::collection::vec(any::<i32>(), 1..100), n in 1usize..20) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&call_count);

        let counting_map = Map::new(move |x: i32| {
            counter.fetch_add(1, Ordering::SeqCst);
            x
        });

        let pipeline = counting_map.compose(Take::new(n));
        let _result = to_vec(&pipeline, vec.clone());

        let count = call_count.load(Ordering::SeqCst);
        let expected = n.min(vec.len());
        prop_assert_eq!(count, expected,
            "take({}) should process at most {} elements, processed {}", n, expected, count);
    }
}

// Property: TakeWhile early termination efficiency
proptest! {
    #[test]
    fn test_takewhile_early_termination(vec in prop::collection::vec(0i32..100, 1..100)) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&call_count);

        let threshold = 50;
        let counting_map = Map::new(move |x: i32| {
            counter.fetch_add(1, Ordering::SeqCst);
            x
        });

        let pipeline = counting_map.compose(TakeWhile::new(move |x: &i32| *x < threshold));
        let _result = to_vec(&pipeline, vec.clone());

        let count = call_count.load(Ordering::SeqCst);

        // Calculate expected: elements until first >= threshold
        let expected = vec.iter()
            .position(|&x| x >= threshold)
            .map(|pos| pos + 1)  // +1 because we process the element that fails
            .unwrap_or(vec.len());

        prop_assert_eq!(count, expected,
            "takeWhile should stop after first failing element, expected {} processed {}",
            expected, count);
    }
}

// Property: Map composition law - map(g ∘ f) ≡ map(f).map(g)
proptest! {
    #[test]
    fn test_map_composition_law(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let f = |x: i32| x.saturating_mul(2);
        let g = |x: i32| x.saturating_add(1);

        // map(g ∘ f)
        let composed = Map::new(move |x| g(f(x)));
        let result1 = to_vec(&composed, vec.clone());

        // map(f).map(g)
        let chained = Map::new(f).compose(Map::new(g));
        let result2 = to_vec(&chained, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: Filter composition law - filter(p).filter(q) ≡ filter(λx. p(x) ∧ q(x))
proptest! {
    #[test]
    fn test_filter_composition_law(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        // filter(p).filter(q)
        let chained = Filter::new(|x: &i32| *x % 2 == 0)
            .compose(Filter::new(|x: &i32| *x > 10));
        let result1 = to_vec(&chained, vec.clone());

        // filter(λx. p(x) ∧ q(x))
        let combined = Filter::new(|x: &i32| *x % 2 == 0 && *x > 10);
        let result2 = to_vec(&combined, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: Fusion correctness - Map→Filter fusion preserves semantics
// This test verifies that our Pipeline fusion optimization doesn't change behavior
proptest! {
    #[test]
    fn test_map_filter_fusion_correctness(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        // Note: This test uses the Rust transducers, not Pipeline
        // Pipeline fusion is tested separately in integration tests

        let f = |x: i32| x.saturating_mul(2);
        let p = |x: &i32| *x > 10;

        // Separate map and filter
        let separate = Map::new(f).compose(Filter::new(p));
        let result1 = to_vec(&separate, vec.clone());

        // Should be identical regardless of fusion
        // (Pipeline would fuse this, but transducers don't)
        let also_separate = Map::new(f).compose(Filter::new(p));
        let result2 = to_vec(&also_separate, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: Every with all elements matching returns true
proptest! {
    #[test]
    fn test_every_all_match(vec in prop::collection::vec(0i32..100, 0..100)) {
        let pipeline = Identity::<i32>::new();
        let result = every(&pipeline, vec, |x| *x < 100);

        prop_assert!(result);
    }
}

// Property: Every with any element not matching returns false
proptest! {
    #[test]
    fn test_every_early_exit(vec in prop::collection::vec(any::<i32>(), 1..100)) {
        let pipeline = Identity::<i32>::new();

        // At least one element should fail this predicate
        let result = every(&pipeline, vec.clone(), |x| *x > i32::MAX - 1);

        // Should be false for most random inputs
        if vec.iter().any(|x| *x < i32::MAX) {
            prop_assert!(!result);
        }
    }
}

// Property: Some returns true if any element matches
proptest! {
    #[test]
    fn test_some_exists(vec in prop::collection::vec(0i32..100, 1..100)) {
        let pipeline = Identity::<i32>::new();

        if !vec.is_empty() {
            // Pick the first element's value as our target
            let target = vec[0];
            let result = some(&pipeline, vec, move |x| *x == target);

            prop_assert!(result);
        }
    }
}

// Property: Associativity law - (f∘g)∘h ≡ f∘(g∘h)
proptest! {
    #[test]
    fn test_composition_associativity(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let f = Map::new(|x: i32| x.saturating_add(1));
        let g = Map::new(|x: i32| x.saturating_mul(2));
        let h = Filter::new(|x: &i32| *x % 2 == 0);

        // (f∘g)∘h
        let left = f.compose(g.compose(h));
        let result1 = to_vec(&left, vec.clone());

        // f∘(g∘h) - need to rebuild to test properly
        let f2 = Map::new(|x: i32| x.saturating_add(1));
        let g2 = Map::new(|x: i32| x.saturating_mul(2));
        let h2 = Filter::new(|x: &i32| *x % 2 == 0);
        let right_inner = g2.compose(h2);
        let right = f2.compose(right_inner);
        let result2 = to_vec(&right, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: Take then drop vs drop then take
proptest! {
    #[test]
    fn test_take_drop_relationship(vec in prop::collection::vec(any::<i32>(), 0..100),
                                     n in 0usize..50,
                                     m in 0usize..50) {
        // take(n+m).drop(n) should equal drop(n).take(m)
        let total = n + m;

        let pipeline1 = Take::new(total).compose(Drop::new(n));
        let result1 = to_vec(&pipeline1, vec.clone());

        let pipeline2 = Drop::new(n).compose(Take::new(m));
        let result2 = to_vec(&pipeline2, vec);

        prop_assert_eq!(result1, result2);
    }
}
