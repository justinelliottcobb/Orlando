//! Terminal operations (collectors) for transducers.
//!
//! Collectors are reducing functions that consume the output of a transducer
//! and produce a final result.

use crate::step::{cont, Step};
use crate::transducer::Transducer;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Execute a transducer over an iterator and collect results into a vector.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::to_vec;
/// use orlando_transducers::transforms::Map;
///
/// let double = Map::new(|x: i32| x * 2);
/// let result = to_vec(&double, vec![1, 2, 3].into_iter());
/// assert_eq!(result, vec![2, 4, 6]);
/// ```
pub fn to_vec<T, U, Iter>(transducer: &impl Transducer<T, U>, source: Iter) -> Vec<U>
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
{
    let reducer = |mut acc: Vec<U>, x: U| {
        acc.push(x);
        cont(acc)
    };

    let transformed = transducer.apply(reducer);
    let mut result = Vec::new();

    for item in source {
        match transformed(result, item) {
            Step::Continue(new_result) => result = new_result,
            Step::Stop(final_result) => {
                result = final_result;
                break;
            }
        }
    }

    result
}

/// Reduce with a custom reducer function.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::reduce;
/// use orlando_transducers::transforms::Map;
/// use orlando_transducers::step::cont;
///
/// let double = Map::new(|x: i32| x * 2);
/// let sum = reduce(&double, vec![1, 2, 3].into_iter(), 0, |acc, x| cont(acc + x));
/// assert_eq!(sum, 12); // (1+2+3)*2
/// ```
pub fn reduce<T, U, Acc, Iter, R>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    initial: Acc,
    reducer: R,
) -> Acc
where
    T: 'static,
    U: 'static,
    Acc: 'static,
    Iter: IntoIterator<Item = T>,
    R: Fn(Acc, U) -> Step<Acc> + 'static,
{
    let transformed = transducer.apply(reducer);
    let mut acc = initial;

    for item in source {
        match transformed(acc, item) {
            Step::Continue(new_acc) => acc = new_acc,
            Step::Stop(final_acc) => {
                acc = final_acc;
                break;
            }
        }
    }

    acc
}

/// Sum numeric values.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::sum;
/// use orlando_transducers::transforms::Map;
///
/// let double = Map::new(|x: i32| x * 2);
/// let result = sum(&double, vec![1, 2, 3].into_iter());
/// assert_eq!(result, 12);
/// ```
pub fn sum<T, U, Iter>(transducer: &impl Transducer<T, U>, source: Iter) -> U
where
    T: 'static,
    U: std::ops::Add<Output = U> + Default + 'static,
    Iter: IntoIterator<Item = T>,
{
    reduce(transducer, source, U::default(), |acc, x| cont(acc + x))
}

/// Count the number of elements.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::count;
/// use orlando_transducers::transforms::Filter;
///
/// let evens = Filter::new(|x: &i32| x % 2 == 0);
/// let result = count(&evens, vec![1, 2, 3, 4, 5].into_iter());
/// assert_eq!(result, 2);
/// ```
pub fn count<T, U, Iter>(transducer: &impl Transducer<T, U>, source: Iter) -> usize
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
{
    reduce(transducer, source, 0usize, |acc, _| cont(acc + 1))
}

/// Get the first element (utilizes early termination).
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::first;
/// use orlando_transducers::transforms::Filter;
///
/// let evens = Filter::new(|x: &i32| x % 2 == 0);
/// let result = first(&evens, vec![1, 3, 4, 5].into_iter());
/// assert_eq!(result, Some(4));
/// ```
pub fn first<T, U, Iter>(transducer: &impl Transducer<T, U>, source: Iter) -> Option<U>
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
{
    use crate::step::stop;

    let reducer = |_acc: Option<U>, x: U| stop(Some(x));
    reduce(transducer, source, None, reducer)
}

/// Get the last element.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::last;
/// use orlando_transducers::transforms::Filter;
///
/// let evens = Filter::new(|x: &i32| x % 2 == 0);
/// let result = last(&evens, vec![2, 3, 4, 5, 6].into_iter());
/// assert_eq!(result, Some(6));
/// ```
pub fn last<T, U, Iter>(transducer: &impl Transducer<T, U>, source: Iter) -> Option<U>
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
{
    reduce(transducer, source, None, |_acc, x| cont(Some(x)))
}

/// Test if all elements match a predicate.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::every;
/// use orlando_transducers::transducer::Identity;
///
/// let id = Identity::<i32>::new();
/// let result = every(&id, vec![2, 4, 6].into_iter(), |x| x % 2 == 0);
/// assert_eq!(result, true);
/// ```
pub fn every<T, U, Iter, P>(transducer: &impl Transducer<T, U>, source: Iter, predicate: P) -> bool
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
    P: Fn(&U) -> bool + 'static,
{
    use crate::step::stop;

    let reducer = move |_acc: bool, x: U| {
        if predicate(&x) {
            cont(true)
        } else {
            stop(false)
        }
    };

    reduce(transducer, source, true, reducer)
}

/// Test if any element matches a predicate.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::some;
/// use orlando_transducers::transducer::Identity;
///
/// let id = Identity::<i32>::new();
/// let result = some(&id, vec![1, 3, 4, 5].into_iter(), |x| x % 2 == 0);
/// assert_eq!(result, true);
/// ```
pub fn some<T, U, Iter, P>(transducer: &impl Transducer<T, U>, source: Iter, predicate: P) -> bool
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
    P: Fn(&U) -> bool + 'static,
{
    use crate::step::stop;

    let reducer = move |_acc: bool, x: U| {
        if predicate(&x) {
            stop(true)
        } else {
            cont(false)
        }
    };

    reduce(transducer, source, false, reducer)
}

