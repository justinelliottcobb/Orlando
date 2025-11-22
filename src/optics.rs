//! # Optics: Functional Lenses for Immutable Data Access
//!
//! Optics provide composable, type-safe access to nested data structures.
//! This module implements a hierarchy of optics based on category theory:
//!
//! - **Lens**: Total focus on a part of a structure (always exists)
//! - **Optional**: Partial focus (may not exist)
//! - **Traversal**: Multiple foci (collection elements)
//! - **Prism**: Focus on a variant of a sum type
//! - **Iso**: Lossless bidirectional conversion
//!
//! ## Lens Laws
//!
//! All lenses must satisfy three fundamental laws:
//!
//! 1. **GetPut**: `set(s, get(s)) = s` - Setting what you got changes nothing
//! 2. **PutGet**: `get(set(s, a)) = a` - Getting what you set returns that value
//! 3. **PutPut**: `set(set(s, a1), a2) = set(s, a2)` - Setting twice = setting once
//!
//! These laws ensure lenses are well-behaved and composable.
//!
//! ## Usage
//!
//! ```rust
//! use orlando_transducers::optics::Lens;
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! // Create a lens that focuses on a specific field
//! let name_lens = Lens::new(
//!     |user: &User| user.name.clone(),
//!     |user: &User, name: String| User { name, age: user.age }
//! );
//!
//! let user = User {
//!     name: "Alice".to_string(),
//!     age: 30,
//! };
//!
//! // Get the focused value
//! let name = name_lens.get(&user);
//! assert_eq!(name, "Alice");
//!
//! // Set the focused value
//! let updated_user = name_lens.set(&user, "Bob".to_string());
//! assert_eq!(updated_user.name, "Bob");
//!
//! // Transform the focused value
//! let upper_user = name_lens.over(&user, |s| s.to_uppercase());
//! assert_eq!(upper_user.name, "ALICE");
//! ```

use std::marker::PhantomData;
use std::rc::Rc;

// Type aliases to satisfy clippy type_complexity lint
type Getter<S, A> = Box<dyn Fn(&S) -> A>;
type Setter<S, A> = Box<dyn Fn(&S, A) -> S>;
type OptionalGetter<S, A> = Box<dyn Fn(&S) -> Option<A>>;

/// A Lens focuses on a part A of a structure S, allowing both reading and updating.
///
/// A Lens is defined by two functions:
/// - `get: &S -> A` - Extract the focused value
/// - `set: (&S, A) -> S` - Update the focused value immutably
///
/// Lenses compose: given `Lens<S, A>` and `Lens<A, B>`, we can create `Lens<S, B>`.
pub struct Lens<S, A>
where
    S: Clone,
    A: Clone,
{
    get: Getter<S, A>,
    set: Setter<S, A>,
    _phantom: PhantomData<(S, A)>,
}

