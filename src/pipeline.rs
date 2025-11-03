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
        let mut state = ProcessState::new();

        for i in 0..source.length() {
            if should_stop {
                break;
            }

            let val = source.get(i);
            let results = self.process_value_with_state(val, &mut state);

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
        let mut state = ProcessState::new();

        for i in 0..source.length() {
            if should_stop {
                break;
            }

            let val = source.get(i);
            let results = self.process_value_with_state(val, &mut state);

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
    fn process_value_with_state(
        &self,
        val: JsValue,
        state: &mut ProcessState,
    ) -> Vec<ProcessResult> {
        self.process_value_from(val, 0, state)
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

// ============================================================================
// Multi-Input Operations (Phase 2a)
// ============================================================================

/// Merge multiple arrays by interleaving their elements in round-robin fashion.
///
/// Takes elements from each array in turn until all arrays are exhausted.
/// If arrays have different lengths, continues with remaining arrays.
///
/// # JavaScript Example
///
/// ```javascript
/// import { merge } from 'orlando-transducers';
///
/// const a = [1, 2, 3];
/// const b = [4, 5, 6];
/// const result = merge([a, b]);
/// // result: [1, 4, 2, 5, 3, 6]
/// ```
#[wasm_bindgen]
pub fn merge(arrays: Array) -> Array {
    let result = Array::new();

    // Convert JS arrays to iterators
    let mut iters: Vec<_> = (0..arrays.length())
        .map(|i| {
            let arr = arrays
                .get(i)
                .dyn_into::<Array>()
                .unwrap_or_else(|_| Array::new());
            (arr, 0)
        })
        .collect();

    let mut active = true;
    while active {
        active = false;
        for (arr, idx) in &mut iters {
            if *idx < arr.length() {
                result.push(&arr.get(*idx));
                *idx += 1;
                active = true;
            }
        }
    }

    result
}

/// Compute the intersection of two arrays (elements in both A and B).
///
/// Returns elements that appear in both arrays, preserving order from the first array.
/// Duplicates from the first array are included if the element exists in the second.
///
/// # JavaScript Example
///
/// ```javascript
/// import { intersection } from 'orlando-transducers';
///
/// const a = [1, 2, 3, 4];
/// const b = [3, 4, 5, 6];
/// const result = intersection(a, b);
/// // result: [3, 4]
/// ```
#[wasm_bindgen]
pub fn intersection(array_a: &Array, array_b: &Array) -> Array {
    use std::collections::HashSet;

    // Build a set from array B for O(1) lookup
    let mut set_b = HashSet::new();
    for i in 0..array_b.length() {
        let val = array_b.get(i);
        // Use JSON stringification for comparison (works for primitives and objects)
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            set_b.insert(json.as_string().unwrap_or_default());
        }
    }

    let result = Array::new();
    for i in 0..array_a.length() {
        let val = array_a.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            if set_b.contains(&json.as_string().unwrap_or_default()) {
                result.push(&val);
            }
        }
    }

    result
}

/// Compute the difference of two arrays (elements in A but not in B).
///
/// Returns elements from the first array that don't appear in the second,
/// preserving order from the first array.
///
/// # JavaScript Example
///
/// ```javascript
/// import { difference } from 'orlando-transducers';
///
/// const a = [1, 2, 3, 4];
/// const b = [3, 4, 5, 6];
/// const result = difference(a, b);
/// // result: [1, 2]
/// ```
#[wasm_bindgen]
pub fn difference(array_a: &Array, array_b: &Array) -> Array {
    use std::collections::HashSet;

    // Build a set from array B for O(1) lookup
    let mut set_b = HashSet::new();
    for i in 0..array_b.length() {
        let val = array_b.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            set_b.insert(json.as_string().unwrap_or_default());
        }
    }

    let result = Array::new();
    for i in 0..array_a.length() {
        let val = array_a.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            if !set_b.contains(&json.as_string().unwrap_or_default()) {
                result.push(&val);
            }
        }
    }

    result
}

/// Compute the union of two arrays (unique elements from both A and B).
///
/// Returns all unique elements that appear in either array.
/// Order is preserved: all unique elements from A first, then unique elements from B.
///
/// # JavaScript Example
///
/// ```javascript
/// import { union } from 'orlando-transducers';
///
/// const a = [1, 2, 3];
/// const b = [3, 4, 5];
/// const result = union(a, b);
/// // result: [1, 2, 3, 4, 5]
/// ```
#[wasm_bindgen]
pub fn union(array_a: &Array, array_b: &Array) -> Array {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let result = Array::new();

    // Add unique elements from A
    for i in 0..array_a.length() {
        let val = array_a.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            if seen.insert(json.as_string().unwrap_or_default()) {
                result.push(&val);
            }
        }
    }

    // Add unique elements from B
    for i in 0..array_b.length() {
        let val = array_b.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            if seen.insert(json.as_string().unwrap_or_default()) {
                result.push(&val);
            }
        }
    }

    result
}

