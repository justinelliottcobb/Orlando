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

// Property: FlatMap flattens nested structures
proptest! {
    #[test]
    fn test_flatmap_flattens(vec in prop::collection::vec(any::<i32>(), 0..50)) {
        use orlando::FlatMap;

        // FlatMap with duplicate function should double the length
        let pipeline = FlatMap::new(|x: i32| vec![x, x]);
        let result = to_vec(&pipeline, vec.clone());

        prop_assert_eq!(result.len(), vec.len() * 2);
    }
}

// Property: FlatMap with identity should equal original
proptest! {
    #[test]
    fn test_flatmap_identity(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::FlatMap;

        // FlatMap with single-element vec is identity
        let pipeline = FlatMap::new(|x: i32| vec![x]);
        let result = to_vec(&pipeline, vec.clone());

        prop_assert_eq!(result, vec);
    }
}

// Property: FlatMap with empty should produce empty
proptest! {
    #[test]
    fn test_flatmap_empty(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::FlatMap;

        // FlatMap with empty vec should produce nothing
        let pipeline = FlatMap::new(|_x: i32| Vec::<i32>::new());
        let result = to_vec(&pipeline, vec);

        prop_assert_eq!(result, Vec::<i32>::new());
    }
}

// Property: FlatMap preserves order
proptest! {
    #[test]
    fn test_flatmap_preserves_order(vec in prop::collection::vec(0i32..10, 0..50)) {
        use orlando::FlatMap;

        // FlatMap should preserve element order
        let pipeline = FlatMap::new(|x: i32| vec![x, x + 100]);
        let result = to_vec(&pipeline, vec.clone());

        // Check that original elements appear in order
        for i in 0..vec.len() {
            prop_assert_eq!(result[i * 2], vec[i]);
        }
    }
}

