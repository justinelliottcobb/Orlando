//! WASM-specific tests for Orlando transducers.
//!
//! These tests run only when targeting WASM and verify JavaScript interop.

#![cfg(target_arch = "wasm32")]

use orlando_transducers::*;
use wasm_bindgen_test::*;

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
    use orlando_transducers::step::*;

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
    use orlando_transducers::simd::*;

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

// Pipeline API tests (JavaScript interop)
#[wasm_bindgen_test]
fn test_wasm_pipeline_basic() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();

    // Test that pipeline can be created
    let source = Array::new();
    source.push(&1.into());
    source.push(&2.into());
    source.push(&3.into());

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 3);
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_map() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let map_fn = Function::new_with_args("x", "return x * 2");
    let pipeline = pipeline.map(&map_fn);

    let source = Array::new();
    source.push(&1.into());
    source.push(&2.into());
    source.push(&3.into());

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 3);
    assert_eq!(result.get(0).as_f64(), Some(2.0));
    assert_eq!(result.get(1).as_f64(), Some(4.0));
    assert_eq!(result.get(2).as_f64(), Some(6.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_filter() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let filter_fn = Function::new_with_args("x", "return x % 2 === 0");
    let pipeline = pipeline.filter(&filter_fn);

    let source = Array::new();
    source.push(&1.into());
    source.push(&2.into());
    source.push(&3.into());
    source.push(&4.into());

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 2);
    assert_eq!(result.get(0).as_f64(), Some(2.0));
    assert_eq!(result.get(1).as_f64(), Some(4.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_pluck() {
    use js_sys::{Array, Object, Reflect};
    use orlando_transducers::Pipeline;
    use wasm_bindgen::JsValue;

    let pipeline = Pipeline::new();
    let pipeline = pipeline.pluck("name");

    // Create test objects
    let source = Array::new();

    let obj1 = Object::new();
    Reflect::set(&obj1, &"name".into(), &"Alice".into()).unwrap();
    Reflect::set(&obj1, &"age".into(), &30.into()).unwrap();
    source.push(&obj1);

    let obj2 = Object::new();
    Reflect::set(&obj2, &"name".into(), &"Bob".into()).unwrap();
    Reflect::set(&obj2, &"age".into(), &25.into()).unwrap();
    source.push(&obj2);

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 2);
    assert_eq!(result.get(0).as_string(), Some("Alice".to_string()));
    assert_eq!(result.get(1).as_string(), Some("Bob".to_string()));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_pluck_missing_property() {
    use js_sys::{Array, Object, Reflect};
    use orlando_transducers::Pipeline;
    use wasm_bindgen::JsValue;

    let pipeline = Pipeline::new();
    let pipeline = pipeline.pluck("missing");

    let source = Array::new();
    let obj = Object::new();
    Reflect::set(&obj, &"name".into(), &"Alice".into()).unwrap();
    source.push(&obj);

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 1);
    assert!(result.get(0).is_undefined());
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_pluck_nested() {
    use js_sys::{Array, Object, Reflect};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let pipeline = pipeline.pluck("value");

    let source = Array::new();

    let obj1 = Object::new();
    Reflect::set(&obj1, &"value".into(), &10.into()).unwrap();
    source.push(&obj1);

    let obj2 = Object::new();
    Reflect::set(&obj2, &"value".into(), &20.into()).unwrap();
    source.push(&obj2);

    let obj3 = Object::new();
    Reflect::set(&obj3, &"value".into(), &30.into()).unwrap();
    source.push(&obj3);

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 3);
    assert_eq!(result.get(0).as_f64(), Some(10.0));
    assert_eq!(result.get(1).as_f64(), Some(20.0));
    assert_eq!(result.get(2).as_f64(), Some(30.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_pluck_composition() {
    use js_sys::{Array, Function, Object, Reflect};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let pipeline = pipeline.pluck("age");
    let filter_fn = Function::new_with_args("x", "return x > 25");
    let pipeline = pipeline.filter(&filter_fn);

    let source = Array::new();

    let obj1 = Object::new();
    Reflect::set(&obj1, &"name".into(), &"Alice".into()).unwrap();
    Reflect::set(&obj1, &"age".into(), &30.into()).unwrap();
    source.push(&obj1);

    let obj2 = Object::new();
    Reflect::set(&obj2, &"name".into(), &"Bob".into()).unwrap();
    Reflect::set(&obj2, &"age".into(), &20.into()).unwrap();
    source.push(&obj2);

    let obj3 = Object::new();
    Reflect::set(&obj3, &"name".into(), &"Charlie".into()).unwrap();
    Reflect::set(&obj3, &"age".into(), &28.into()).unwrap();
    source.push(&obj3);

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 2); // Only Alice (30) and Charlie (28)
    assert_eq!(result.get(0).as_f64(), Some(30.0));
    assert_eq!(result.get(1).as_f64(), Some(28.0));
}
