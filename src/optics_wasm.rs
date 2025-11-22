//! WASM-compatible optics API for JavaScript interop.
//!
//! This module provides JavaScript bindings for functional lenses,
//! allowing type-safe access and updates to nested data structures.

use js_sys::{Function, Object, Reflect};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

// Type aliases to satisfy clippy type_complexity lint
type JsGetter = Rc<dyn Fn(&JsValue) -> JsValue>;
type JsSetter = Rc<dyn Fn(&JsValue, JsValue) -> JsValue>;

/// A Lens provides focused access to a part of a JavaScript object.
///
/// Lenses enable immutable updates to nested data structures in a composable way.
///
/// # Examples (in JavaScript)
///
/// ```javascript
/// import { lens } from './pkg/orlando.js';
///
/// const user = { name: "Alice", age: 30 };
///
/// const nameLens = lens('name');
/// console.log(nameLens.get(user));           // "Alice"
///
/// const updated = nameLens.set(user, "Bob");
/// console.log(updated.name);                 // "Bob"
/// console.log(user.name);                    // "Alice" (unchanged)
///
/// const upper = nameLens.over(user, s => s.toUpperCase());
/// console.log(upper.name);                   // "ALICE"
/// ```
#[wasm_bindgen]
pub struct JsLens {
    get_fn: JsGetter,
    set_fn: JsSetter,
}

#[wasm_bindgen]
impl JsLens {
    /// Extract the focused value from the source object.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript object to extract from
    ///
    /// # Returns
    ///
    /// The focused value
    #[wasm_bindgen]
    pub fn get(&self, source: &JsValue) -> JsValue {
        (self.get_fn)(source)
    }

    /// Update the focused value immutably, returning a new object.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript object to update
    /// * `value` - The new value to set
    ///
    /// # Returns
    ///
    /// A new object with the updated value
    #[wasm_bindgen]
    pub fn set(&self, source: &JsValue, value: JsValue) -> JsValue {
        (self.set_fn)(source, value)
    }

    /// Transform the focused value using a function.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript object to transform
    /// * `f` - A JavaScript function to apply to the focused value
    ///
    /// # Returns
    ///
    /// A new object with the transformed value
    #[wasm_bindgen]
    pub fn over(&self, source: &JsValue, f: &Function) -> JsValue {
        let current = self.get(source);
        let this = JsValue::null();
        let updated = f.call1(&this, &current).unwrap_or_else(|_| current.clone());
        self.set(source, updated)
    }

    /// Compose two lenses to focus deeper into nested structures.
    ///
    /// # Arguments
    ///
    /// * `other` - The lens to compose with
    ///
    /// # Returns
    ///
    /// A new lens that focuses on the composition
    #[wasm_bindgen]
    pub fn compose(&self, other: &JsLens) -> JsLens {
        let self_get = self.get_fn.clone();
        let self_set = self.set_fn.clone();
        let other_get = other.get_fn.clone();
        let other_set = other.set_fn.clone();

        // Clone for the setter closure
        let self_get_2 = self_get.clone();

        JsLens {
            get_fn: Rc::new(move |source: &JsValue| {
                let intermediate = self_get(source);
                other_get(&intermediate)
            }),
            set_fn: Rc::new(move |source: &JsValue, value: JsValue| {
                let intermediate = self_get_2(source);
                let updated_intermediate = other_set(&intermediate, value);
                self_set(source, updated_intermediate)
            }),
        }
    }
}

/// Create a lens that focuses on a property of an object.
///
/// # Arguments
///
/// * `prop` - The property name to focus on
///
/// # Returns
///
/// A lens that focuses on the specified property
///
/// # Examples
///
/// ```javascript
/// const nameLens = lens('name');
/// const user = { name: "Alice", age: 30 };
/// console.log(nameLens.get(user)); // "Alice"
/// ```
#[wasm_bindgen]
pub fn lens(prop: &str) -> JsLens {
    let prop_get = prop.to_string();
    let prop_set = prop.to_string();

    JsLens {
        get_fn: Rc::new(move |source: &JsValue| {
            if let Some(obj) = source.dyn_ref::<Object>() {
                Reflect::get(obj, &JsValue::from_str(&prop_get)).unwrap_or(JsValue::undefined())
            } else {
                JsValue::undefined()
            }
        }),
        set_fn: Rc::new(move |source: &JsValue, value: JsValue| {
            if let Some(obj) = source.dyn_ref::<Object>() {
                // Clone the object
                let new_obj = Object::assign(&Object::new(), obj);
                // Set the property
                let _ = Reflect::set(&new_obj, &JsValue::from_str(&prop_set), &value);
                new_obj.into()
            } else {
                source.clone()
            }
        }),
    }
}