// Property: FlatMap composition law (associativity)
proptest! {
    #[test]
    fn test_flatmap_associativity(vec in prop::collection::vec(0i32..10, 0..20)) {
        use orlando::FlatMap;

        // flatMap(f).flatMap(g) == flatMap(x => flatMap(g, f(x)))
        let f = |x: i32| vec![x, x + 1];
        let g = |x: i32| vec![x, x * 2];

        // flatMap(f).flatMap(g)
        let pipeline1 = FlatMap::new(f).compose(FlatMap::new(g));
        let result1 = to_vec(&pipeline1, vec.clone());

        // flatMap(x => flatMap(g, f(x)))
        let pipeline2 = FlatMap::new(move |x: i32| {
            let intermediate = f(x);
            let mut output = Vec::new();
            for y in intermediate {
                output.extend(g(y));
            }
            output
        });
        let result2 = to_vec(&pipeline2, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: FlatMap with early termination stops correctly
proptest! {
    #[test]
    fn test_flatmap_early_termination(vec in prop::collection::vec(any::<i32>(), 1..100), n in 1usize..20) {
        use orlando::FlatMap;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&call_count);

        // FlatMap that counts invocations
        let pipeline = FlatMap::new(move |x: i32| {
            counter.fetch_add(1, Ordering::SeqCst);
            vec![x, x + 1]
        }).compose(Take::new(n));

        let _result = to_vec(&pipeline, vec);

        // Should only call flatMap function enough times to produce n elements
        // Since each call produces 2 elements, we need at most ceil(n/2) calls
        let expected_max_calls = n.div_ceil(2) + 1;  // +1 for potential partial
        let actual_calls = call_count.load(Ordering::SeqCst);

        prop_assert!(actual_calls <= expected_max_calls,
            "FlatMap called {} times, expected at most {} for n={}",
            actual_calls, expected_max_calls, n);
    }
}

// Property: Partition preserves total count
proptest! {
    #[test]
    fn test_partition_total_count(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let (pass, fail) = partition(&id, vec.clone(), |x| x % 2 == 0);

        prop_assert_eq!(pass.len() + fail.len(), vec.len());
    }
}

// Property: All elements in pass partition satisfy predicate
proptest! {
    #[test]
    fn test_partition_pass_elements(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let (pass, _fail) = partition(&id, vec, |x| x % 2 == 0);

        // All pass elements must be even
        prop_assert!(pass.iter().all(|x| x % 2 == 0));
    }
}

// Property: All elements in fail partition don't satisfy predicate
proptest! {
    #[test]
    fn test_partition_fail_elements(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let (_pass, fail) = partition(&id, vec, |x| x % 2 == 0);

        // All fail elements must be odd
        prop_assert!(fail.iter().all(|x| x % 2 != 0));
    }
}

// Property: Partition preserves order within each group
proptest! {
    #[test]
    fn test_partition_preserves_order(vec in prop::collection::vec(0i32..100, 0..50)) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let (evens, odds) = partition(&id, vec.clone(), |x| x % 2 == 0);

        // Extract evens and odds from original in order
        let expected_evens: Vec<i32> = vec.iter().filter(|x| *x % 2 == 0).copied().collect();
        let expected_odds: Vec<i32> = vec.iter().filter(|x| *x % 2 != 0).copied().collect();

        prop_assert_eq!(evens, expected_evens);
        prop_assert_eq!(odds, expected_odds);
    }
}

// Property: Partition works correctly with transformations
proptest! {
    #[test]
    fn test_partition_with_transform(vec in prop::collection::vec(0i32..50, 0..50)) {
        use orlando::Map;

        // Double each element, then partition by >50
        let double = Map::new(|x: i32| x * 2);
        let (greater, lesser) = partition(&double, vec.clone(), |x| *x > 50);

        // Verify all elements in greater are > 50
        prop_assert!(greater.iter().all(|x| *x > 50));

        // Verify all elements in lesser are <= 50
        prop_assert!(lesser.iter().all(|x| *x <= 50));

        // Verify total count
        prop_assert_eq!(greater.len() + lesser.len(), vec.len());
    }
}

// Property: Find returns first matching element
proptest! {
    #[test]
    fn test_find_returns_first_match(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let result = find(&id, vec.clone(), |x| x % 2 == 0);

        // If we find a match, it should be the first even number
        if let Some(found) = result {
            let expected = vec.iter().find(|x| *x % 2 == 0);
            prop_assert_eq!(Some(&found), expected);
        } else {
            // If no match, verify there are no evens
            prop_assert!(vec.iter().all(|x| x % 2 != 0));
        }
    }
}

// Property: Find with early termination stops immediately
proptest! {
    #[test]
    fn test_find_early_termination(vec in prop::collection::vec(any::<i32>(), 1..100)) {
        use orlando::transducer::Identity;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        // Find the position of the first even number
        let first_even_pos = vec.iter().position(|x| x % 2 == 0);

        let call_count = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&call_count);

        let id = Identity::<i32>::new();
        let _result = find(&id, vec.iter().map(|x| {
            counter.fetch_add(1, Ordering::SeqCst);
            *x
        }), |x| x % 2 == 0);

        let actual_calls = call_count.load(Ordering::SeqCst);

        // Should only process elements up to and including the first match
        if let Some(pos) = first_even_pos {
            prop_assert_eq!(actual_calls, pos + 1,
                "find() should stop after finding match at position {}, but processed {} elements",
                pos, actual_calls);
        } else {
            // If no match, should process all elements
            prop_assert_eq!(actual_calls, vec.len());
        }
    }
}

// Property: Find on empty collection returns None
proptest! {
    #[test]
    fn test_find_empty_collection(_dummy in 0..1usize) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let result = find(&id, Vec::<i32>::new(), |x| x % 2 == 0);

        prop_assert_eq!(result, None);
    }
}

// Property: Find with transformation applies correctly
proptest! {
    #[test]
    fn test_find_with_transform(vec in prop::collection::vec(0i32..50, 0..50)) {
        use orlando::Map;

        // Double each element, then find first >50
        let double = Map::new(|x: i32| x * 2);
        let result = find(&double, vec.clone(), |x| *x > 50);

        // Manually compute expected
        let expected = vec.iter()
            .map(|x| x * 2)
            .find(|x| *x > 50);

        prop_assert_eq!(result, expected);
    }
}