/// Compute the symmetric difference of two arrays (elements in A or B but not both).
///
/// Returns elements that appear in exactly one of the two arrays.
/// Order: unique-to-A elements first, then unique-to-B elements.
///
/// # JavaScript Example
///
/// ```javascript
/// import { symmetricDifference } from 'orlando-transducers';
///
/// const a = [1, 2, 3, 4];
/// const b = [3, 4, 5, 6];
/// const result = symmetricDifference(a, b);
/// // result: [1, 2, 5, 6]
/// ```
#[wasm_bindgen(js_name = symmetricDifference)]
pub fn symmetric_difference(array_a: &Array, array_b: &Array) -> Array {
    use std::collections::HashSet;

    // Build sets from both arrays
    let mut set_a = HashSet::new();
    for i in 0..array_a.length() {
        let val = array_a.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            set_a.insert(json.as_string().unwrap_or_default());
        }
    }

    let mut set_b = HashSet::new();
    for i in 0..array_b.length() {
        let val = array_b.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            set_b.insert(json.as_string().unwrap_or_default());
        }
    }

    let result = Array::new();
    let mut seen = HashSet::new();

    // Elements in A but not B
    for i in 0..array_a.length() {
        let val = array_a.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            let json_str = json.as_string().unwrap_or_default();
            if !set_b.contains(&json_str) && seen.insert(json_str) {
                result.push(&val);
            }
        }
    }

    // Elements in B but not A
    for i in 0..array_b.length() {
        let val = array_b.get(i);
        if let Ok(json) = js_sys::JSON::stringify(&val) {
            let json_str = json.as_string().unwrap_or_default();
            if !set_a.contains(&json_str) && seen.insert(json_str) {
                result.push(&val);
            }
        }
    }

    result
}

// ============================================================================
// Phase 2b: Additional Operations
// ============================================================================

/// Take the last N elements from an array.
///
/// This operation processes the entire array and returns only the last N elements.
/// Unlike `take()`, which stops early, this requires buffering the full result.
///
/// # JavaScript Example
///
/// ```javascript
/// import { takeLast } from 'orlando-transducers';
///
/// const data = [1, 2, 3, 4, 5];
/// const result = takeLast(data, 3);
/// // result: [3, 4, 5]
/// ```
#[wasm_bindgen(js_name = takeLast)]
pub fn take_last(source: &Array, n: u32) -> Array {
    let len = source.length();

    let result = Array::new();

    if n >= len {
        // Return all elements if n is greater than or equal to array length
        for i in 0..len {
            result.push(&source.get(i));
        }
    } else {
        // Return last n elements
        let start = len - n;
        for i in start..len {
            result.push(&source.get(i));
        }
    }

    result
}

/// Drop the last N elements from an array.
///
/// This operation processes the entire array and returns all elements except the last N.
///
/// # JavaScript Example
///
/// ```javascript
/// import { dropLast } from 'orlando-transducers';
///
/// const data = [1, 2, 3, 4, 5];
/// const result = dropLast(data, 2);
/// // result: [1, 2, 3]
/// ```
#[wasm_bindgen(js_name = dropLast)]
pub fn drop_last(source: &Array, n: u32) -> Array {
    let len = source.length();

    let result = Array::new();

    if n >= len {
        // Return empty array if n is greater than or equal to array length
        return result;
    }

    // Return all but last n elements
    let end = len - n;
    for i in 0..end {
        result.push(&source.get(i));
    }

    result
}

/// Create sliding windows of size N over an array.
///
/// Returns an array of arrays, where each sub-array is a window of N consecutive elements.
/// Windows overlap - each window slides by one element.
///
/// # JavaScript Example
///
/// ```javascript
/// import { aperture } from 'orlando-transducers';
///
/// const data = [1, 2, 3, 4, 5];
/// const result = aperture(data, 3);
/// // result: [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
/// ```
#[wasm_bindgen]
pub fn aperture(source: &Array, size: u32) -> Array {
    let len = source.length();
    let result = Array::new();

    if size == 0 || size > len {
        return result;
    }

    // Create sliding windows
    for i in 0..=(len - size) {
        let window = Array::new();
        for j in 0..size {
            window.push(&source.get(i + j));
        }
        result.push(&window);
    }

    result
}
