//! Standard transducer transformations.
//!
//! This module provides common transducers like map, filter, take, etc.

use crate::step::{cont, stop, Step};
use crate::transducer::Transducer;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

/// Map transducer - transforms each value with a function.
///
/// # Category Theory
///
/// Map lifts a function `f: A -> B` to a transducer `Map(f): A ~> B`.
/// This is a functor homomorphism.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Map;
/// use orlando::transducer::Transducer;
/// use orlando::step::cont;
///
/// let double = Map::new(|x: i32| x * 2);
/// ```
pub struct Map<F, In, Out> {
    f: Rc<F>,
    _phantom: PhantomData<(In, Out)>,
}

impl<F, In, Out> Map<F, In, Out>
where
    F: Fn(In) -> Out,
{
    pub fn new(f: F) -> Self {
        Map {
            f: Rc::new(f),
            _phantom: PhantomData,
        }
    }
}

impl<F, In, Out> Transducer<In, Out> for Map<F, In, Out>
where
    F: Fn(In) -> Out + 'static,
    In: 'static,
    Out: 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, In) -> Step<Acc>>
    where
        R: Fn(Acc, Out) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let f = Rc::clone(&self.f);
        Box::new(move |acc, val| reducer(acc, f(val)))
    }
}

/// Filter transducer - only passes values matching a predicate.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Filter;
///
/// let evens_only = Filter::new(|x: &i32| x % 2 == 0);
/// ```
pub struct Filter<P, T> {
    predicate: Rc<P>,
    _phantom: PhantomData<T>,
}

impl<P, T> Filter<P, T>
where
    P: Fn(&T) -> bool,
{
    pub fn new(predicate: P) -> Self {
        Filter {
            predicate: Rc::new(predicate),
            _phantom: PhantomData,
        }
    }
}

impl<P, T> Transducer<T, T> for Filter<P, T>
where
    P: Fn(&T) -> bool + 'static,
    T: 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let predicate = Rc::clone(&self.predicate);
        Box::new(move |acc, val| {
            if predicate(&val) {
                reducer(acc, val)
            } else {
                cont(acc)
            }
        })
    }
}

/// Reject transducer - inverse of Filter, only passes values NOT matching a predicate.
///
/// This is more intuitive than writing `filter(x => !predicate(x))` for exclusion logic.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Reject;
/// use orlando::collectors::to_vec;
///
/// let no_evens = Reject::new(|x: &i32| x % 2 == 0);
/// let result = to_vec(&no_evens, vec![1, 2, 3, 4, 5]);
/// assert_eq!(result, vec![1, 3, 5]); // Only odd numbers
/// ```
pub struct Reject<P, T> {
    predicate: Rc<P>,
    _phantom: PhantomData<T>,
}

impl<P, T> Reject<P, T>
where
    P: Fn(&T) -> bool,
{
    pub fn new(predicate: P) -> Self {
        Reject {
            predicate: Rc::new(predicate),
            _phantom: PhantomData,
        }
    }
}

impl<P, T> Transducer<T, T> for Reject<P, T>
where
    P: Fn(&T) -> bool + 'static,
    T: 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let predicate = Rc::clone(&self.predicate);
        Box::new(move |acc, val| {
            // Inverse of filter - pass if predicate is FALSE
            if !predicate(&val) {
                reducer(acc, val)
            } else {
                cont(acc)
            }
        })
    }
}

/// Chunk transducer - groups consecutive elements into fixed-size chunks.
///
/// Only emits complete chunks. The final partial chunk (if any) is dropped.
/// This is consistent with streaming semantics where we don't have a completion phase.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Chunk;
/// use orlando::collectors::to_vec;
///
/// let chunker = Chunk::new(2);
/// let result = to_vec(&chunker, vec![1, 2, 3, 4, 5, 6]);
/// assert_eq!(result, vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
///
/// // Partial chunk at end is dropped
/// let result2 = to_vec(&chunker, vec![1, 2, 3, 4, 5]);
/// assert_eq!(result2, vec![vec![1, 2], vec![3, 4]]); // [5] is dropped
/// ```
pub struct Chunk<T> {
    size: usize,
    buffer: Rc<RefCell<Vec<T>>>,
}

