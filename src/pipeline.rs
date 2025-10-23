//! WASM-compatible pipeline API for JavaScript interop.
//!
//! This module provides a fluent API for building transducer pipelines
//! that can be called from JavaScript via WASM.

use js_sys::{Array, Function, Reflect};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::console;

/// A pipeline represents a composition of transducers that can be applied to data.
///
/// This is the main API for JavaScript consumers.
///
/// # Examples (in JavaScript)
///
/// ```javascript
/// import { Pipeline } from './pkg/orlando.js';
///
/// const pipeline = new Pipeline()
///   .map(x => x * 2)
///   .filter(x => x > 5)
///   .take(3);
///
/// const result = pipeline.toArray([1, 2, 3, 4, 5, 6]);
/// console.log(result); // [6, 8, 10]
/// ```
#[wasm_bindgen]
pub struct Pipeline {
    operations: Vec<Operation>,
}

/// Internal representation of pipeline operations
#[derive(Clone)]
enum Operation {
    Map(Rc<dyn Fn(JsValue) -> JsValue>),
    Filter(Rc<dyn Fn(&JsValue) -> bool>),
    /// Fused Map + Filter for better performance
    MapFilter {
        map: Rc<dyn Fn(JsValue) -> JsValue>,
        filter: Rc<dyn Fn(&JsValue) -> bool>,
    },
    FlatMap(Rc<dyn Fn(JsValue) -> Vec<JsValue>>),
    Take(usize),
    TakeWhile(Rc<dyn Fn(&JsValue) -> bool>),
    Drop(usize),
    DropWhile(Rc<dyn Fn(&JsValue) -> bool>),
    Tap(Rc<dyn Fn(&JsValue)>),
}

