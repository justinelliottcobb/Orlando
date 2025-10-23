//! Integration tests for Orlando transducers.

use orlando::*;

#[test]
fn test_map_filter_take_pipeline() {
    let pipeline = Map::new(|x: i32| x * 2)
        .compose(Filter::new(|x: &i32| *x % 3 == 0))
        .compose(Take::new(5));

    let result = to_vec(&pipeline, 1..100);
    assert_eq!(result, vec![6, 12, 18, 24, 30]);
}

#[test]
fn test_complex_pipeline() {
    // Build a complex pipeline with 10+ operations
    let pipeline = Map::new(|x: i32| x + 1)
        .compose(Filter::new(|x: &i32| *x % 2 == 0))
        .compose(Map::new(|x: i32| x * 3))
        .compose(Filter::new(|x: &i32| *x > 10))
        .compose(Take::new(5))
        .compose(Map::new(|x: i32| x - 1));

    let result = to_vec(&pipeline, 0..100);
    assert_eq!(result, vec![11, 17, 23, 29, 35]);
}

#[test]
fn test_take_while() {
    let pipeline = TakeWhile::new(|x: &i32| *x < 10);
    let result = to_vec(&pipeline, 1..100);
    assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_drop() {
    let pipeline = Drop::new(5);
    let result = to_vec(&pipeline, 1..11);
    assert_eq!(result, vec![6, 7, 8, 9, 10]);
}

#[test]
fn test_drop_while() {
    let pipeline = DropWhile::new(|x: &i32| *x < 5);
    let result = to_vec(&pipeline, 1..11);
    assert_eq!(result, vec![5, 6, 7, 8, 9, 10]);
}

#[test]
fn test_unique() {
    let pipeline = Unique::<i32>::new();
    let result = to_vec(&pipeline, vec![1, 1, 2, 2, 3, 3, 2, 1]);
    assert_eq!(result, vec![1, 2, 3, 2, 1]);
}

#[test]
fn test_unique_by() {
    let pipeline = UniqueBy::new(|x: &i32| x.abs());
    let result = to_vec(&pipeline, vec![1, -1, 2, -2, 3, -3]);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_scan() {
    // Running sum
    let pipeline = Scan::new(0, |acc: &i32, x: &i32| acc + x);
    let result = to_vec(&pipeline, vec![1, 2, 3, 4, 5]);
    assert_eq!(result, vec![1, 3, 6, 10, 15]);
}

#[test]
fn test_tap_side_effects() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let logged = Rc::new(RefCell::new(Vec::new()));
    let logged_clone = Rc::clone(&logged);

    let pipeline = Tap::new(move |x: &i32| {
        logged_clone.borrow_mut().push(*x);
    })
    .compose(Filter::new(|x: &i32| *x % 2 == 0));

    let result = to_vec(&pipeline, vec![1, 2, 3, 4, 5]);

    assert_eq!(result, vec![2, 4]);
    assert_eq!(*logged.borrow(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_collectors_sum() {
    let pipeline = Map::new(|x: i32| x * 2);
    let result = sum(&pipeline, vec![1, 2, 3, 4, 5]);
    assert_eq!(result, 30);
}

#[test]
fn test_collectors_count() {
    let pipeline = Filter::new(|x: &i32| *x % 2 == 0);
    let result = count(&pipeline, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert_eq!(result, 5);
}

#[test]
fn test_collectors_first() {
    let pipeline = Filter::new(|x: &i32| *x % 5 == 0);
    let result = first(&pipeline, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert_eq!(result, Some(5));
}

#[test]
fn test_collectors_last() {
    let pipeline = Filter::new(|x: &i32| *x % 2 == 0);
    let result = last(&pipeline, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert_eq!(result, Some(10));
}

#[test]
fn test_collectors_every() {
    let id = Identity::<i32>::new();
    assert!(every(&id, vec![2, 4, 6, 8], |x| x % 2 == 0));
    assert!(!every(&id, vec![2, 3, 6, 8], |x| x % 2 == 0));
}

#[test]
fn test_collectors_some() {
    let id = Identity::<i32>::new();
    assert!(some(&id, vec![1, 3, 4, 5], |x| x % 2 == 0));
    assert!(!some(&id, vec![1, 3, 5, 7], |x| x % 2 == 0));
}

#[test]
fn test_early_termination_efficiency() {
    // This should only process 10 elements, not 1 million
    use std::cell::RefCell;
    use std::rc::Rc;

    let processed = Rc::new(RefCell::new(0));
    let processed_clone = Rc::clone(&processed);

    let pipeline = Tap::new(move |_: &i32| {
        *processed_clone.borrow_mut() += 1;
    })
    .compose(Take::new(10));

    let result = to_vec(&pipeline, 1..1_000_000);

    assert_eq!(result.len(), 10);
    assert_eq!(*processed.borrow(), 10); // Should only process 10 elements
}

#[test]
fn test_large_dataset() {
    let pipeline = Map::new(|x: i32| x * 2)
        .compose(Filter::new(|x: &i32| *x % 7 == 0))
        .compose(Take::new(100));

    let result = to_vec(&pipeline, 1..1_000_000);
    assert_eq!(result.len(), 100);
    assert_eq!(result[0], 14);
    assert_eq!(result[99], 1400);
}

#[test]
fn test_composition_associativity() {
    // (f ∘ g) ∘ h = f ∘ (g ∘ h)
    let f = Map::new(|x: i32| x + 1);
    let g = Map::new(|x: i32| x * 2);
    let h = Map::new(|x: i32| x - 3);

    let left = f.compose(g).compose(h);
    let right_inner = Map::new(|x: i32| x * 2).compose(Map::new(|x: i32| x - 3));
    let right = Map::new(|x: i32| x + 1).compose(right_inner);

    let data = vec![1, 2, 3, 4, 5];
    assert_eq!(to_vec(&left, data.clone()), to_vec(&right, data));
}

#[test]
fn test_identity_laws() {
    let f = Map::new(|x: i32| x * 2);
    let id = Identity::<i32>::new();

    let data = vec![1, 2, 3, 4, 5];

    // id ∘ f = f
    let left = id.compose(Map::new(|x: i32| x * 2));
    assert_eq!(to_vec(&left, data.clone()), to_vec(&f, data.clone()));

    // f ∘ id = f
    let right = Map::new(|x: i32| x * 2).compose(Identity::<i32>::new());
    assert_eq!(to_vec(&right, data.clone()), to_vec(&f, data.clone()));
}

// ========================================
// Phase 2b Advanced Operations Tests
// ========================================

#[test]
fn test_interpose() {
    let pipeline = Interpose::new(0);
    let result = to_vec(&pipeline, vec![1, 2, 3, 4]);
    assert_eq!(result, vec![1, 0, 2, 0, 3, 0, 4]);
}

#[test]
fn test_interpose_empty() {
    let pipeline = Interpose::new(0);
    let result = to_vec(&pipeline, Vec::<i32>::new());
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_interpose_single() {
    let pipeline = Interpose::new(0);
    let result = to_vec(&pipeline, vec![1]);
    assert_eq!(result, vec![1]);
}

#[test]
fn test_interpose_with_strings() {
    let pipeline = Interpose::new(",".to_string());
    let result = to_vec(
        &pipeline,
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
    );
    assert_eq!(
        result,
        vec![
            "a".to_string(),
            ",".to_string(),
            "b".to_string(),
            ",".to_string(),
            "c".to_string()
        ]
    );
}

#[test]
fn test_interpose_composition() {
    let pipeline = Map::new(|x: i32| x * 2).compose(Interpose::new(0));
    let result = to_vec(&pipeline, vec![1, 2, 3]);
    assert_eq!(result, vec![2, 0, 4, 0, 6]);
}

#[test]
fn test_repeat_each() {
    let pipeline = RepeatEach::new(3);
    let result = to_vec(&pipeline, vec![1, 2, 3]);
    assert_eq!(result, vec![1, 1, 1, 2, 2, 2, 3, 3, 3]);
}

#[test]
fn test_repeat_each_zero() {
    let pipeline = RepeatEach::new(0);
    let result = to_vec(&pipeline, vec![1, 2, 3]);
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_repeat_each_one() {
    let pipeline = RepeatEach::new(1);
    let result = to_vec(&pipeline, vec![1, 2, 3]);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_repeat_each_composition() {
    let pipeline = Map::new(|x: i32| x * 2).compose(RepeatEach::new(2));
    let result = to_vec(&pipeline, vec![1, 2, 3]);
    assert_eq!(result, vec![2, 2, 4, 4, 6, 6]);
}

#[test]
fn test_repeat_each_with_take() {
    // RepeatEach should respect early termination
    let pipeline = RepeatEach::new(3).compose(Take::new(5));
    let result = to_vec(&pipeline, vec![1, 2, 3]);
    assert_eq!(result, vec![1, 1, 1, 2, 2]); // Stops after 5 elements
}

#[test]
fn test_partition_by() {
    let id = Identity::new();
    let data = vec![1, 1, 2, 2, 3, 1, 1];
    let groups = partition_by(&id, data, |x| *x);

    assert_eq!(groups.len(), 4);
    assert_eq!(groups[0], vec![1, 1]);
    assert_eq!(groups[1], vec![2, 2]);
    assert_eq!(groups[2], vec![3]);
    assert_eq!(groups[3], vec![1, 1]);
}

#[test]
fn test_partition_by_empty() {
    let id = Identity::new();
    let data: Vec<i32> = vec![];
    let groups = partition_by(&id, data, |x| *x);
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_partition_by_single_group() {
    let id = Identity::new();
    let data = vec![1, 1, 1, 1];
    let groups = partition_by(&id, data, |x| *x);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0], vec![1, 1, 1, 1]);
}

#[test]
fn test_partition_by_with_transform() {
    let pipeline = Map::new(|x: i32| x % 3);
    let data = vec![3, 6, 9, 1, 4, 7, 2, 5];
    let groups = partition_by(&pipeline, data, |x| *x);

    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0], vec![0, 0, 0]); // 3%3, 6%3, 9%3
    assert_eq!(groups[1], vec![1, 1, 1]); // 1%3, 4%3, 7%3
    assert_eq!(groups[2], vec![2, 2]); // 2%3, 5%3
}

#[test]
fn test_top_k() {
    let id = Identity::new();
    let data = vec![1, 5, 2, 8, 3, 9, 4, 6, 7];
    let top3 = top_k(&id, data, 3);

    assert_eq!(top3.len(), 3);
    assert_eq!(top3, vec![9, 8, 7]); // In descending order
}

#[test]
fn test_top_k_k_greater_than_n() {
    let id = Identity::new();
    let data = vec![1, 2, 3];
    let top10 = top_k(&id, data, 10);

    assert_eq!(top10.len(), 3);
    assert_eq!(top10, vec![3, 2, 1]);
}

#[test]
fn test_top_k_empty() {
    let id = Identity::new();
    let data: Vec<i32> = vec![];
    let top3 = top_k(&id, data, 3);
    assert_eq!(top3.len(), 0);
}

#[test]
fn test_top_k_with_transform() {
    let pipeline = Map::new(|x: i32| x * 2);
    let data = vec![1, 5, 2, 8, 3];
    let top3 = top_k(&pipeline, data, 3);

    assert_eq!(top3.len(), 3);
    assert_eq!(top3, vec![16, 10, 6]); // Top 3 after doubling
}

#[test]
fn test_frequencies() {
    let id = Identity::new();
    let data = vec![1, 2, 3, 1, 2, 1];
    let freqs = frequencies(&id, data);

    assert_eq!(freqs.get(&1), Some(&3));
    assert_eq!(freqs.get(&2), Some(&2));
    assert_eq!(freqs.get(&3), Some(&1));
}

#[test]
fn test_frequencies_empty() {
    let id = Identity::new();
    let data: Vec<i32> = vec![];
    let freqs = frequencies(&id, data);
    assert_eq!(freqs.len(), 0);
}

#[test]
fn test_frequencies_with_transform() {
    let pipeline = Map::new(|x: i32| x % 3);
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let freqs = frequencies(&pipeline, data);

    assert_eq!(freqs.get(&0), Some(&3)); // 3, 6, 9
    assert_eq!(freqs.get(&1), Some(&3)); // 1, 4, 7
    assert_eq!(freqs.get(&2), Some(&3)); // 2, 5, 8
}

#[test]
fn test_zip_longest() {
    let a = vec![1, 2, 3];
    let b = vec![4, 5];
    let result = zip_longest(a, b, 0, 0);

    assert_eq!(result, vec![(1, 4), (2, 5), (3, 0)]);
}

#[test]
fn test_zip_longest_equal_length() {
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let result = zip_longest(a, b, 0, 0);

    assert_eq!(result, vec![(1, 4), (2, 5), (3, 6)]);
}

#[test]
fn test_zip_longest_first_longer() {
    let a = vec![1, 2, 3, 4];
    let b = vec![5, 6];
    let result = zip_longest(a, b, -1, -1);

    assert_eq!(result, vec![(1, 5), (2, 6), (3, -1), (4, -1)]);
}

#[test]
fn test_zip_longest_empty() {
    let a: Vec<i32> = vec![];
    let b: Vec<i32> = vec![];
    let result = zip_longest(a, b, 0, 0);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_cartesian_product() {
    let a = vec![1, 2];
    let b = vec![3, 4, 5];
    let result = cartesian_product(a, b);

    assert_eq!(result.len(), 6);
    assert!(result.contains(&(1, 3)));
    assert!(result.contains(&(1, 4)));
    assert!(result.contains(&(1, 5)));
    assert!(result.contains(&(2, 3)));
    assert!(result.contains(&(2, 4)));
    assert!(result.contains(&(2, 5)));
}

#[test]
fn test_cartesian_product_empty() {
    let a: Vec<i32> = vec![];
    let b = vec![1, 2, 3];
    let result = cartesian_product(a, b);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_cartesian_product_single() {
    let a = vec![1];
    let b = vec![2];
    let result = cartesian_product(a, b);
    assert_eq!(result, vec![(1, 2)]);
}

#[test]
fn test_reservoir_sample() {
    let id = Identity::new();
    let data: Vec<i32> = (1..=100).collect();
    let sample = reservoir_sample(&id, data.clone(), 10);

    // Verify size
    assert_eq!(sample.len(), 10);

    // Verify all sampled values are from the original data
    for value in &sample {
        assert!(data.contains(value));
    }
}

#[test]
fn test_reservoir_sample_k_greater_than_n() {
    let id = Identity::new();
    let data = vec![1, 2, 3];
    let sample = reservoir_sample(&id, data.clone(), 10);

    // Should return all elements when k > n
    assert_eq!(sample.len(), 3);
    assert!(sample.contains(&1));
    assert!(sample.contains(&2));
    assert!(sample.contains(&3));
}

#[test]
fn test_reservoir_sample_empty() {
    let id = Identity::new();
    let data: Vec<i32> = vec![];
    let sample = reservoir_sample(&id, data, 10);
    assert_eq!(sample.len(), 0);
}

#[test]
fn test_reservoir_sample_with_transform() {
    let pipeline = Map::new(|x: i32| x * 2);
    let data: Vec<i32> = (1..=50).collect();
    let sample = reservoir_sample(&pipeline, data, 5);

    // Verify size
    assert_eq!(sample.len(), 5);

    // Verify all values are even (from the doubling transform)
    for value in &sample {
        assert_eq!(value % 2, 0);
    }
}
