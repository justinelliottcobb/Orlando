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
    assert_eq!(every(&id, vec![2, 4, 6, 8], |x| x % 2 == 0), true);
    assert_eq!(every(&id, vec![2, 3, 6, 8], |x| x % 2 == 0), false);
}

#[test]
fn test_collectors_some() {
    let id = Identity::<i32>::new();
    assert_eq!(some(&id, vec![1, 3, 4, 5], |x| x % 2 == 0), true);
    assert_eq!(some(&id, vec![1, 3, 5, 7], |x| x % 2 == 0), false);
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