/// Partition elements into two groups based on a predicate.
///
/// Returns a tuple of (pass, fail) where `pass` contains elements that
/// satisfy the predicate and `fail` contains those that don't.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::partition;
/// use orlando_transducers::transducer::Identity;
///
/// let id = Identity::<i32>::new();
/// let (evens, odds) = partition(&id, vec![1, 2, 3, 4, 5].into_iter(), |x| x % 2 == 0);
/// assert_eq!(evens, vec![2, 4]);
/// assert_eq!(odds, vec![1, 3, 5]);
/// ```
pub fn partition<T, U, Iter, P>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    predicate: P,
) -> (Vec<U>, Vec<U>)
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
    P: Fn(&U) -> bool + 'static,
{
    let reducer = move |mut acc: (Vec<U>, Vec<U>), x: U| {
        if predicate(&x) {
            acc.0.push(x);
        } else {
            acc.1.push(x);
        }
        cont(acc)
    };

    reduce(transducer, source, (Vec::new(), Vec::new()), reducer)
}

/// Find the first element that satisfies a predicate.
///
/// Returns `Some(element)` if found, `None` otherwise.
/// Utilizes early termination to stop as soon as a match is found.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::find;
/// use orlando_transducers::transducer::Identity;
///
/// let id = Identity::<i32>::new();
/// let result = find(&id, vec![1, 3, 4, 5].into_iter(), |x| x % 2 == 0);
/// assert_eq!(result, Some(4));
/// ```
pub fn find<T, U, Iter, P>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    predicate: P,
) -> Option<U>
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
    P: Fn(&U) -> bool + 'static,
{
    use crate::step::stop;

    let reducer = move |_acc: Option<U>, x: U| {
        if predicate(&x) {
            stop(Some(x))
        } else {
            cont(None)
        }
    };

    reduce(transducer, source, None, reducer)
}

/// Group elements by a key function into a HashMap.
///
/// Returns a HashMap where keys are produced by the key function and values
/// are vectors of elements that share that key.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::group_by;
/// use orlando_transducers::transducer::Identity;
/// use std::collections::HashMap;
///
/// let id = Identity::<i32>::new();
/// let groups = group_by(&id, vec![1, 2, 3, 4, 5, 6].into_iter(), |x| x % 3);
///
/// assert_eq!(groups.get(&0), Some(&vec![3, 6]));
/// assert_eq!(groups.get(&1), Some(&vec![1, 4]));
/// assert_eq!(groups.get(&2), Some(&vec![2, 5]));
/// ```
pub fn group_by<T, U, K, Iter, F>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    key_fn: F,
) -> HashMap<K, Vec<U>>
where
    T: 'static,
    U: 'static,
    K: Eq + Hash + 'static,
    Iter: IntoIterator<Item = T>,
    F: Fn(&U) -> K + 'static,
{
    let reducer = move |mut acc: HashMap<K, Vec<U>>, x: U| {
        let key = key_fn(&x);
        acc.entry(key).or_default().push(x);
        cont(acc)
    };

    reduce(transducer, source, HashMap::new(), reducer)
}

/// Test if NO elements match a predicate (inverse of `some`).
///
/// Returns true if all elements fail the predicate, false if any match.
/// Utilizes early termination to stop as soon as a match is found.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::none;
/// use orlando_transducers::transducer::Identity;
///
/// let id = Identity::<i32>::new();
/// assert!(none(&id, vec![1, 3, 5, 7].into_iter(), |x| x % 2 == 0)); // No evens
/// assert!(!none(&id, vec![1, 2, 3].into_iter(), |x| x % 2 == 0)); // Has evens
/// ```
pub fn none<T, U, Iter, P>(transducer: &impl Transducer<T, U>, source: Iter, predicate: P) -> bool
where
    T: 'static,
    U: 'static,
    Iter: IntoIterator<Item = T>,
    P: Fn(&U) -> bool + 'static,
{
    use crate::step::stop;

    // Inverse of some - return false (stop) if any element matches
    let reducer = move |_acc: bool, x: U| {
        if predicate(&x) {
            stop(false) // Found a match, return false
        } else {
            cont(true) // Keep looking
        }
    };

    reduce(transducer, source, true, reducer)
}

/// Test if the collection contains a specific value.
///
/// Returns true if any element equals the target value, false otherwise.
/// Utilizes early termination to stop as soon as the value is found.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::contains;
/// use orlando_transducers::transducer::Identity;
///
/// let id = Identity::<i32>::new();
/// assert!(contains(&id, vec![1, 2, 3, 4, 5].into_iter(), &3));
/// assert!(!contains(&id, vec![1, 2, 4, 5].into_iter(), &3));
/// ```
pub fn contains<T, U, Iter>(transducer: &impl Transducer<T, U>, source: Iter, value: &U) -> bool
where
    T: 'static,
    U: PartialEq + Clone + 'static,
    Iter: IntoIterator<Item = T>,
{
    use crate::step::stop;

    let target = value.clone();
    let reducer = move |_acc: bool, x: U| {
        if x == target {
            stop(true) // Found it!
        } else {
            cont(false) // Keep looking
        }
    };

    reduce(transducer, source, false, reducer)
}

