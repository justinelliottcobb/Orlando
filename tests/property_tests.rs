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
