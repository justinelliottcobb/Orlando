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

// Regression tests for take() state bug
#[wasm_bindgen_test]
fn test_wasm_pipeline_take_with_filter() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    // This is the exact bug reported by the user
    let pipeline = Pipeline::new();
    let map_fn = Function::new_with_args("x", "return x * 30");
    let filter_fn = Function::new_with_args("x", "return x > 10");
    let pipeline = pipeline.map(&map_fn).filter(&filter_fn).take(1);

    let source = Array::new();
    source.push(&(-1).into());
    source.push(&2.into());
    source.push(&3.into());
    source.push(&4.into());
    source.push(&5.into());
    source.push(&(-6).into());
    source.push(&7.into());
    source.push(&8.into());
    source.push(&9.into());
    source.push(&10.into());

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 1, "take(1) should only return 1 element");
    assert_eq!(result.get(0).as_f64(), Some(60.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_take_multiple_with_filter() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let filter_fn = Function::new_with_args("x", "return x > 0");
    let pipeline = pipeline.filter(&filter_fn).take(3);

    let source = Array::new();
    source.push(&(-1).into());
    source.push(&1.into());
    source.push(&2.into());
    source.push(&(-3).into());
    source.push(&3.into());
    source.push(&4.into());
    source.push(&5.into());

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 3);
    assert_eq!(result.get(0).as_f64(), Some(1.0));
    assert_eq!(result.get(1).as_f64(), Some(2.0));
    assert_eq!(result.get(2).as_f64(), Some(3.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_drop_with_filter() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let filter_fn = Function::new_with_args("x", "return x > 0");
    let pipeline = pipeline.filter(&filter_fn).drop(2).take(2);

    let source = Array::new();
    source.push(&(-1).into());
    source.push(&1.into());
    source.push(&(-2).into());
    source.push(&2.into());
    source.push(&3.into());
    source.push(&4.into());
    source.push(&5.into());

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 2);
    assert_eq!(result.get(0).as_f64(), Some(3.0));
    assert_eq!(result.get(1).as_f64(), Some(4.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_take_without_filter() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let map_fn = Function::new_with_args("x", "return x * 2");
    let pipeline = pipeline.map(&map_fn).take(5);

    let source = Array::new();
    for i in 1..=10 {
        source.push(&i.into());
    }

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 5);
    assert_eq!(result.get(0).as_f64(), Some(2.0));
    assert_eq!(result.get(1).as_f64(), Some(4.0));
    assert_eq!(result.get(2).as_f64(), Some(6.0));
    assert_eq!(result.get(3).as_f64(), Some(8.0));
    assert_eq!(result.get(4).as_f64(), Some(10.0));
}

// Comprehensive integration tests for stateful operations
#[wasm_bindgen_test]
fn test_wasm_pipeline_takewhile_with_map() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let map_fn = Function::new_with_args("x", "return x * 2");
    let pred_fn = Function::new_with_args("x", "return x < 20");
    let pipeline = pipeline.map(&map_fn).takeWhile(&pred_fn);

    let source = Array::new();
    for i in 1..=20 {
        source.push(&i.into());
    }

    let result = pipeline.to_array(&source);
    // Should take while x*2 < 20, so x < 10, meaning [2, 4, 6, 8, 10, 12, 14, 16, 18]
    assert_eq!(result.length(), 9);
    assert_eq!(result.get(0).as_f64(), Some(2.0));
    assert_eq!(result.get(8).as_f64(), Some(18.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_dropwhile_with_filter() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let filter_fn = Function::new_with_args("x", "return x % 2 === 0");
    let pred_fn = Function::new_with_args("x", "return x < 10");
    let pipeline = pipeline.filter(&filter_fn).dropWhile(&pred_fn).take(3);

    let source = Array::new();
    for i in 1..=20 {
        source.push(&i.into());
    }

    let result = pipeline.to_array(&source);
    // Even numbers: [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
    // Drop while < 10: [10, 12, 14, 16, 18, 20]
    // Take 3: [10, 12, 14]
    assert_eq!(result.length(), 3);
    assert_eq!(result.get(0).as_f64(), Some(10.0));
    assert_eq!(result.get(1).as_f64(), Some(12.0));
    assert_eq!(result.get(2).as_f64(), Some(14.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_complex_stateful_combination() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let map_fn = Function::new_with_args("x", "return x + 1");
    let filter_fn = Function::new_with_args("x", "return x % 2 === 0");
    let pipeline = pipeline
        .map(&map_fn) // [2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
        .filter(&filter_fn) // [2, 4, 6, 8, 10]
        .drop(1) // [4, 6, 8, 10]
        .take(2); // [4, 6]

    let source = Array::new();
    for i in 1..=10 {
        source.push(&i.into());
    }

    let result = pipeline.to_array(&source);
    assert_eq!(result.length(), 2);
    assert_eq!(result.get(0).as_f64(), Some(4.0));
    assert_eq!(result.get(1).as_f64(), Some(6.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_flatmap_with_take() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    let pipeline = Pipeline::new();
    let flatmap_fn = Function::new_with_args("x", "return [x, x + 1]");
    let pipeline = pipeline.flatMap(&flatmap_fn).take(5);

    let source = Array::new();
    for i in 1..=10 {
        source.push(&i.into());
    }

    let result = pipeline.to_array(&source);
    // flatMap produces: [1, 2, 2, 3, 3, 4, ...]
    // take(5): [1, 2, 2, 3, 3]
    assert_eq!(result.length(), 5);
    assert_eq!(result.get(0).as_f64(), Some(1.0));
    assert_eq!(result.get(1).as_f64(), Some(2.0));
    assert_eq!(result.get(2).as_f64(), Some(2.0));
    assert_eq!(result.get(3).as_f64(), Some(3.0));
    assert_eq!(result.get(4).as_f64(), Some(3.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_multiple_takes() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;

    // This tests that take operations compose correctly
    let pipeline = Pipeline::new();
    let pipeline = pipeline.take(10).take(5);

    let source = Array::new();
    for i in 1..=20 {
        source.push(&i.into());
    }

    let result = pipeline.to_array(&source);
    // The inner take(10) limits to first 10, then take(5) limits to first 5
    assert_eq!(result.length(), 5);
    assert_eq!(result.get(0).as_f64(), Some(1.0));
    assert_eq!(result.get(4).as_f64(), Some(5.0));
}

#[wasm_bindgen_test]
fn test_wasm_pipeline_reduce_with_stateful_ops() {
    use js_sys::{Array, Function};
    use orlando_transducers::Pipeline;
    use wasm_bindgen::JsValue;

    let pipeline = Pipeline::new();
    let filter_fn = Function::new_with_args("x", "return x % 2 === 0");
    let pipeline = pipeline.filter(&filter_fn).take(3);

    let source = Array::new();
    for i in 1..=20 {
        source.push(&i.into());
    }

    let reducer = Function::new_with_args("acc, val", "return acc + val");
    let result = pipeline.reduce(&source, &reducer, JsValue::from(0));

    // Even numbers [2, 4, 6], sum = 12
    assert_eq!(result.as_f64(), Some(12.0));
}