/// Zip two iterators into pairs (helper function, not a transducer).
///
/// This doesn't fit the single-input transducer model, so it's implemented
/// as a standalone helper function. Stops when either iterator is exhausted.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::zip;
///
/// let a = vec![1, 2, 3];
/// let b = vec!['a', 'b', 'c', 'd'];
/// let result = zip(a, b);
/// assert_eq!(result, vec![(1, 'a'), (2, 'b'), (3, 'c')]);
/// ```
pub fn zip<T, U, IterT, IterU>(iter_a: IterT, iter_b: IterU) -> Vec<(T, U)>
where
    IterT: IntoIterator<Item = T>,
    IterU: IntoIterator<Item = U>,
{
    iter_a.into_iter().zip(iter_b).collect()
}

/// Zip two iterators with a combining function (helper function, not a transducer).
///
/// Like `zip`, but applies a function to combine the elements instead of
/// creating tuples. Stops when either iterator is exhausted.
///
/// # Examples
///
/// ```
/// use orlando_transducers::collectors::zip_with;
///
/// let a = vec![1, 2, 3];
/// let b = vec![10, 20, 30];
/// let result = zip_with(a, b, |x, y| x + y);
/// assert_eq!(result, vec![11, 22, 33]);
/// ```
pub fn zip_with<T, U, V, IterT, IterU, F>(iter_a: IterT, iter_b: IterU, combine: F) -> Vec<V>
where
    IterT: IntoIterator<Item = T>,
    IterU: IntoIterator<Item = U>,
    F: Fn(T, U) -> V,
{
    iter_a
        .into_iter()
        .zip(iter_b)
        .map(|(a, b)| combine(a, b))
        .collect()
}

/// Merge multiple iterators by interleaving their elements in round-robin fashion.
///
/// Takes elements from each iterator in turn until all iterators are exhausted.
/// If iterators have different lengths, continues with remaining iterators.
///
/// # Examples
///
/// ```
/// use orlando_transducers::merge;
///
/// let a = vec![1, 2, 3];
/// let b = vec![4, 5, 6];
/// let result = merge(vec![a, b]);
/// assert_eq!(result, vec![1, 4, 2, 5, 3, 6]);
/// ```
///
/// ```
/// use orlando_transducers::merge;
///
/// // Different length iterators
/// let a = vec![1, 2];
/// let b = vec![3, 4, 5, 6];
/// let result = merge(vec![a, b]);
/// assert_eq!(result, vec![1, 3, 2, 4, 5, 6]);
/// ```
pub fn merge<T, I>(iterators: Vec<I>) -> Vec<T>
where
    I: IntoIterator<Item = T>,
{
    let mut iters: Vec<_> = iterators.into_iter().map(|i| i.into_iter()).collect();
    let mut result = Vec::new();
    let mut active = true;

    while active {
        active = false;
        for iter in &mut iters {
            if let Some(val) = iter.next() {
                result.push(val);
                active = true;
            }
        }
    }

    result
}

/// Compute the intersection of two iterators (elements in both A and B).
///
/// Returns elements that appear in both iterators, preserving order from the first iterator.
/// Duplicates from the first iterator are included if the element exists in the second.
///
/// # Examples
///
/// ```
/// use orlando_transducers::intersection;
///
/// let a = vec![1, 2, 3, 4];
/// let b = vec![3, 4, 5, 6];
/// let result = intersection(a, b);
/// assert_eq!(result, vec![3, 4]);
/// ```
///
/// ```
/// use orlando_transducers::intersection;
///
/// let a = vec![1, 2, 2, 3];
/// let b = vec![2, 3, 4];
/// let result = intersection(a, b);
/// assert_eq!(result, vec![2, 2, 3]);
/// ```
pub fn intersection<T, IterA, IterB>(iter_a: IterA, iter_b: IterB) -> Vec<T>
where
    T: Eq + Hash + Clone,
    IterA: IntoIterator<Item = T>,
    IterB: IntoIterator<Item = T>,
{
    let set_b: HashSet<T> = iter_b.into_iter().collect();
    iter_a
        .into_iter()
        .filter(|item| set_b.contains(item))
        .collect()
}

/// Compute the difference of two iterators (elements in A but not in B).
///
/// Returns elements from the first iterator that don't appear in the second,
/// preserving order from the first iterator.
///
/// # Examples
///
/// ```
/// use orlando_transducers::difference;
///
/// let a = vec![1, 2, 3, 4];
/// let b = vec![3, 4, 5, 6];
/// let result = difference(a, b);
/// assert_eq!(result, vec![1, 2]);
/// ```
///
/// ```
/// use orlando_transducers::difference;
///
/// let a = vec![1, 2, 2, 3];
/// let b = vec![2];
/// let result = difference(a, b);
/// assert_eq!(result, vec![1, 3]);
/// ```
pub fn difference<T, IterA, IterB>(iter_a: IterA, iter_b: IterB) -> Vec<T>
where
    T: Eq + Hash + Clone,
    IterA: IntoIterator<Item = T>,
    IterB: IntoIterator<Item = T>,
{
    let set_b: HashSet<T> = iter_b.into_iter().collect();
    iter_a
        .into_iter()
        .filter(|item| !set_b.contains(item))
        .collect()
}

