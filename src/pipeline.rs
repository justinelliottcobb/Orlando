//! WASM-compatible pipeline API for JavaScript interop.
//!
//! This module provides a fluent API for building transducer pipelines
//! that can be called from JavaScript via WASM.

use js_sys::{Array, Function};
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

    /// Execute the pipeline and collect results into an array.
    ///
    /// # Arguments
    ///
    /// * `source` - JavaScript array to process
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self, source: &Array) -> Array {
        let result = Array::new();

        for i in 0..source.length() {
            let val = source.get(i);

            match self.process_value(val) {
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
                    break;
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

        for i in 0..source.length() {
            let val = source.get(i);

            match self.process_value(val) {
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
                    break;
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
    #[allow(unused_assignments)]
    fn process_value(&self, mut val: JsValue) -> ProcessResult {
        let mut take_count = 0;
        let mut drop_count = 0;
        let mut dropping = false;

        for op in &self.operations {
            match op {
                Operation::Map(f) => {
                    val = f(val);
                }
                Operation::Filter(pred) => {
                    if !pred(&val) {
                        return ProcessResult::Skip;
                    }
                }
                // OPTIMIZED: Fused Map + Filter in single operation
                // This eliminates one function call and one match arm per element
                Operation::MapFilter { map, filter } => {
                    val = map(val);
                    if !filter(&val) {
                        return ProcessResult::Skip;
                    }
                }
                Operation::Take(n) => {
                    take_count += 1;
                    if take_count > *n {
                        return ProcessResult::Stop(None);
                    }
                }
                Operation::TakeWhile(pred) => {
                    if !pred(&val) {
                        return ProcessResult::Stop(None);
                    }
                }
                Operation::Drop(n) => {
                    if drop_count < *n {
                        drop_count += 1;
                        return ProcessResult::Skip;
                    }
                }
                Operation::DropWhile(pred) => {
                    if !dropping && pred(&val) {
                        return ProcessResult::Skip;
                    } else {
                        dropping = false;
                    }
                }
                Operation::Tap(f) => {
                    f(&val);
                }
            }
        }

        ProcessResult::Continue(val)
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

// Export convenience functions

/// Create a new pipeline.
#[wasm_bindgen(js_name = pipeline)]
pub fn create_pipeline() -> Pipeline {
    Pipeline::new()
}
