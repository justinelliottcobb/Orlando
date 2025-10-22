//! WASM-specific tests for Orlando transducers.
//!
//! These tests run only when targeting WASM and verify JavaScript interop.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use orlando::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_basic_pipeline() {
    let pipeline = Map::new(|x: i32| x * 2)
        .compose(Filter::new(|x: &i32| *x % 3 == 0))
        .compose(Take::new(5));

    let result = to_vec(&pipeline, 1..100);
    assert_eq!(result, vec![6, 12, 18, 24, 30]);
}

#[wasm_bindgen_test]
fn test_wasm_early_termination() {
    let pipeline = Take::<i32>::new(10);
    let result = to_vec(&pipeline, 1..1_000_000);
    
    assert_eq!(result.len(), 10);
    assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[wasm_bindgen_test]
fn test_wasm_collectors_sum() {
    let pipeline = Map::new(|x: i32| x * 2);
    let result = sum(&pipeline, vec![1, 2, 3, 4, 5]);
    assert_eq!(result, 30);
}

#[wasm_bindgen_test]
fn test_wasm_collectors_count() {
    let pipeline = Filter::new(|x: &i32| *x % 2 == 0);
    let result = count(&pipeline, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert_eq!(result, 5);
}

#[wasm_bindgen_test]
fn test_wasm_unique() {
    let pipeline = Unique::<i32>::new();
    let result = to_vec(&pipeline, vec![1, 1, 2, 2, 3, 3, 2, 1]);
    assert_eq!(result, vec![1, 2, 3, 2, 1]);
}

#[wasm_bindgen_test]
fn test_wasm_scan() {
    let pipeline = Scan::new(0, |acc: &i32, x: &i32| acc + x);
    let result = to_vec(&pipeline, vec![1, 2, 3, 4, 5]);
    assert_eq!(result, vec![1, 3, 6, 10, 15]);
}

#[wasm_bindgen_test]
fn test_wasm_complex_pipeline() {
    let pipeline = Map::new(|x: i32| x + 1)
        .compose(Filter::new(|x: &i32| *x % 2 == 0))
        .compose(Map::new(|x: i32| x * 3))
        .compose(Take::new(5));

    let result = to_vec(&pipeline, 0..100);
    assert_eq!(result.len(), 5);
}

#[wasm_bindgen_test]
fn test_wasm_identity_laws() {
    let f = Map::new(|x: i32| x * 2);
    let id = Identity::<i32>::new();
    let data = vec![1, 2, 3, 4, 5];

    // id ∘ f = f
    let left = id.compose(Map::new(|x: i32| x * 2));
    assert_eq!(to_vec(&left, data.clone()), to_vec(&f, data.clone()));

    // f ∘ id = f
    let right = Map::new(|x: i32| x * 2).compose(Identity::<i32>::new());
    assert_eq!(to_vec(&right, data.clone()), to_vec(&f, data));
}

#[wasm_bindgen_test]
fn test_wasm_step_monad() {
    use orlando::step::*;

    let c = cont(42);
    assert!(c.is_continue());
    assert!(!c.is_stop());
    assert_eq!(c.unwrap(), 42);

    let s = stop(42);
    assert!(s.is_stop());
    assert!(!s.is_continue());
    assert_eq!(s.unwrap(), 42);
}

#[wasm_bindgen_test]
fn test_wasm_simd_operations() {
    use orlando::simd::*;

    let data = vec![1.0, 2.0, 3.0, 4.0];
    
    // map_f64_simd
    let result = map_f64_simd(&data, |x| x * 2.0);
    assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0]);

    // sum_f64_simd
    let sum = sum_f64_simd(&data);
    assert_eq!(sum, 10.0);

    // mul_f64_simd
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let b = vec![2.0, 3.0, 4.0, 5.0];
    let result = mul_f64_simd(&a, &b);
    assert_eq!(result, vec![2.0, 6.0, 12.0, 20.0]);
}