/// Compute the union of two iterators (unique elements from both A and B).
///
/// Returns all unique elements that appear in either iterator.
/// Order is preserved: all unique elements from A first, then unique elements from B.
///
/// # Examples
///
/// ```
/// use orlando_transducers::union;
///
/// let a = vec![1, 2, 3];
/// let b = vec![3, 4, 5];
/// let result = union(a, b);
/// assert_eq!(result, vec![1, 2, 3, 4, 5]);
/// ```
///
/// ```
/// use orlando_transducers::union;
///
/// let a = vec![1, 2, 2, 3];
/// let b = vec![3, 4, 4, 5];
/// let result = union(a, b);
/// assert_eq!(result, vec![1, 2, 3, 4, 5]);
/// ```
pub fn union<T, IterA, IterB>(iter_a: IterA, iter_b: IterB) -> Vec<T>
where
    T: Eq + Hash + Clone,
    IterA: IntoIterator<Item = T>,
    IterB: IntoIterator<Item = T>,
{
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    // Add all unique elements from A
    for item in iter_a {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }

    // Add all unique elements from B that aren't in A
    for item in iter_b {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }

    result
}

/// Compute the symmetric difference of two iterators (elements in A or B but not both).
///
/// Returns elements that appear in exactly one of the two iterators.
/// Order: unique-to-A elements first, then unique-to-B elements.
///
/// # Examples
///
/// ```
/// use orlando_transducers::symmetric_difference;
///
/// let a = vec![1, 2, 3, 4];
/// let b = vec![3, 4, 5, 6];
/// let result = symmetric_difference(a, b);
/// assert_eq!(result, vec![1, 2, 5, 6]);
/// ```
///
/// ```
/// use orlando_transducers::symmetric_difference;
///
/// let a = vec![1, 2];
/// let b = vec![3, 4];
/// let result = symmetric_difference(a, b);
/// assert_eq!(result, vec![1, 2, 3, 4]);
/// ```
pub fn symmetric_difference<T, IterA, IterB>(iter_a: IterA, iter_b: IterB) -> Vec<T>
where
    T: Eq + Hash + Clone,
    IterA: IntoIterator<Item = T>,
    IterB: IntoIterator<Item = T>,
{
    let vec_a: Vec<T> = iter_a.into_iter().collect();
    let vec_b: Vec<T> = iter_b.into_iter().collect();

    let set_a: HashSet<&T> = vec_a.iter().collect();
    let set_b: HashSet<&T> = vec_b.iter().collect();

    let mut result = Vec::new();
    let mut seen = HashSet::new();

    // Elements in A but not B (preserving order from A)
    for item in &vec_a {
        if !set_b.contains(item) && seen.insert(item) {
            result.push(item.clone());
        }
    }

    // Elements in B but not A (preserving order from B)
    for item in &vec_b {
        if !set_a.contains(item) && seen.insert(item) {
            result.push(item.clone());
        }
    }

    result
}

// ============================================================================
// Advanced Collectors (Phase 2b)
// ============================================================================

/// Perform reservoir sampling to randomly sample n elements from a stream.
///
/// Uses Algorithm R for uniform random sampling with constant memory (O(n)).
/// Each element has an equal probability of being selected, even for streams
/// of unknown size.
///
/// # Examples
///
/// ```
/// use orlando_transducers::{reservoir_sample, Map};
///
/// let pipeline = Map::new(|x: i32| x * 2);
/// let sample = reservoir_sample(&pipeline, 1..1000, 10);
/// assert_eq!(sample.len(), 10);
/// // Each element from the processed stream has equal 10/999 probability
/// ```
///
/// ```
/// use orlando_transducers::{reservoir_sample, transducer::Identity};
///
/// // Sample 5 items from large dataset
/// let id = Identity::<i32>::new();
/// let sample = reservoir_sample(&id, 1..1_000_000, 5);
/// assert_eq!(sample.len(), 5);
/// ```
pub fn reservoir_sample<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    n: usize,
) -> Vec<U>
where
    T: 'static,
    U: 'static + Clone,
    Iter: IntoIterator<Item = T>,
{
    use rand::Rng;
    use std::cell::RefCell;
    use std::rc::Rc;

    let rng = Rc::new(RefCell::new(rand::thread_rng()));
    let reservoir: Rc<RefCell<Vec<U>>> = Rc::new(RefCell::new(Vec::with_capacity(n)));
    let count = Rc::new(RefCell::new(0usize));

    let reducer = {
        let rng = Rc::clone(&rng);
        let reservoir = Rc::clone(&reservoir);
        let count = Rc::clone(&count);

        move |_acc: (), x: U| {
            let mut c = count.borrow_mut();
            *c += 1;

            let mut res = reservoir.borrow_mut();
            if res.len() < n {
                // Fill reservoir
                res.push(x);
            } else {
                // Randomly replace elements with decreasing probability
                let mut rng_mut = rng.borrow_mut();
                let j = rng_mut.gen_range(0..*c);
                if j < n {
                    res[j] = x;
                }
            }

            cont(())
        }
    };

    reduce(transducer, source, (), reducer);
    Rc::try_unwrap(reservoir)
        .unwrap_or_else(|_| panic!("Failed to unwrap reservoir"))
        .into_inner()
}