impl<T> Chunk<T>
where
    T: Clone,
{
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "Chunk size must be greater than 0");
        Chunk {
            size,
            buffer: Rc::new(RefCell::new(Vec::with_capacity(size))),
        }
    }
}

impl<T> Transducer<T, Vec<T>> for Chunk<T>
where
    T: Clone + 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, Vec<T>) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let size = self.size;
        let buffer = Rc::clone(&self.buffer);

        Box::new(move |acc, val| {
            let mut buf = buffer.borrow_mut();
            buf.push(val);

            // If buffer is full, emit the chunk
            if buf.len() == size {
                let chunk = buf.clone();
                buf.clear();
                reducer(acc, chunk)
            } else {
                // Keep accumulating
                cont(acc)
            }
        })
    }
}

/// Take transducer - takes the first n elements, then stops.
///
/// This demonstrates early termination via the Step monad.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Take;
///
/// let take_5 = Take::<i32>::new(5);
/// ```
pub struct Take<T> {
    n: usize,
    count: Rc<RefCell<usize>>,
    _phantom: PhantomData<T>,
}

impl<T> Take<T> {
    pub fn new(n: usize) -> Self {
        Take {
            n,
            count: Rc::new(RefCell::new(0)),
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> Transducer<T, T> for Take<T> {
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let n = self.n;
        let count = Rc::clone(&self.count);

        Box::new(move |acc, val| {
            let mut c = count.borrow_mut();
            if *c < n {
                *c += 1;
                let result = reducer(acc, val);
                if *c >= n {
                    // Convert to Stop to signal termination - extract value regardless of Continue/Stop
                    match result {
                        Step::Continue(value) | Step::Stop(value) => stop(value),
                    }
                } else {
                    result
                }
            } else {
                stop(acc)
            }
        })
    }
}

/// TakeWhile transducer - takes elements while predicate is true, then stops.
///
/// # Examples
///
/// ```
/// use orlando::transforms::TakeWhile;
///
/// let take_while_positive = TakeWhile::new(|x: &i32| *x > 0);
/// ```
pub struct TakeWhile<P, T> {
    predicate: Rc<P>,
    _phantom: PhantomData<T>,
}

impl<P, T> TakeWhile<P, T>
where
    P: Fn(&T) -> bool,
{
    pub fn new(predicate: P) -> Self {
        TakeWhile {
            predicate: Rc::new(predicate),
            _phantom: PhantomData,
        }
    }
}

impl<P, T> Transducer<T, T> for TakeWhile<P, T>
where
    P: Fn(&T) -> bool + 'static,
    T: 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let predicate = Rc::clone(&self.predicate);
        Box::new(move |acc, val| {
            if predicate(&val) {
                reducer(acc, val)
            } else {
                stop(acc)
            }
        })
    }
}

/// Drop transducer - skips the first n elements.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Drop;
///
/// let skip_5 = Drop::<i32>::new(5);
/// ```
pub struct Drop<T> {
    n: usize,
    count: Rc<RefCell<usize>>,
    _phantom: PhantomData<T>,
}