impl<S, A> Lens<S, A>
where
    S: Clone,
    A: Clone,
{
    /// Create a new lens from getter and setter functions.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use orlando_transducers::optics::Lens;
    /// #[derive(Clone)]
    /// struct User { name: String }
    ///
    /// let name_lens = Lens::new(
    ///     |user: &User| user.name.clone(),
    ///     |user: &User, name: String| User { name }
    /// );
    /// ```
    pub fn new<G, S2>(get_fn: G, set_fn: S2) -> Self
    where
        G: Fn(&S) -> A + 'static,
        S2: Fn(&S, A) -> S + 'static,
    {
        Lens {
            get: Box::new(get_fn),
            set: Box::new(set_fn),
            _phantom: PhantomData,
        }
    }

    /// Extract the focused value from the structure.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use orlando_transducers::optics::Lens;
    /// # #[derive(Clone)]
    /// # struct User { name: String }
    /// # let lens = Lens::new(
    /// #     |user: &User| user.name.clone(),
    /// #     |user: &User, name: String| User { name }
    /// # );
    /// # let user = User { name: "Alice".to_string() };
    /// let name = lens.get(&user);
    /// assert_eq!(name, "Alice");
    /// ```
    pub fn get(&self, source: &S) -> A {
        (self.get)(source)
    }

    /// Update the focused value immutably, returning a new structure.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use orlando_transducers::optics::Lens;
    /// # #[derive(Clone, PartialEq, Debug)]
    /// # struct User { name: String }
    /// # let lens = Lens::new(
    /// #     |user: &User| user.name.clone(),
    /// #     |user: &User, name: String| User { name }
    /// # );
    /// # let user = User { name: "Alice".to_string() };
    /// let updated = lens.set(&user, "Bob".to_string());
    /// assert_eq!(updated.name, "Bob");
    /// assert_eq!(user.name, "Alice"); // Original unchanged
    /// ```
    pub fn set(&self, source: &S, value: A) -> S {
        (self.set)(source, value)
    }

    /// Transform the focused value using a function.
    ///
    /// This is equivalent to: `set(source, f(get(source)))`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use orlando_transducers::optics::Lens;
    /// # #[derive(Clone)]
    /// # struct User { name: String }
    /// # let lens = Lens::new(
    /// #     |user: &User| user.name.clone(),
    /// #     |user: &User, name: String| User { name }
    /// # );
    /// # let user = User { name: "Alice".to_string() };
    /// let updated = lens.over(&user, |name| name.to_uppercase());
    /// assert_eq!(updated.name, "ALICE");
    /// ```
    pub fn over<F>(&self, source: &S, f: F) -> S
    where
        F: Fn(A) -> A,
    {
        let current = self.get(source);
        let updated = f(current);
        self.set(source, updated)
    }

    /// Compose two lenses to focus deeper into nested structures.
    ///
    /// Given `Lens<S, A>` and `Lens<A, B>`, produces `Lens<S, B>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use orlando_transducers::optics::Lens;
    /// # #[derive(Clone)]
    /// # struct User { address: Address }
    /// # #[derive(Clone)]
    /// # struct Address { city: String }
    /// # let address_lens = Lens::new(
    /// #     |user: &User| user.address.clone(),
    /// #     |user: &User, address: Address| User { address }
    /// # );
    /// # let city_lens = Lens::new(
    /// #     |addr: &Address| addr.city.clone(),
    /// #     |addr: &Address, city: String| Address { city }
    /// # );
    /// let user_city_lens = address_lens.compose(city_lens);
    /// # let user = User { address: Address { city: "NYC".to_string() } };
    /// let city = user_city_lens.get(&user);
    /// assert_eq!(city, "NYC");
    /// ```
    pub fn compose<B>(self, other: Lens<A, B>) -> Lens<S, B>
    where
        B: Clone + 'static,
        A: 'static,
        S: 'static,
    {
        // Wrap lenses in Rc to share ownership between closures
        let self_rc_get = Rc::new(self.get);
        let self_rc_set = Rc::new(self.set);
        let other_rc_get = Rc::new(other.get);
        let other_rc_set = Rc::new(other.set);

        // Clone Rc for the setter closure
        let self_rc_get_2 = self_rc_get.clone();

        Lens::new(
            move |source: &S| {
                let intermediate = self_rc_get(source);
                other_rc_get(&intermediate)
            },
            move |source: &S, value: B| {
                let intermediate = self_rc_get_2(source);
                let updated_intermediate = other_rc_set(&intermediate, value);
                self_rc_set(source, updated_intermediate)
            },
        )
    }
}

/// An Optional focuses on a part A of a structure S that may not exist.
///
/// Like Lens, but the focused value may be None (e.g., nullable fields in JavaScript).
pub struct Optional<S, A>
where
    S: Clone,
    A: Clone,
{
    get: OptionalGetter<S, A>,
    set: Setter<S, A>,
    _phantom: PhantomData<(S, A)>,
}