/// Group consecutive elements by a key function (SQL-like PARTITION BY).
///
/// Unlike `group_by`, this only groups consecutive elements with the same key.
/// This is more memory-efficient for pre-sorted data and preserves streaming.
///
/// # Examples
///
/// ```
/// use orlando_transducers::{partition_by, transducer::Identity};
///
/// let data = vec![1, 1, 2, 2, 1, 1];
/// let id = Identity::new();
/// let groups = partition_by(&id, data, |x| *x);
///
/// assert_eq!(groups.len(), 3);
/// assert_eq!(groups[0], vec![1, 1]);
/// assert_eq!(groups[1], vec![2, 2]);
/// assert_eq!(groups[2], vec![1, 1]);
/// ```
///
/// ```
/// use orlando_transducers::{partition_by, Map};
///
/// let pipeline = Map::new(|x: i32| x % 3);
/// let data = vec![3, 6, 9, 1, 4, 7, 2, 5];
/// let groups = partition_by(&pipeline, data, |x| *x);
/// // Groups by result: [[0,0,0], [1,1,1], [2,2]]
/// assert_eq!(groups.len(), 3);
/// ```
pub fn partition_by<T, U, K, Iter, F>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
    key_fn: F,
) -> Vec<Vec<U>>
where
    T: 'static,
    U: 'static + Clone,
    K: Eq + Hash + 'static,
    Iter: IntoIterator<Item = T>,
    F: Fn(&U) -> K + 'static,
{
    use std::cell::RefCell;
    use std::rc::Rc;

    let result: Rc<RefCell<Vec<Vec<U>>>> = Rc::new(RefCell::new(Vec::new()));
    let current_group: Rc<RefCell<Vec<U>>> = Rc::new(RefCell::new(Vec::new()));
    let current_key: Rc<RefCell<Option<K>>> = Rc::new(RefCell::new(None));

    let reducer = {
        let result = Rc::clone(&result);
        let current_group = Rc::clone(&current_group);
        let current_key = Rc::clone(&current_key);

        move |_acc: (), x: U| {
            let key = key_fn(&x);

            let mut key_ref = current_key.borrow_mut();
            let mut group_ref = current_group.borrow_mut();

            match key_ref.as_ref() {
                None => {
                    // First element
                    *key_ref = Some(key);
                    group_ref.push(x);
                }
                Some(prev_key) if key == *prev_key => {
                    // Same group
                    group_ref.push(x);
                }
                Some(_) => {
                    // New group - save old group and start new one
                    if !group_ref.is_empty() {
                        result.borrow_mut().push(group_ref.clone());
                        group_ref.clear();
                    }
                    *key_ref = Some(key);
                    group_ref.push(x);
                }
            }

            cont(())
        }
    };

    reduce(transducer, source, (), reducer);

    // Don't forget the last group
    let final_group = current_group.borrow().clone();
    if !final_group.is_empty() {
        result.borrow_mut().push(final_group);
    }

    Rc::try_unwrap(result)
        .unwrap_or_else(|_| panic!("Failed to unwrap result"))
        .into_inner()
}

/// Find the top K elements using a min-heap (O(n log k) complexity).
///
/// This is much more efficient than sorting the entire collection when k << n.
/// Uses a binary heap to maintain only the top k elements.
///
/// # Examples
///
/// ```
/// use orlando_transducers::{top_k, transducer::Identity};
///
/// let data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
/// let id = Identity::new();
/// let top3 = top_k(&id, data, 3);
///
/// assert_eq!(top3.len(), 3);
/// assert!(top3.contains(&9));
/// assert!(top3.contains(&6));
/// assert!(top3.contains(&5));
/// ```
///
/// ```
/// use orlando_transducers::{top_k, Map};
///
/// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// struct Product { sales: i32, name: &'static str }
///
/// let pipeline = Map::new(|p: Product| p);
/// let products = vec![
///     Product { sales: 100, name: "A" },
///     Product { sales: 500, name: "B" },
///     Product { sales: 200, name: "C" },
/// ];
///
/// let top2 = top_k(&pipeline, products, 2);
/// assert_eq!(top2.len(), 2);
/// assert_eq!(top2[0].name, "B"); // Highest sales
/// ```
pub fn top_k<T, U, Iter>(transducer: &impl Transducer<T, U>, source: Iter, k: usize) -> Vec<U>
where
    T: 'static,
    U: Ord + Clone + 'static,
    Iter: IntoIterator<Item = T>,
{
    use std::cell::RefCell;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    use std::rc::Rc;

    let heap: Rc<RefCell<BinaryHeap<Reverse<U>>>> =
        Rc::new(RefCell::new(BinaryHeap::with_capacity(k + 1)));

    let reducer = {
        let heap = Rc::clone(&heap);

        move |_acc: (), x: U| {
            let mut heap_ref = heap.borrow_mut();
            if heap_ref.len() < k {
                heap_ref.push(Reverse(x));
            } else if let Some(Reverse(min)) = heap_ref.peek() {
                if &x > min {
                    heap_ref.push(Reverse(x));
                    if heap_ref.len() > k {
                        heap_ref.pop();
                    }
                }
            }
            cont(())
        }
    };

    reduce(transducer, source, (), reducer);

    // Extract and reverse (to get descending order)
    let final_heap = Rc::try_unwrap(heap)
        .unwrap_or_else(|_| panic!("Failed to unwrap heap"))
        .into_inner();
    let mut result: Vec<U> = final_heap.into_iter().map(|Reverse(x)| x).collect();
    result.sort_by(|a, b| b.cmp(a)); // Descending order
    result
}