impl<T> Drop<T> {
    pub fn new(n: usize) -> Self {
        Drop {
            n,
            count: Rc::new(RefCell::new(0)),
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> Transducer<T, T> for Drop<T> {
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let n = self.n;
        let count = Rc::clone(&self.count);

        Box::new(move |acc, val| {
            let mut c = count.borrow_mut();
            if *c < n {
                *c += 1;
                cont(acc)
            } else {
                reducer(acc, val)
            }
        })
    }
}

/// DropWhile transducer - skips elements while predicate is true.
///
/// # Examples
///
/// ```
/// use orlando::transforms::DropWhile;
///
/// let drop_negatives = DropWhile::new(|x: &i32| *x < 0);
/// ```
pub struct DropWhile<P, T> {
    predicate: Rc<P>,
    dropping: Rc<RefCell<bool>>,
    _phantom: PhantomData<T>,
}

impl<P, T> DropWhile<P, T>
where
    P: Fn(&T) -> bool,
{
    pub fn new(predicate: P) -> Self {
        DropWhile {
            predicate: Rc::new(predicate),
            dropping: Rc::new(RefCell::new(true)),
            _phantom: PhantomData,
        }
    }
}

impl<P, T> Transducer<T, T> for DropWhile<P, T>
where
    P: Fn(&T) -> bool + 'static,
    T: 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let predicate = Rc::clone(&self.predicate);
        let dropping = Rc::clone(&self.dropping);

        Box::new(move |acc, val| {
            let mut d = dropping.borrow_mut();
            if *d && predicate(&val) {
                cont(acc)
            } else {
                *d = false;
                reducer(acc, val)
            }
        })
    }
}

/// Unique transducer - deduplicates consecutive equal elements.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Unique;
///
/// let unique = Unique::<i32>::new();
/// ```
pub struct Unique<T> {
    last: Rc<RefCell<Option<T>>>,
}

impl<T> Unique<T> {
    pub fn new() -> Self {
        Unique {
            last: Rc::new(RefCell::new(None)),
        }
    }
}

impl<T> Default for Unique<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialEq + Clone + 'static> Transducer<T, T> for Unique<T> {
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let last = Rc::clone(&self.last);

        Box::new(move |acc, val| {
            let mut l = last.borrow_mut();
            let should_process = match l.as_ref() {
                None => true,
                Some(prev) => prev != &val,
            };

            if should_process {
                *l = Some(val.clone());
                reducer(acc, val)
            } else {
                cont(acc)
            }
        })
    }
}

/// UniqueBy transducer - deduplicates by a key function.
///
/// # Examples
///
/// ```
/// use orlando::transforms::UniqueBy;
///
/// let unique_by_abs = UniqueBy::new(|x: &i32| x.abs());
/// ```
pub struct UniqueBy<F, T, K> {
    key_fn: Rc<F>,
    seen: Rc<RefCell<HashSet<K>>>,
    _phantom: PhantomData<T>,
}

impl<F, T, K> UniqueBy<F, T, K>
where
    F: Fn(&T) -> K,
    K: Eq + Hash,
{
    pub fn new(key_fn: F) -> Self {
        UniqueBy {
            key_fn: Rc::new(key_fn),
            seen: Rc::new(RefCell::new(HashSet::new())),
            _phantom: PhantomData,
        }
    }
}

impl<F, T, K> Transducer<T, T> for UniqueBy<F, T, K>
where
    F: Fn(&T) -> K + 'static,
    T: 'static,
    K: Eq + Hash + 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let key_fn = Rc::clone(&self.key_fn);
        let seen = Rc::clone(&self.seen);

        Box::new(move |acc, val| {
            let key = key_fn(&val);
            let mut s = seen.borrow_mut();
            if s.insert(key) {
                reducer(acc, val)
            } else {
                cont(acc)
            }
        })
    }
}

/// Scan transducer - running accumulation (like reduce, but emits all intermediate values).
///
/// # Examples
///
/// ```
/// use orlando::transforms::Scan;
///
/// // Running sum
/// let running_sum = Scan::new(0, |acc: &i32, x: &i32| acc + x);
/// ```
pub struct Scan<F, T, S> {
    f: Rc<F>,
    #[allow(dead_code)]
    initial: S,
    state: Rc<RefCell<S>>,
    _phantom: PhantomData<T>,
}

impl<F, T, S> Scan<F, T, S>
where
    F: Fn(&S, &T) -> S,
    S: Clone,
{
    pub fn new(initial: S, f: F) -> Self {
        Scan {
            f: Rc::new(f),
            initial: initial.clone(),
            state: Rc::new(RefCell::new(initial)),
            _phantom: PhantomData,
        }
    }
}