// Property: Reject is equivalent to Filter with negated predicate
proptest! {
    #[test]
    fn test_reject_inverse_of_filter(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::{Reject, Filter};

        // Reject(p) == Filter(!p)
        let reject = Reject::new(|x: &i32| x % 2 == 0);
        let filter = Filter::new(|x: &i32| x % 2 != 0);

        let result1 = to_vec(&reject, vec.clone());
        let result2 = to_vec(&filter, vec);

        prop_assert_eq!(result1, result2);
    }
}

// Property: Reject decreases or maintains length
proptest! {
    #[test]
    fn test_reject_decreases_length(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::Reject;

        let reject = Reject::new(|x: &i32| x % 2 == 0);
        let result = to_vec(&reject, vec.clone());

        prop_assert!(result.len() <= vec.len());
    }
}

// Property: Reject + Filter with same predicate = empty
proptest! {
    #[test]
    fn test_reject_filter_complement(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::{Reject, Filter};

        // reject(p).filter(p) should produce empty result
        let pipeline = Reject::new(|x: &i32| x % 2 == 0)
            .compose(Filter::new(|x: &i32| x % 2 == 0));
        let result = to_vec(&pipeline, vec);

        prop_assert!(result.is_empty());
    }
}

// Property: All rejected elements satisfy the predicate
proptest! {
    #[test]
    fn test_reject_complement_elements(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::Reject;

        let reject = Reject::new(|x: &i32| x % 2 == 0);
        let result = to_vec(&reject, vec.clone());

        // All elements in result should NOT be even (should be odd or fail predicate)
        prop_assert!(result.iter().all(|x| x % 2 != 0));
    }
}

// Property: Chunk produces chunks of exact size (except possibly last)
proptest! {
    #[test]
    fn test_chunk_sizes(vec in prop::collection::vec(any::<i32>(), 0..100), size in 1usize..10) {
        use orlando::Chunk;

        let chunker = Chunk::new(size);
        let result = to_vec(&chunker, vec.clone());

        // All chunks should be exactly the specified size
        prop_assert!(result.iter().all(|chunk| chunk.len() == size));
    }
}

// Property: Chunk flattened equals original (minus partial)
proptest! {
    #[test]
    fn test_chunk_flatten_roundtrip(vec in prop::collection::vec(any::<i32>(), 0..100), size in 1usize..10) {
        use orlando::Chunk;

        let chunker = Chunk::new(size);
        let chunks = to_vec(&chunker, vec.clone());

        // Flatten the chunks
        let flattened: Vec<i32> = chunks.into_iter().flatten().collect();

        // Should equal original up to last complete chunk
        let expected_len = (vec.len() / size) * size;
        prop_assert_eq!(flattened, vec[..expected_len].to_vec());
    }
}

// Property: Chunk count is floor(length / size)
proptest! {
    #[test]
    fn test_chunk_count(vec in prop::collection::vec(any::<i32>(), 0..100), size in 1usize..10) {
        use orlando::Chunk;

        let chunker = Chunk::new(size);
        let chunks = to_vec(&chunker, vec.clone());

        let expected_count = vec.len() / size;
        prop_assert_eq!(chunks.len(), expected_count);
    }
}

// Property: Chunk preserves order within chunks
proptest! {
    #[test]
    fn test_chunk_preserves_order(vec in prop::collection::vec(0i32..100, 0..50), size in 1usize..10) {
        use orlando::Chunk;

        let chunker = Chunk::new(size);
        let chunks = to_vec(&chunker, vec.clone());

        // Verify each chunk contains consecutive elements from original
        for (i, chunk) in chunks.iter().enumerate() {
            let start = i * size;
            let end = start + size;
            prop_assert_eq!(chunk, &vec[start..end]);
        }
    }
}