#[wasm_bindgen]
impl Pipeline {
    /// Create a new empty pipeline.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Pipeline {
        Pipeline {
            operations: Vec::new(),
        }
    }

    /// Add a map operation to the pipeline.
    ///
    /// # Arguments
    ///
    /// * `f` - A JavaScript function that transforms each value
    #[wasm_bindgen]
    pub fn map(&self, f: &Function) -> Pipeline {
        let f = f.clone();
        let mut ops = self.operations.clone();

        let map_fn = Rc::new(move |val: JsValue| -> JsValue {
            let this = JsValue::null();
            f.call1(&this, &val).unwrap_or(JsValue::undefined())
        }) as Rc<dyn Fn(JsValue) -> JsValue>;

        ops.push(Operation::Map(map_fn));
        Pipeline { operations: ops }
    }

    /// Add a filter operation to the pipeline.
    ///
    /// # Arguments
    ///
    /// * `pred` - A JavaScript function that returns true for values to keep
    #[wasm_bindgen]
    pub fn filter(&self, pred: &Function) -> Pipeline {
        let pred = pred.clone();
        let mut ops = self.operations.clone();

        let filter_fn = Rc::new(move |val: &JsValue| -> bool {
            let this = JsValue::null();
            match pred.call1(&this, val) {
                Ok(result) => result.as_bool().unwrap_or(false),
                Err(_) => false,
            }
        }) as Rc<dyn Fn(&JsValue) -> bool>;

        // OPTIMIZATION: Fuse Map + Filter into a single operation
        // This reduces function call overhead and improves cache locality
        if let Some(Operation::Map(map_fn)) = ops.pop() {
            ops.push(Operation::MapFilter {
                map: map_fn,
                filter: filter_fn,
            });
        } else {
            ops.push(Operation::Filter(filter_fn));
        }

        Pipeline { operations: ops }
    }

    /// Add a flatMap operation to the pipeline.
    ///
    /// Maps each value to an array and flattens the results.
    ///
    /// # Arguments
    ///
    /// * `f` - A JavaScript function that returns an array for each value
    #[wasm_bindgen(js_name = flatMap)]
    pub fn flat_map(&self, f: &Function) -> Pipeline {
        let f = f.clone();
        let mut ops = self.operations.clone();

        let flatmap_fn = Rc::new(move |val: JsValue| -> Vec<JsValue> {
            let this = JsValue::null();
            match f.call1(&this, &val) {
                Ok(result) => {
                    // Convert JsValue array to Vec<JsValue>
                    if let Ok(array) = result.dyn_into::<Array>() {
                        (0..array.length()).map(|i| array.get(i)).collect()
                    } else {
                        vec![]
                    }
                }
                Err(_) => vec![],
            }
        }) as Rc<dyn Fn(JsValue) -> Vec<JsValue>>;

        ops.push(Operation::FlatMap(flatmap_fn));
        Pipeline { operations: ops }
    }

    /// Take the first n elements.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of elements to take
    #[wasm_bindgen]
    pub fn take(&self, n: usize) -> Pipeline {
        let mut ops = self.operations.clone();
        ops.push(Operation::Take(n));
        Pipeline { operations: ops }
    }

    /// Take elements while predicate is true.
    ///
    /// # Arguments
    ///
    /// * `pred` - A JavaScript function that returns true to continue taking
    #[wasm_bindgen(js_name = takeWhile)]
    pub fn take_while(&self, pred: &Function) -> Pipeline {
        let pred = pred.clone();
        let mut ops = self.operations.clone();
        ops.push(Operation::TakeWhile(Rc::new(move |val| {
            let this = JsValue::null();
            match pred.call1(&this, val) {
                Ok(result) => result.as_bool().unwrap_or(false),
                Err(_) => false,
            }
        })));
        Pipeline { operations: ops }
    }

    /// Skip the first n elements.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of elements to skip
    #[wasm_bindgen]
    pub fn drop(&self, n: usize) -> Pipeline {
        let mut ops = self.operations.clone();
        ops.push(Operation::Drop(n));
        Pipeline { operations: ops }
    }

    /// Skip elements while predicate is true.
    ///
    /// # Arguments
    ///
    /// * `pred` - A JavaScript function that returns true to continue skipping
    #[wasm_bindgen(js_name = dropWhile)]
    pub fn drop_while(&self, pred: &Function) -> Pipeline {
        let pred = pred.clone();
        let mut ops = self.operations.clone();
        ops.push(Operation::DropWhile(Rc::new(move |val| {
            let this = JsValue::null();
            match pred.call1(&this, val) {
                Ok(result) => result.as_bool().unwrap_or(false),
                Err(_) => false,
            }
        })));
        Pipeline { operations: ops }
    }

    /// Perform side effects without transforming values.
    ///
    /// # Arguments
    ///
    /// * `f` - A JavaScript function to call for each value
    #[wasm_bindgen]
    pub fn tap(&self, f: &Function) -> Pipeline {
        let f = f.clone();
        let mut ops = self.operations.clone();
        ops.push(Operation::Tap(Rc::new(move |val| {
            let this = JsValue::null();
            let _ = f.call1(&this, val);
        })));
        Pipeline { operations: ops }
    }

    /// Extract a property from each object (JavaScript convenience).
    ///
    /// This is cleaner than `.map(x => x.propertyName)` for extracting properties.
    ///
    /// # Arguments
    ///
    /// * `property_name` - The name of the property to extract
    ///
    /// # Examples (JavaScript)
    ///
    /// ```javascript
    /// const users = [
    ///   { name: 'Alice', age: 30 },
    ///   { name: 'Bob', age: 25 }
    /// ];
    /// const names = new Pipeline().pluck('name').toArray(users);
    /// // names: ['Alice', 'Bob']
    /// ```
    #[wasm_bindgen]
    pub fn pluck(&self, property_name: &str) -> Pipeline {
        let prop_key = JsValue::from_str(property_name);
        let mut ops = self.operations.clone();

        let map_fn = Rc::new(move |val: JsValue| -> JsValue {
            // Use Reflect.get to extract the property
            Reflect::get(&val, &prop_key).unwrap_or(JsValue::undefined())
        }) as Rc<dyn Fn(JsValue) -> JsValue>;

        ops.push(Operation::Map(map_fn));
        Pipeline { operations: ops }
    }

    /// Execute the pipeline and collect results into an array.
    ///
    /// # Arguments
    ///
    /// * `source` - JavaScript array to process
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self, source: &Array) -> Array {
        let result = Array::new();
        let mut should_stop = false;

        for i in 0..source.length() {
            if should_stop {
                break;
            }

            let val = source.get(i);
            let results = self.process_value(val);

            for res in results {
                match res {
                    ProcessResult::Continue(v) => {
                        result.push(&v);
                    }
                    ProcessResult::Skip => {
                        // Skip this value
                    }
                    ProcessResult::Stop(v) => {
                        if let Some(val) = v {
                            result.push(&val);
                        }
                        should_stop = true;
                        break;
                    }
                }
            }
        }

        result
    }

    /// Reduce the source array with a custom reducer function.
    ///
    /// # Arguments
    ///
    /// * `source` - JavaScript array to process
    /// * `reducer` - JavaScript function (acc, val) => acc
    /// * `initial` - Initial accumulator value
    #[wasm_bindgen]
    pub fn reduce(&self, source: &Array, reducer: &Function, initial: JsValue) -> JsValue {
        let mut acc = initial;
        let mut should_stop = false;

        for i in 0..source.length() {
            if should_stop {
                break;
            }

            let val = source.get(i);
            let results = self.process_value(val);

            for res in results {
                match res {
                    ProcessResult::Continue(v) => {
                        let this = JsValue::null();
                        acc = reducer.call2(&this, &acc, &v).unwrap_or(acc);
                    }
                    ProcessResult::Skip => {}
                    ProcessResult::Stop(v) => {
                        if let Some(val) = v {
                            let this = JsValue::null();
                            acc = reducer.call2(&this, &acc, &val).unwrap_or(acc);
                        }
                        should_stop = true;
                        break;
                    }
                }
            }
        }

        acc
    }

    /// Log pipeline execution to console (for debugging).
    #[wasm_bindgen(js_name = logExecution)]
    pub fn log_execution(&self, source: &Array) -> Array {
        console::log_1(&"Pipeline execution:".into());

        let pipeline = self.tap(&Function::new_with_args("x", "console.log('Value:', x)"));

        pipeline.to_array(source)
    }

    // Internal helper to process a single value through the pipeline
    fn process_value(&self, val: JsValue) -> Vec<ProcessResult> {
        self.process_value_from(val, 0, &mut ProcessState::new())
    }

    // Process a value starting from a specific operation index
    #[allow(unused_assignments)]
    fn process_value_from(
        &self,
        mut val: JsValue,
        start_idx: usize,
        state: &mut ProcessState,
    ) -> Vec<ProcessResult> {
        for (idx, op) in self.operations.iter().enumerate().skip(start_idx) {
            match op {
                Operation::Map(f) => {
                    val = f(val);
                }
                Operation::Filter(pred) => {
                    if !pred(&val) {
                        return vec![ProcessResult::Skip];
                    }
                }
                // OPTIMIZED: Fused Map + Filter in single operation
                // This eliminates one function call and one match arm per element
                Operation::MapFilter { map, filter } => {
                    val = map(val);
                    if !filter(&val) {
                        return vec![ProcessResult::Skip];
                    }
                }
                Operation::FlatMap(f) => {
                    // Expand the value into multiple values
                    let expanded = f(val);
                    let mut results = Vec::new();

                    // Process each expanded value through the remaining operations
                    for expanded_val in expanded {
                        let sub_results = self.process_value_from(expanded_val, idx + 1, state);

                        // Check if we should stop early
                        let should_stop = sub_results
                            .iter()
                            .any(|r| matches!(r, ProcessResult::Stop(_)));

                        results.extend(sub_results);

                        if should_stop {
                            break;
                        }
                    }

                    return results;
                }
                Operation::Take(n) => {
                    state.take_count += 1;
                    if state.take_count > *n {
                        return vec![ProcessResult::Stop(None)];
                    }
                }
                Operation::TakeWhile(pred) => {
                    if !pred(&val) {
                        return vec![ProcessResult::Stop(None)];
                    }
                }
                Operation::Drop(n) => {
                    if state.drop_count < *n {
                        state.drop_count += 1;
                        return vec![ProcessResult::Skip];
                    }
                }
                Operation::DropWhile(pred) => {
                    if !state.dropping && pred(&val) {
                        return vec![ProcessResult::Skip];
                    } else {
                        state.dropping = false;
                    }
                }
                Operation::Tap(f) => {
                    f(&val);
                }
            }
        }

        vec![ProcessResult::Continue(val)]
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

enum ProcessResult {
    Continue(JsValue),
    Skip,
    Stop(Option<JsValue>),
}

/// State maintained during pipeline processing
struct ProcessState {
    take_count: usize,
    drop_count: usize,
    dropping: bool,
}

impl ProcessState {
    fn new() -> Self {
        ProcessState {
            take_count: 0,
            drop_count: 0,
            dropping: false,
        }
    }
}

// Export convenience functions

/// Create a new pipeline.
#[wasm_bindgen(js_name = pipeline)]
pub fn create_pipeline() -> Pipeline {
    Pipeline::new()
}