impl<F, T, S> Transducer<T, S> for Scan<F, T, S>
where
    F: Fn(&S, &T) -> S + 'static,
    T: 'static,
    S: Clone + 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, S) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let f = Rc::clone(&self.f);
        let state = Rc::clone(&self.state);

        Box::new(move |acc, val| {
            let mut s = state.borrow_mut();
            let new_state = f(&*s, &val);
            *s = new_state.clone();
            reducer(acc, new_state)
        })
    }
}

/// FlatMap transducer - maps each element to a collection and flattens the result.
///
/// This is the monadic bind operation for transducers. Also known as `chain` in
/// some functional programming libraries.
///
/// # Category Theory
///
/// FlatMap is the bind operation (>>=) for the transducer monad:
/// ```text
/// flatMap : (A -> [B]) -> A ~> B
/// ```
///
/// # Examples
///
/// ```
/// use orlando::transforms::FlatMap;
/// use orlando::transducer::Transducer;
/// use orlando::collectors::to_vec;
///
/// // Duplicate and increment each element
/// let flat = FlatMap::new(|x: i32| vec![x, x + 1]);
/// let result = to_vec(&flat, vec![1, 2, 3]);
/// assert_eq!(result, vec![1, 2, 2, 3, 3, 4]);
/// ```
pub struct FlatMap<F, In, Out> {
    f: Rc<F>,
    _phantom: PhantomData<(In, Out)>,
}

impl<F, In, Out> FlatMap<F, In, Out>
where
    F: Fn(In) -> Vec<Out>,
{
    pub fn new(f: F) -> Self {
        FlatMap {
            f: Rc::new(f),
            _phantom: PhantomData,
        }
    }
}

impl<F, In, Out> Transducer<In, Out> for FlatMap<F, In, Out>
where
    F: Fn(In) -> Vec<Out> + 'static,
    In: 'static,
    Out: 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, In) -> Step<Acc>>
    where
        R: Fn(Acc, Out) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let f = Rc::clone(&self.f);
        Box::new(move |mut acc, val| {
            // Apply function to get collection
            let collection = f(val);

            // Reduce over the collection
            for item in collection {
                match reducer(acc, item) {
                    Step::Continue(new_acc) => acc = new_acc,
                    Step::Stop(final_acc) => return stop(final_acc),
                }
            }

            cont(acc)
        })
    }
}

/// Tap transducer - performs side effects without transforming values.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Tap;
///
/// let logger = Tap::new(|x: &i32| println!("Value: {}", x));
/// ```
pub struct Tap<F, T> {
    f: Rc<F>,
    _phantom: PhantomData<T>,
}

impl<F, T> Tap<F, T>
where
    F: Fn(&T),
{
    pub fn new(f: F) -> Self {
        Tap {
            f: Rc::new(f),
            _phantom: PhantomData,
        }
    }
}

impl<F, T> Transducer<T, T> for Tap<F, T>
where
    F: Fn(&T) + 'static,
    T: 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let f = Rc::clone(&self.f);
        Box::new(move |acc, val| {
            f(&val);
            reducer(acc, val)
        })
    }
}

/// Interpose transducer - inserts a separator between elements.
///
/// Useful for joining elements with a delimiter while maintaining streaming semantics.
/// Unlike string join, this works with any type and keeps the separator as an element.
///
/// # Examples
///
/// ```
/// use orlando::transforms::Interpose;
/// use orlando::collectors::to_vec;
///
/// let comma = Interpose::new(0);
/// let result = to_vec(&comma, vec![1, 2, 3]);
/// assert_eq!(result, vec![1, 0, 2, 0, 3]);
/// ```
///
/// ```
/// use orlando::transforms::Interpose;
/// use orlando::collectors::to_vec;
///
/// // Works with strings too
/// let space = Interpose::new(" ".to_string());
/// let result = to_vec(&space, vec!["hello".to_string(), "world".to_string()]);
/// assert_eq!(result, vec!["hello", " ", "world"]);
/// ```
pub struct Interpose<T> {
    separator: T,
    is_first: Rc<RefCell<bool>>,
}