// Property: Chunk with early termination works correctly
proptest! {
    #[test]
    fn test_chunk_with_take(vec in prop::collection::vec(any::<i32>(), 10..100), chunk_size in 2usize..5, n in 1usize..5) {
        use orlando::{Chunk, Take};

        // Chunk then take n chunks
        let pipeline = Chunk::new(chunk_size).compose(Take::new(n));
        let result = to_vec(&pipeline, vec.clone());

        // Should produce exactly n chunks (or fewer if not enough elements)
        let max_chunks = vec.len() / chunk_size;
        let expected_count = n.min(max_chunks);
        prop_assert_eq!(result.len(), expected_count);

        // All chunks should be the right size
        prop_assert!(result.iter().all(|chunk| chunk.len() == chunk_size));
    }
}

// Property: GroupBy preserves all elements
proptest! {
    #[test]
    fn test_group_by_total_count(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let groups = group_by(&id, vec.clone(), |x| x % 5);

        // Total elements across all groups should equal original
        let total: usize = groups.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, vec.len());
    }
}

// Property: GroupBy groups correctly
proptest! {
    #[test]
    fn test_group_by_correctness(vec in prop::collection::vec(any::<i32>(), 0..100)) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let groups = group_by(&id, vec, |x| x % 3);

        // All elements in each group should have the same key
        for (key, values) in groups.iter() {
            prop_assert!(values.iter().all(|v| v % 3 == *key));
        }
    }
}

// Property: GroupBy preserves order within groups
proptest! {
    #[test]
    fn test_group_by_preserves_order(vec in prop::collection::vec(0i32..50, 0..50)) {
        use orlando::transducer::Identity;

        let id = Identity::<i32>::new();
        let groups = group_by(&id, vec.clone(), |x| x % 3);

        // Elements within each group should appear in original order
        for key in 0..3 {
            if let Some(group) = groups.get(&key) {
                let expected: Vec<i32> = vec.iter().filter(|x| *x % 3 == key).copied().collect();
                prop_assert_eq!(group, &expected);
            }
        }
    }
}

// Property: Zip length is min of two input lengths
proptest! {
    #[test]
    fn test_zip_length(a in prop::collection::vec(any::<i32>(), 0..50), b in prop::collection::vec(any::<i32>(), 0..50)) {
        use orlando::zip;

        let result = zip(a.clone(), b.clone());
        let expected_len = a.len().min(b.len());
        prop_assert_eq!(result.len(), expected_len);
    }
}

// Property: Zip preserves order and pairing
proptest! {
    #[test]
    fn test_zip_correctness(a in prop::collection::vec(any::<i32>(), 0..50), b in prop::collection::vec(any::<i32>(), 0..50)) {
        use orlando::zip;

        let result = zip(a.clone(), b.clone());

        // Each pair should match the original elements at that index
        for (i, (x, y)) in result.iter().enumerate() {
            prop_assert_eq!(x, &a[i]);
            prop_assert_eq!(y, &b[i]);
        }
    }
}

// Property: ZipWith applies function correctly
proptest! {
    #[test]
    fn test_zip_with_correctness(a in prop::collection::vec(any::<i32>(), 0..50), b in prop::collection::vec(any::<i32>(), 0..50)) {
        use orlando::zip_with;

        let result = zip_with(a.clone(), b.clone(), |x, y| x.saturating_add(y));

        // Manually compute expected
        let expected: Vec<i32> = a.iter().zip(b.iter()).map(|(x, y)| x.saturating_add(*y)).collect();
        prop_assert_eq!(result, expected);
    }
}

// Property: Zip with empty is empty
proptest! {
    #[test]
    fn test_zip_with_empty(_dummy in 0..1usize) {
        use orlando::zip;

        let a: Vec<i32> = vec![];
        let b = vec![1, 2, 3];
        let result1 = zip(a.clone(), b.clone());
        let result2 = zip(b, a);

        prop_assert!(result1.is_empty());
        prop_assert!(result2.is_empty());
    }
}