impl<S, A> Optional<S, A>
where
    S: Clone,
    A: Clone,
{
    /// Create a new optional from getter and setter functions.
    pub fn new<G, S2>(get_fn: G, set_fn: S2) -> Self
    where
        G: Fn(&S) -> Option<A> + 'static,
        S2: Fn(&S, A) -> S + 'static,
    {
        Optional {
            get: Box::new(get_fn),
            set: Box::new(set_fn),
            _phantom: PhantomData,
        }
    }

    /// Extract the focused value, which may not exist.
    pub fn get(&self, source: &S) -> Option<A> {
        (self.get)(source)
    }

    /// Extract the focused value with a default if it doesn't exist.
    pub fn get_or(&self, source: &S, default: A) -> A {
        self.get(source).unwrap_or(default)
    }

    /// Update the focused value immutably if it exists.
    pub fn set(&self, source: &S, value: A) -> S {
        (self.set)(source, value)
    }

    /// Transform the focused value using a function, only if it exists.
    ///
    /// If the value doesn't exist, returns the source unchanged.
    pub fn over<F>(&self, source: &S, f: F) -> S
    where
        F: Fn(A) -> A,
    {
        match self.get(source) {
            Some(current) => {
                let updated = f(current);
                self.set(source, updated)
            }
            None => source.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    #[derive(Clone, Debug, PartialEq)]
    struct User {
        name: String,
        age: u32,
        address: Option<Address>,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Address {
        city: String,
        zip: String,
    }

    #[test]
    fn test_lens_get() {
        let name_lens = Lens::new(
            |user: &User| user.name.clone(),
            |user: &User, name: String| User {
                name,
                age: user.age,
                address: user.address.clone(),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        assert_eq!(name_lens.get(&user), "Alice");
    }

    #[test]
    fn test_lens_set() {
        let name_lens = Lens::new(
            |user: &User| user.name.clone(),
            |user: &User, name: String| User {
                name,
                age: user.age,
                address: user.address.clone(),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        let updated = name_lens.set(&user, "Bob".to_string());
        assert_eq!(updated.name, "Bob");
        assert_eq!(updated.age, 30);
        assert_eq!(user.name, "Alice"); // Original unchanged
    }

    #[test]
    fn test_lens_over() {
        let name_lens = Lens::new(
            |user: &User| user.name.clone(),
            |user: &User, name: String| User {
                name,
                age: user.age,
                address: user.address.clone(),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        let updated = name_lens.over(&user, |name| name.to_uppercase());
        assert_eq!(updated.name, "ALICE");
    }

    #[test]
    fn test_lens_law_get_put() {
        // Law 1: set(s, get(s)) = s
        let name_lens = Lens::new(
            |user: &User| user.name.clone(),
            |user: &User, name: String| User {
                name,
                age: user.age,
                address: user.address.clone(),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        let result = name_lens.set(&user, name_lens.get(&user));
        assert_eq!(result, user);
    }

    #[test]
    fn test_lens_law_put_get() {
        // Law 2: get(set(s, a)) = a
        let name_lens = Lens::new(
            |user: &User| user.name.clone(),
            |user: &User, name: String| User {
                name,
                age: user.age,
                address: user.address.clone(),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        let new_name = "Bob".to_string();
        let updated = name_lens.set(&user, new_name.clone());
        assert_eq!(name_lens.get(&updated), new_name);
    }

    #[test]
    fn test_lens_law_put_put() {
        // Law 3: set(set(s, a1), a2) = set(s, a2)
        let name_lens = Lens::new(
            |user: &User| user.name.clone(),
            |user: &User, name: String| User {
                name,
                age: user.age,
                address: user.address.clone(),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        let name1 = "Bob".to_string();
        let name2 = "Charlie".to_string();

        let result1 = name_lens.set(&name_lens.set(&user, name1), name2.clone());
        let result2 = name_lens.set(&user, name2);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_lens_composition() {
        let address_lens = Lens::new(
            |user: &User| user.address.clone().unwrap(),
            |user: &User, address: Address| User {
                name: user.name.clone(),
                age: user.age,
                address: Some(address),
            },
        );

        let city_lens = Lens::new(
            |addr: &Address| addr.city.clone(),
            |addr: &Address, city: String| Address {
                city,
                zip: addr.zip.clone(),
            },
        );

        let user_city_lens = address_lens.compose(city_lens);

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: Some(Address {
                city: "NYC".to_string(),
                zip: "10001".to_string(),
            }),
        };

        let city = user_city_lens.get(&user);
        assert_eq!(city, "NYC");

        let updated = user_city_lens.set(&user, "Boston".to_string());
        assert_eq!(updated.address.unwrap().city, "Boston");
    }

    #[test]
    fn test_optional_get_some() {
        let address_lens = Optional::new(
            |user: &User| user.address.clone(),
            |user: &User, address: Address| User {
                name: user.name.clone(),
                age: user.age,
                address: Some(address),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: Some(Address {
                city: "NYC".to_string(),
                zip: "10001".to_string(),
            }),
        };

        assert_eq!(
            address_lens.get(&user),
            Some(Address {
                city: "NYC".to_string(),
                zip: "10001".to_string()
            })
        );
    }

    #[test]
    fn test_optional_get_none() {
        let address_lens = Optional::new(
            |user: &User| user.address.clone(),
            |user: &User, address: Address| User {
                name: user.name.clone(),
                age: user.age,
                address: Some(address),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        assert_eq!(address_lens.get(&user), None);
    }

    #[test]
    fn test_optional_get_or() {
        let address_lens = Optional::new(
            |user: &User| user.address.clone(),
            |user: &User, address: Address| User {
                name: user.name.clone(),
                age: user.age,
                address: Some(address),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        let default = Address {
            city: "Unknown".to_string(),
            zip: "00000".to_string(),
        };

        assert_eq!(address_lens.get_or(&user, default.clone()), default);
    }

    #[test]
    fn test_optional_over_some() {
        let address_lens = Optional::new(
            |user: &User| user.address.clone(),
            |user: &User, address: Address| User {
                name: user.name.clone(),
                age: user.age,
                address: Some(address),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: Some(Address {
                city: "NYC".to_string(),
                zip: "10001".to_string(),
            }),
        };

        let updated = address_lens.over(&user, |addr| Address {
            city: addr.city.to_uppercase(),
            zip: addr.zip,
        });

        assert_eq!(updated.address.unwrap().city, "NYC");
    }

    #[test]
    fn test_optional_over_none() {
        let address_lens = Optional::new(
            |user: &User| user.address.clone(),
            |user: &User, address: Address| User {
                name: user.name.clone(),
                age: user.age,
                address: Some(address),
            },
        );

        let user = User {
            name: "Alice".to_string(),
            age: 30,
            address: None,
        };

        let updated = address_lens.over(&user, |addr| Address {
            city: addr.city.to_uppercase(),
            zip: addr.zip,
        });

        assert_eq!(updated, user); // Unchanged when None
    }

    // Property-based tests for lens laws
    #[cfg(not(target_arch = "wasm32"))]
    mod lens_laws_properties {
        use super::*;

        // Strategy to generate arbitrary User instances
        fn arbitrary_user() -> impl Strategy<Value = User> {
            (any::<String>(), 1u32..100u32, any::<Option<String>>()).prop_map(
                |(name, age, city_opt)| User {
                    name,
                    age,
                    address: city_opt.map(|city| Address {
                        city,
                        zip: "12345".to_string(),
                    }),
                },
            )
        }

        proptest! {
            /// Law 1: GetPut - Setting what you got changes nothing
            /// For all s: S, set(s, get(s)) = s
            #[test]
            fn prop_lens_law_get_put(user in arbitrary_user()) {
                let name_lens = Lens::new(
                    |user: &User| user.name.clone(),
                    |user: &User, name: String| User {
                        name,
                        age: user.age,
                        address: user.address.clone(),
                    },
                );

                let result = name_lens.set(&user, name_lens.get(&user));
                prop_assert_eq!(result, user);
            }

            /// Law 2: PutGet - Getting what you set returns that value
            /// For all s: S, a: A, get(set(s, a)) = a
            #[test]
            fn prop_lens_law_put_get(user in arbitrary_user(), new_name in any::<String>()) {
                let name_lens = Lens::new(
                    |user: &User| user.name.clone(),
                    |user: &User, name: String| User {
                        name,
                        age: user.age,
                        address: user.address.clone(),
                    },
                );

                let updated = name_lens.set(&user, new_name.clone());
                prop_assert_eq!(name_lens.get(&updated), new_name);
            }

            /// Law 3: PutPut - Setting twice equals setting once (last write wins)
            /// For all s: S, a1: A, a2: A, set(set(s, a1), a2) = set(s, a2)
            #[test]
            fn prop_lens_law_put_put(
                user in arbitrary_user(),
                name1 in any::<String>(),
                name2 in any::<String>()
            ) {
                let name_lens = Lens::new(
                    |user: &User| user.name.clone(),
                    |user: &User, name: String| User {
                        name,
                        age: user.age,
                        address: user.address.clone(),
                    },
                );

                let result1 = name_lens.set(&name_lens.set(&user, name1), name2.clone());
                let result2 = name_lens.set(&user, name2);

                prop_assert_eq!(result1, result2);
            }

            /// Test that `over` is equivalent to get + transform + set
            #[test]
            fn prop_lens_over_equivalence(user in arbitrary_user()) {
                let name_lens = Lens::new(
                    |user: &User| user.name.clone(),
                    |user: &User, name: String| User {
                        name,
                        age: user.age,
                        address: user.address.clone(),
                    },
                );

                let transform = |s: String| s.to_uppercase();

                // over should be equivalent to: set(s, f(get(s)))
                let result_over = name_lens.over(&user, transform);
                let result_manual = name_lens.set(&user, transform(name_lens.get(&user)));

                prop_assert_eq!(result_over, result_manual);
            }

            /// Test lens composition laws
            #[test]
            fn prop_lens_composition_get(user in arbitrary_user()) {
                // Skip if no address using prop_assume!
                prop_assume!(user.address.is_some());

                let address_lens = Lens::new(
                    |user: &User| user.address.clone().unwrap(),
                    |user: &User, address: Address| User {
                        name: user.name.clone(),
                        age: user.age,
                        address: Some(address),
                    },
                );

                let city_lens = Lens::new(
                    |addr: &Address| addr.city.clone(),
                    |addr: &Address, city: String| Address {
                        city,
                        zip: addr.zip.clone(),
                    },
                );

                let composed = address_lens.compose(city_lens);

                // Composed get should equal nested gets
                let composed_get = composed.get(&user);
                let manual_get = user.address.as_ref().unwrap().city.clone();

                prop_assert_eq!(composed_get, manual_get);
            }

            /// Test lens composition set
            #[test]
            fn prop_lens_composition_set(user in arbitrary_user(), new_city in any::<String>()) {
                // Skip if no address using prop_assume!
                prop_assume!(user.address.is_some());

                let address_lens = Lens::new(
                    |user: &User| user.address.clone().unwrap(),
                    |user: &User, address: Address| User {
                        name: user.name.clone(),
                        age: user.age,
                        address: Some(address),
                    },
                );

                let city_lens = Lens::new(
                    |addr: &Address| addr.city.clone(),
                    |addr: &Address, city: String| Address {
                        city,
                        zip: addr.zip.clone(),
                    },
                );

                let composed = address_lens.compose(city_lens);

                // Composed set should update nested value
                let updated = composed.set(&user, new_city.clone());
                prop_assert_eq!(updated.address.as_ref().unwrap().city.clone(), new_city);

                // Other fields should remain unchanged
                prop_assert_eq!(updated.name, user.name);
                prop_assert_eq!(updated.age, user.age);
                prop_assert_eq!(
                    updated.address.as_ref().unwrap().zip.clone(),
                    user.address.as_ref().unwrap().zip.clone()
                );
            }
        }
    }

    // Property-based tests for Optional laws
    #[cfg(not(target_arch = "wasm32"))]
    mod optional_laws_properties {
        use super::*;

        fn arbitrary_user() -> impl Strategy<Value = User> {
            (any::<String>(), 1u32..100u32, any::<Option<String>>()).prop_map(
                |(name, age, city_opt)| User {
                    name,
                    age,
                    address: city_opt.map(|city| Address {
                        city,
                        zip: "12345".to_string(),
                    }),
                },
            )
        }

        proptest! {
            /// Test Optional.get returns None when field is None
            #[test]
            fn prop_optional_get_none(name in any::<String>(), age in 1u32..100u32) {
                let user = User {
                    name,
                    age,
                    address: None,
                };

                let address_lens = Optional::new(
                    |user: &User| user.address.clone(),
                    |user: &User, address: Address| User {
                        name: user.name.clone(),
                        age: user.age,
                        address: Some(address),
                    },
                );

                prop_assert_eq!(address_lens.get(&user), None);
            }

            /// Test Optional.get returns Some when field exists
            #[test]
            fn prop_optional_get_some(user in arbitrary_user()) {
                prop_assume!(user.address.is_some());

                let address_lens = Optional::new(
                    |user: &User| user.address.clone(),
                    |user: &User, address: Address| User {
                        name: user.name.clone(),
                        age: user.age,
                        address: Some(address),
                    },
                );

                prop_assert_eq!(address_lens.get(&user), user.address);
            }

            /// Test Optional.over is no-op when value is None
            #[test]
            fn prop_optional_over_none(name in any::<String>(), age in 1u32..100u32) {
                let user = User {
                    name,
                    age,
                    address: None,
                };

                let address_lens = Optional::new(
                    |user: &User| user.address.clone(),
                    |user: &User, address: Address| User {
                        name: user.name.clone(),
                        age: user.age,
                        address: Some(address),
                    },
                );

                let updated = address_lens.over(&user, |addr| Address {
                    city: addr.city.to_uppercase(),
                    zip: addr.zip,
                });

                prop_assert_eq!(updated, user);
            }

            /// Test Optional.over transforms value when Some
            #[test]
            fn prop_optional_over_some(user in arbitrary_user()) {
                prop_assume!(user.address.is_some());

                let address_lens = Optional::new(
                    |user: &User| user.address.clone(),
                    |user: &User, address: Address| User {
                        name: user.name.clone(),
                        age: user.age,
                        address: Some(address),
                    },
                );

                let updated = address_lens.over(&user, |addr| Address {
                    city: addr.city.to_uppercase(),
                    zip: addr.zip.clone(),
                });

                prop_assert_eq!(
                    updated.address.as_ref().unwrap().city.clone(),
                    user.address.as_ref().unwrap().city.to_uppercase()
                );
            }

            /// Test Optional.get_or returns default when None
            #[test]
            fn prop_optional_get_or_none(name in any::<String>(), age in 1u32..100u32, default_city in any::<String>()) {
                let user = User {
                    name,
                    age,
                    address: None,
                };

                let default_addr = Address {
                    city: default_city,
                    zip: "00000".to_string(),
                };

                let address_lens = Optional::new(
                    |user: &User| user.address.clone(),
                    |user: &User, address: Address| User {
                        name: user.name.clone(),
                        age: user.age,
                        address: Some(address),
                    },
                );

                prop_assert_eq!(address_lens.get_or(&user, default_addr.clone()), default_addr);
            }

            /// Test Optional.get_or returns value when Some
            #[test]
            fn prop_optional_get_or_some(user in arbitrary_user(), default_city in any::<String>()) {
                prop_assume!(user.address.is_some());

                let default_addr = Address {
                    city: default_city,
                    zip: "00000".to_string(),
                };

                let address_lens = Optional::new(
                    |user: &User| user.address.clone(),
                    |user: &User, address: Address| User {
                        name: user.name.clone(),
                        age: user.age,
                        address: Some(address),
                    },
                );

                prop_assert_eq!(
                    address_lens.get_or(&user, default_addr),
                    user.address.clone().unwrap()
                );
            }
        }
    }
}