/// Create a lens that focuses on a nested path in an object.
///
/// # Arguments
///
/// * `path` - Array of property names representing the path
///
/// # Returns
///
/// A lens that focuses on the nested value
///
/// # Examples
///
/// ```javascript
/// const cityLens = lensPath(['address', 'city']);
/// const user = { name: "Alice", address: { city: "NYC", zip: "10001" } };
/// console.log(cityLens.get(user)); // "NYC"
/// ```
#[wasm_bindgen(js_name = lensPath)]
pub fn lens_path(path: &JsValue) -> Result<JsLens, JsValue> {
    let arr = path
        .dyn_ref::<js_sys::Array>()
        .ok_or_else(|| JsValue::from_str("path must be an array"))?;

    if arr.length() == 0 {
        return Err(JsValue::from_str("path cannot be empty"));
    }

    // Start with the first property lens
    let first_prop = arr
        .get(0)
        .as_string()
        .ok_or_else(|| JsValue::from_str("path elements must be strings"))?;

    let mut result = lens(&first_prop);

    // Compose with remaining properties
    for i in 1..arr.length() {
        let prop = arr
            .get(i)
            .as_string()
            .ok_or_else(|| JsValue::from_str("path elements must be strings"))?;
        let next_lens = lens(&prop);
        result = result.compose(&next_lens);
    }

    Ok(result)
}

/// An Optional lens focuses on a property that may not exist.
///
/// # Examples (in JavaScript)
///
/// ```javascript
/// import { optional } from './pkg/orlando.js';
///
/// const addressLens = optional('address');
/// const user1 = { name: "Alice", address: { city: "NYC" } };
/// const user2 = { name: "Bob" };
///
/// console.log(addressLens.get(user1));  // { city: "NYC" }
/// console.log(addressLens.get(user2));  // undefined
///
/// // over is a no-op when value doesn't exist
/// const updated = addressLens.over(user2, addr => ({ ...addr, city: "LA" }));
/// console.log(updated);  // { name: "Bob" } (unchanged)
/// ```
#[wasm_bindgen]
pub struct JsOptional {
    get_fn: JsGetter,
    set_fn: JsSetter,
}

#[wasm_bindgen]
impl JsOptional {
    /// Extract the focused value, which may be undefined.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript object to extract from
    ///
    /// # Returns
    ///
    /// The focused value or undefined if it doesn't exist
    #[wasm_bindgen]
    pub fn get(&self, source: &JsValue) -> JsValue {
        (self.get_fn)(source)
    }

    /// Extract the focused value with a default if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript object to extract from
    /// * `default` - The default value to return if the property doesn't exist
    ///
    /// # Returns
    ///
    /// The focused value or the default
    #[wasm_bindgen(js_name = getOr)]
    pub fn get_or(&self, source: &JsValue, default: JsValue) -> JsValue {
        let value = self.get(source);
        if value.is_undefined() || value.is_null() {
            default
        } else {
            value
        }
    }

    /// Update the focused value immutably if it exists.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript object to update
    /// * `value` - The new value to set
    ///
    /// # Returns
    ///
    /// A new object with the updated value
    #[wasm_bindgen]
    pub fn set(&self, source: &JsValue, value: JsValue) -> JsValue {
        (self.set_fn)(source, value)
    }

    /// Transform the focused value using a function, only if it exists.
    ///
    /// If the value doesn't exist, returns the source unchanged.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript object to transform
    /// * `f` - A JavaScript function to apply to the focused value
    ///
    /// # Returns
    ///
    /// A new object with the transformed value, or the original if value doesn't exist
    #[wasm_bindgen]
    pub fn over(&self, source: &JsValue, f: &Function) -> JsValue {
        let current = self.get(source);
        if current.is_undefined() || current.is_null() {
            // Value doesn't exist, return source unchanged
            source.clone()
        } else {
            let this = JsValue::null();
            let updated = f.call1(&this, &current).unwrap_or_else(|_| current.clone());
            self.set(source, updated)
        }
    }
}

/// Create an optional lens that focuses on a property that may not exist.
///
/// # Arguments
///
/// * `prop` - The property name to focus on
///
/// # Returns
///
/// An optional lens that focuses on the specified property
///
/// # Examples
///
/// ```javascript
/// const addressLens = optional('address');
/// const user = { name: "Alice" };
/// console.log(addressLens.get(user)); // undefined
/// console.log(addressLens.getOr(user, { city: "Unknown" })); // { city: "Unknown" }
/// ```
#[wasm_bindgen]
pub fn optional(prop: &str) -> JsOptional {
    let prop_get = prop.to_string();
    let prop_set = prop.to_string();

    JsOptional {
        get_fn: Rc::new(move |source: &JsValue| {
            if let Some(obj) = source.dyn_ref::<Object>() {
                Reflect::get(obj, &JsValue::from_str(&prop_get)).unwrap_or(JsValue::undefined())
            } else {
                JsValue::undefined()
            }
        }),
        set_fn: Rc::new(move |source: &JsValue, value: JsValue| {
            if let Some(obj) = source.dyn_ref::<Object>() {
                // Clone the object
                let new_obj = Object::assign(&Object::new(), obj);
                // Set the property
                let _ = Reflect::set(&new_obj, &JsValue::from_str(&prop_set), &value);
                new_obj.into()
            } else {
                source.clone()
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    // WASM tests will be in tests/wasm_tests.rs
}