impl<T> Interpose<T>
where
    T: Clone,
{
    pub fn new(separator: T) -> Self {
        Interpose {
            separator,
            is_first: Rc::new(RefCell::new(true)),
        }
    }
}

impl<T> Transducer<T, T> for Interpose<T>
where
    T: Clone + 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let separator = self.separator.clone();
        let is_first = Rc::clone(&self.is_first);

        Box::new(move |acc, val| {
            let mut first = is_first.borrow_mut();
            if *first {
                *first = false;
                reducer(acc, val)
            } else {
                // Emit separator, then the value
                match reducer(acc, separator.clone()) {
                    Step::Continue(acc2) => reducer(acc2, val),
                    Step::Stop(final_acc) => stop(final_acc),
                }
            }
        })
    }
}

/// RepeatEach transducer - repeats each element n times.
///
/// Useful for data augmentation, sampling, or creating test data patterns.
///
/// # Examples
///
/// ```
/// use orlando::transforms::RepeatEach;
/// use orlando::collectors::to_vec;
///
/// let triple = RepeatEach::new(3);
/// let result = to_vec(&triple, vec![1, 2]);
/// assert_eq!(result, vec![1, 1, 1, 2, 2, 2]);
/// ```
///
/// ```
/// use orlando::transforms::RepeatEach;
/// use orlando::collectors::to_vec;
///
/// // Repeat 0 times filters everything out
/// let none = RepeatEach::new(0);
/// let result = to_vec(&none, vec![1, 2, 3]);
/// assert_eq!(result, Vec::<i32>::new());
/// ```
pub struct RepeatEach<T> {
    n: usize,
    _phantom: PhantomData<T>,
}

impl<T> RepeatEach<T> {
    pub fn new(n: usize) -> Self {
        RepeatEach {
            n,
            _phantom: PhantomData,
        }
    }
}