/// Count the frequency of each element.
///
/// Returns a HashMap mapping each unique element to its count.
///
/// # Examples
///
/// ```
/// use orlando_transducers::{frequencies, transducer::Identity};
/// use std::collections::HashMap;
///
/// let data = vec![1, 2, 2, 3, 3, 3];
/// let id = Identity::new();
/// let freq = frequencies(&id, data);
///
/// assert_eq!(freq.get(&1), Some(&1));
/// assert_eq!(freq.get(&2), Some(&2));
/// assert_eq!(freq.get(&3), Some(&3));
/// ```
///
/// ```
/// use orlando_transducers::{frequencies, Map};
///
/// let pipeline = Map::new(|s: &str| s.to_lowercase());
/// let words = vec!["Hello", "World", "hello", "WORLD"];
/// let freq = frequencies(&pipeline, words);
///
/// assert_eq!(freq.get("hello"), Some(&2));
/// assert_eq!(freq.get("world"), Some(&2));
/// ```
pub fn frequencies<T, U, Iter>(
    transducer: &impl Transducer<T, U>,
    source: Iter,
) -> HashMap<U, usize>
where
    T: 'static,
    U: Eq + Hash + Clone + 'static,
    Iter: IntoIterator<Item = T>,
{
    let reducer = |mut acc: HashMap<U, usize>, x: U| {
        *acc.entry(x).or_insert(0) += 1;
        cont(acc)
    };

    reduce(transducer, source, HashMap::new(), reducer)
}

/// Zip two iterators, continuing until both are exhausted (unlike `zip`).
///
/// When one iterator is shorter, uses the provided fill value for missing elements.
///
/// # Examples
///
/// ```
/// use orlando_transducers::zip_longest;
///
/// let a = vec![1, 2, 3];
/// let b = vec![4, 5];
/// let result = zip_longest(a, b, 0, 0);
///
/// assert_eq!(result, vec![(1, 4), (2, 5), (3, 0)]);
/// ```
///
/// ```
/// use orlando_transducers::zip_longest;
///
/// let short = vec![1, 2];
/// let long = vec![10, 20, 30, 40];
/// let result = zip_longest(short, long, 999, 0);
///
/// assert_eq!(result, vec![(1, 10), (2, 20), (999, 30), (999, 40)]);
/// ```
pub fn zip_longest<T, U, IterT, IterU>(
    iter_a: IterT,
    iter_b: IterU,
    fill_a: T,
    fill_b: U,
) -> Vec<(T, U)>
where
    T: Clone,
    U: Clone,
    IterT: IntoIterator<Item = T>,
    IterU: IntoIterator<Item = U>,
{
    let mut iter_a = iter_a.into_iter();
    let mut iter_b = iter_b.into_iter();
    let mut result = Vec::new();

    loop {
        match (iter_a.next(), iter_b.next()) {
            (Some(a), Some(b)) => result.push((a, b)),
            (Some(a), None) => result.push((a, fill_b.clone())),
            (None, Some(b)) => result.push((fill_a.clone(), b)),
            (None, None) => break,
        }
    }

    result
}

/// Compute the cartesian product of two iterators.
///
/// Returns all possible pairs (a, b) where a is from the first iterator
/// and b is from the second iterator.
///
/// # Examples
///
/// ```
/// use orlando_transducers::cartesian_product;
///
/// let colors = vec!["red", "blue"];
/// let sizes = vec!["S", "M", "L"];
/// let products = cartesian_product(colors, sizes);
///
/// assert_eq!(products.len(), 6);
/// assert!(products.contains(&("red", "S")));
/// assert!(products.contains(&("blue", "L")));
/// ```
///
/// ```
/// use orlando_transducers::cartesian_product;
///
/// let a = vec![1, 2];
/// let b = vec![3, 4];
/// let result = cartesian_product(a, b);
///
/// assert_eq!(result, vec![(1, 3), (1, 4), (2, 3), (2, 4)]);
/// ```
pub fn cartesian_product<T, U, IterT, IterU>(iter_a: IterT, iter_b: IterU) -> Vec<(T, U)>
where
    T: Clone,
    U: Clone,
    IterT: IntoIterator<Item = T>,
    IterU: IntoIterator<Item = U>,
{
    let vec_a: Vec<T> = iter_a.into_iter().collect();
    let vec_b: Vec<U> = iter_b.into_iter().collect();

    let mut result = Vec::with_capacity(vec_a.len() * vec_b.len());

    for a in &vec_a {
        for b in &vec_b {
            result.push((a.clone(), b.clone()));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::{Filter, Map};

    #[test]
    fn test_to_vec() {
        let double = Map::new(|x: i32| x * 2);
        let result = to_vec(&double, vec![1, 2, 3]);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_sum() {
        let double = Map::new(|x: i32| x * 2);
        let result = sum(&double, vec![1, 2, 3]);
        assert_eq!(result, 12);
    }

    #[test]
    fn test_count() {
        let evens = Filter::new(|x: &i32| x % 2 == 0);
        let result = count(&evens, vec![1, 2, 3, 4, 5]);
        assert_eq!(result, 2);
    }

    #[test]
    fn test_first() {
        let evens = Filter::new(|x: &i32| x % 2 == 0);
        let result = first(&evens, vec![1, 3, 4, 5]);
        assert_eq!(result, Some(4));
    }

    #[test]
    fn test_every() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        assert!(every(&id, vec![2, 4, 6], |x| x % 2 == 0));
        assert!(!every(&id, vec![2, 3, 6], |x| x % 2 == 0));
    }

    #[test]
    fn test_partition() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let (evens, odds) = partition(&id, vec![1, 2, 3, 4, 5], |x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4]);
        assert_eq!(odds, vec![1, 3, 5]);
    }

    #[test]
    fn test_partition_with_transform() {
        // Partition after transformation
        let double = Map::new(|x: i32| x * 2);
        let (greater, lesser) = partition(&double, vec![1, 2, 3, 4, 5], |x| *x > 5);
        assert_eq!(greater, vec![6, 8, 10]); // doubled: 3->6, 4->8, 5->10
        assert_eq!(lesser, vec![2, 4]); // doubled: 1->2, 2->4
    }

    #[test]
    fn test_partition_all_pass() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let (pass, fail) = partition(&id, vec![2, 4, 6], |x| x % 2 == 0);
        assert_eq!(pass, vec![2, 4, 6]);
        assert_eq!(fail, Vec::<i32>::new());
    }

    #[test]
    fn test_partition_all_fail() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let (pass, fail) = partition(&id, vec![1, 3, 5], |x| x % 2 == 0);
        assert_eq!(pass, Vec::<i32>::new());
        assert_eq!(fail, vec![1, 3, 5]);
    }

    #[test]
    fn test_find() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let result = find(&id, vec![1, 3, 4, 5], |x| x % 2 == 0);
        assert_eq!(result, Some(4));
    }

    #[test]
    fn test_find_with_transform() {
        let double = Map::new(|x: i32| x * 2);
        let result = find(&double, vec![1, 2, 3, 4, 5], |x| *x > 5);
        assert_eq!(result, Some(6)); // 3*2 = 6, first element >5
    }

    #[test]
    fn test_find_not_found() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let result = find(&id, vec![1, 3, 5, 7], |x| x % 2 == 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_empty() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let result = find(&id, Vec::<i32>::new(), |x| x % 2 == 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_group_by() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let groups = group_by(&id, vec![1, 2, 3, 4, 5, 6], |x| x % 3);

        assert_eq!(groups.get(&0), Some(&vec![3, 6]));
        assert_eq!(groups.get(&1), Some(&vec![1, 4]));
        assert_eq!(groups.get(&2), Some(&vec![2, 5]));
    }

    #[test]
    fn test_group_by_with_transform() {
        // Group after doubling
        let double = Map::new(|x: i32| x * 2);
        let groups = group_by(&double, vec![1, 2, 3, 4, 5, 6], |x| x % 4);

        assert_eq!(groups.get(&0), Some(&vec![4, 8, 12])); // 2*2=4, 4*2=8, 6*2=12
        assert_eq!(groups.get(&2), Some(&vec![2, 6, 10])); // 1*2=2, 3*2=6, 5*2=10
    }

    #[test]
    fn test_group_by_empty() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let groups = group_by(&id, Vec::<i32>::new(), |x| x % 3);

        assert!(groups.is_empty());
    }

    #[test]
    fn test_group_by_single_group() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        let groups = group_by(&id, vec![3, 6, 9, 12], |x| x % 3);

        // All should be in group 0
        assert_eq!(groups.get(&0), Some(&vec![3, 6, 9, 12]));
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_none() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        assert!(none(&id, vec![1, 3, 5, 7], |x| x % 2 == 0)); // No evens
        assert!(!none(&id, vec![1, 2, 3], |x| x % 2 == 0)); // Has evens
    }

    #[test]
    fn test_none_empty() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        assert!(none(&id, vec![], |_x| true)); // Empty collection = none match
    }

    #[test]
    fn test_none_with_transducer() {
        use crate::transforms::Map;
        let pipeline = Map::new(|x: i32| x * 2);
        assert!(none(&pipeline, vec![1, 2, 3], |x| *x > 10));
        assert!(!none(&pipeline, vec![1, 2, 6], |x| *x > 10)); // 6*2 = 12 > 10
    }

    #[test]
    fn test_none_all_match() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        // None should return false when all elements match
        assert!(!none(&id, vec![2, 4, 6, 8], |x| x % 2 == 0));
    }

    #[test]
    fn test_contains() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        assert!(contains(&id, vec![1, 2, 3, 4, 5], &3));
        assert!(!contains(&id, vec![1, 2, 4, 5], &3));
    }

    #[test]
    fn test_contains_empty() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        assert!(!contains(&id, vec![], &42));
    }

    #[test]
    fn test_contains_first_element() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        assert!(contains(&id, vec![1, 2, 3], &1));
    }

    #[test]
    fn test_contains_last_element() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        assert!(contains(&id, vec![1, 2, 3, 4, 5], &5));
    }

    #[test]
    fn test_contains_with_transducer() {
        use crate::transforms::Map;
        let pipeline = Map::new(|x: i32| x * 2);
        assert!(contains(&pipeline, vec![1, 2, 3], &4)); // 2 * 2 = 4
        assert!(!contains(&pipeline, vec![1, 2, 3], &5)); // No element maps to 5
    }

    #[test]
    fn test_zip() {
        let a = vec![1, 2, 3];
        let b = vec!['a', 'b', 'c', 'd'];
        let result = zip(a, b);
        assert_eq!(result, vec![(1, 'a'), (2, 'b'), (3, 'c')]);
    }

    #[test]
    fn test_zip_equal_length() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let result = zip(a, b);
        assert_eq!(result, vec![(1, 4), (2, 5), (3, 6)]);
    }

    #[test]
    fn test_zip_with() {
        let a = vec![1, 2, 3];
        let b = vec![10, 20, 30];
        let result = zip_with(a, b, |x, y| x + y);
        assert_eq!(result, vec![11, 22, 33]);
    }

    #[test]
    fn test_zip_with_different_types() {
        let numbers = vec![1, 2, 3];
        let strings = vec!["a", "b", "c"];
        let result = zip_with(numbers, strings, |n, s| format!("{}{}", n, s));
        assert_eq!(result, vec!["1a", "2b", "3c"]);
    }

    // Phase 2a: Multi-Input Operations Tests

    #[test]
    fn test_merge_two_equal_length() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let result = merge(vec![a, b]);
        assert_eq!(result, vec![1, 4, 2, 5, 3, 6]);
    }

    #[test]
    fn test_merge_different_lengths() {
        let a = vec![1, 2];
        let b = vec![3, 4, 5, 6];
        let result = merge(vec![a, b]);
        assert_eq!(result, vec![1, 3, 2, 4, 5, 6]);
    }

    #[test]
    fn test_merge_three_streams() {
        let a = vec![1, 2];
        let b = vec![3, 4];
        let c = vec![5, 6];
        let result = merge(vec![a, b, c]);
        assert_eq!(result, vec![1, 3, 5, 2, 4, 6]);
    }

    #[test]
    fn test_merge_empty_stream() {
        let a = vec![1, 2, 3];
        let b: Vec<i32> = vec![];
        let result = merge(vec![a, b]);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_merge_single_stream() {
        let a = vec![1, 2, 3];
        let result = merge(vec![a]);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_merge_with_transducers() {
        // Hybrid composition: process streams, then merge
        let pipeline_a = Map::new(|x: i32| x * 2);
        let pipeline_b = Map::new(|x: i32| x + 10);

        let a_result = to_vec(&pipeline_a, vec![1, 2, 3]);
        let b_result = to_vec(&pipeline_b, vec![1, 2, 3]);

        let merged = merge(vec![a_result, b_result]);
        assert_eq!(merged, vec![2, 11, 4, 12, 6, 13]);
    }

    #[test]
    fn test_intersection_basic() {
        let a = vec![1, 2, 3, 4];
        let b = vec![3, 4, 5, 6];
        let result = intersection(a, b);
        assert_eq!(result, vec![3, 4]);
    }

    #[test]
    fn test_intersection_no_overlap() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let result: Vec<i32> = intersection(a, b);
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_intersection_complete_overlap() {
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3];
        let result = intersection(a, b);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_intersection_preserves_duplicates() {
        let a = vec![1, 2, 2, 3];
        let b = vec![2, 3, 4];
        let result = intersection(a, b);
        assert_eq!(result, vec![2, 2, 3]);
    }

    #[test]
    fn test_intersection_with_transducers() {
        // Hybrid composition: process then intersect
        let pipeline = Map::new(|x: i32| x * 2);
        let a_processed = to_vec(&pipeline, vec![1, 2, 3, 4]);
        let b_processed = to_vec(&pipeline, vec![3, 4, 5, 6]);

        let result = intersection(a_processed, b_processed);
        assert_eq!(result, vec![6, 8]); // 3*2=6, 4*2=8
    }

    #[test]
    fn test_difference_basic() {
        let a = vec![1, 2, 3, 4];
        let b = vec![3, 4, 5, 6];
        let result = difference(a, b);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_difference_no_overlap() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let result = difference(a, b);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_difference_complete_overlap() {
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3];
        let result: Vec<i32> = difference(a, b);
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_difference_removes_all_occurrences() {
        let a = vec![1, 2, 2, 3];
        let b = vec![2];
        let result = difference(a, b);
        assert_eq!(result, vec![1, 3]);
    }

    #[test]
    fn test_union_basic() {
        let a = vec![1, 2, 3];
        let b = vec![3, 4, 5];
        let result = union(a, b);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_union_no_overlap() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let result = union(a, b);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_union_complete_overlap() {
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3];
        let result = union(a, b);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_union_removes_duplicates() {
        let a = vec![1, 2, 2, 3];
        let b = vec![3, 4, 4, 5];
        let result = union(a, b);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_union_empty() {
        let a: Vec<i32> = vec![];
        let b = vec![1, 2, 3];
        let result = union(a, b);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_symmetric_difference_basic() {
        let a = vec![1, 2, 3, 4];
        let b = vec![3, 4, 5, 6];
        let result = symmetric_difference(a, b);
        assert_eq!(result, vec![1, 2, 5, 6]);
    }

    #[test]
    fn test_symmetric_difference_no_overlap() {
        let a = vec![1, 2];
        let b = vec![3, 4];
        let result = symmetric_difference(a, b);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_symmetric_difference_complete_overlap() {
        let a = vec![1, 2, 3];
        let b = vec![1, 2, 3];
        let result: Vec<i32> = symmetric_difference(a, b);
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_symmetric_difference_preserves_order() {
        let a = vec![4, 3, 2, 1];
        let b = vec![6, 5, 2, 1];
        let result = symmetric_difference(a, b);
        assert_eq!(result, vec![4, 3, 6, 5]);
    }

    #[test]
    fn test_symmetric_difference_duplicates() {
        let a = vec![1, 1, 2, 3];
        let b = vec![3, 4, 4];
        let result = symmetric_difference(a, b);
        assert_eq!(result, vec![1, 2, 4]);
    }
}