impl<T> Transducer<T, T> for RepeatEach<T>
where
    T: Clone + 'static,
{
    #[inline(always)]
    fn apply<Acc, R>(&self, reducer: R) -> Box<dyn Fn(Acc, T) -> Step<Acc>>
    where
        R: Fn(Acc, T) -> Step<Acc> + 'static,
        Acc: 'static,
    {
        let n = self.n;

        Box::new(move |mut acc, val| {
            for _ in 0..n {
                match reducer(acc, val.clone()) {
                    Step::Continue(new_acc) => acc = new_acc,
                    Step::Stop(final_acc) => return stop(final_acc),
                }
            }
            cont(acc)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map() {
        let double = Map::new(|x: i32| x * 2);
        let reducer = |acc: Vec<i32>, x: i32| {
            let mut v = acc;
            v.push(x);
            cont(v)
        };

        let transformed = double.apply(reducer);
        let result = transformed(vec![], 5);
        assert_eq!(result.unwrap(), vec![10]);
    }

    #[test]
    fn test_filter() {
        let evens = Filter::new(|x: &i32| x % 2 == 0);
        let reducer = |acc: Vec<i32>, x: i32| {
            let mut v = acc;
            v.push(x);
            cont(v)
        };

        let transformed = evens.apply(reducer);
        let r1 = transformed(vec![], 2);
        let r2 = transformed(r1.unwrap(), 3);
        assert_eq!(r2.unwrap(), vec![2]);
    }

    #[test]
    fn test_reject() {
        let no_evens = Reject::new(|x: &i32| x % 2 == 0);
        let reducer = |acc: Vec<i32>, x: i32| {
            let mut v = acc;
            v.push(x);
            cont(v)
        };

        let transformed = no_evens.apply(reducer);
        let r1 = transformed(vec![], 2); // even, should be rejected
        let r2 = transformed(r1.unwrap(), 3); // odd, should pass
        assert_eq!(r2.unwrap(), vec![3]);
    }

    #[test]
    fn test_reject_composition() {
        use crate::collectors::to_vec;

        // Reject evens, then double the remaining odds
        let pipeline = Reject::new(|x: &i32| x % 2 == 0).compose(Map::new(|x: i32| x * 2));
        let result = to_vec(&pipeline, vec![1, 2, 3, 4, 5]);
        assert_eq!(result, vec![2, 6, 10]); // [1, 3, 5] doubled
    }

    #[test]
    fn test_reject_vs_filter() {
        use crate::collectors::to_vec;

        // Reject(p) should be equivalent to Filter(!p)
        let data = vec![1, 2, 3, 4, 5, 6];

        let reject_evens = Reject::new(|x: &i32| x % 2 == 0);
        let filter_odds = Filter::new(|x: &i32| x % 2 != 0);

        let result1 = to_vec(&reject_evens, data.clone());
        let result2 = to_vec(&filter_odds, data);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_chunk() {
        use crate::collectors::to_vec;

        let chunker = Chunk::new(2);
        let result = to_vec(&chunker, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(result, vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
    }

    #[test]
    fn test_chunk_partial() {
        use crate::collectors::to_vec;

        // Partial chunk at end is dropped
        let chunker = Chunk::new(2);
        let result = to_vec(&chunker, vec![1, 2, 3, 4, 5]);
        assert_eq!(result, vec![vec![1, 2], vec![3, 4]]); // [5] is dropped
    }

    #[test]
    fn test_chunk_exact() {
        use crate::collectors::to_vec;

        let chunker = Chunk::new(3);
        let result = to_vec(&chunker, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(result, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]);
    }

    #[test]
    fn test_chunk_composition() {
        use crate::collectors::to_vec;

        // Chunk after doubling
        let pipeline = Map::new(|x: i32| x * 2).compose(Chunk::new(2));
        let result = to_vec(&pipeline, vec![1, 2, 3, 4]);
        assert_eq!(result, vec![vec![2, 4], vec![6, 8]]);
    }

    #[test]
    #[should_panic(expected = "Chunk size must be greater than 0")]
    fn test_chunk_zero_size() {
        let _chunker = Chunk::<i32>::new(0);
    }

    #[test]
    fn test_take() {
        let take_2 = Take::<i32>::new(2);
        let reducer = |acc: Vec<i32>, x: i32| {
            let mut v = acc;
            v.push(x);
            cont(v)
        };

        let transformed = take_2.apply(reducer);
        let r1 = transformed(vec![], 1);
        assert!(r1.is_continue());
        let r2 = transformed(r1.unwrap(), 2);
        assert!(r2.is_stop()); // Should stop after 2 elements
    }

    #[test]
    fn test_flatmap() {
        use crate::collectors::to_vec;

        // Test basic flattening
        let flat = FlatMap::new(|x: i32| vec![x, x + 1]);
        let result = to_vec(&flat, vec![1, 2, 3]);
        assert_eq!(result, vec![1, 2, 2, 3, 3, 4]);
    }

    #[test]
    fn test_flatmap_empty() {
        use crate::collectors::to_vec;

        // Test with function that returns empty collections
        let flat = FlatMap::new(|x: i32| if x % 2 == 0 { vec![x] } else { vec![] });
        let result = to_vec(&flat, vec![1, 2, 3, 4]);
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_flatmap_composition() {
        use crate::collectors::to_vec;

        // Test composing with other transducers
        let pipeline = Map::new(|x: i32| x * 2).compose(FlatMap::new(|x: i32| vec![x, x + 1]));
        let result = to_vec(&pipeline, vec![1, 2, 3]);
        assert_eq!(result, vec![2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_flatmap_early_termination() {
        use crate::collectors::to_vec;

        // FlatMap should respect early termination
        let pipeline = FlatMap::new(|x: i32| vec![x, x + 1, x + 2]).compose(Take::new(5));
        let result = to_vec(&pipeline, vec![1, 2, 3, 4, 5]);
        // Should stop after 5 elements total
        assert_eq!(result.len(), 5);
        assert_eq!(result, vec![1, 2, 3, 2, 3]);
    }
}
