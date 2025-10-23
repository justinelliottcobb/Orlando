//! Terminal operations (collectors) for transducers.
//!
//! Collectors are reducing functions that consume the output of a transducer
//! and produce a final result.

use crate::step::{cont, Step};
use crate::transducer::Transducer;
use std::collections::HashMap;
use std::hash::Hash;

/// Execute a transducer over an iterator and collect results into a vector.
///
/// # Examples
///
/// ```
/// use orlando::collectors::to_vec;
/// use orlando::transforms::Map;
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
/// use orlando::collectors::reduce;
/// use orlando::transforms::Map;
/// use orlando::step::cont;
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
/// use orlando::collectors::sum;
/// use orlando::transforms::Map;
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
/// use orlando::collectors::count;
/// use orlando::transforms::Filter;
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
/// use orlando::collectors::first;
/// use orlando::transforms::Filter;
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
/// use orlando::collectors::last;
/// use orlando::transforms::Filter;
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
/// use orlando::collectors::every;
/// use orlando::transducer::Identity;
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
/// use orlando::collectors::some;
/// use orlando::transducer::Identity;
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
/// use orlando::collectors::partition;
/// use orlando::transducer::Identity;
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
/// use orlando::collectors::find;
/// use orlando::transducer::Identity;
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
/// use orlando::collectors::group_by;
/// use orlando::transducer::Identity;
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
/// use orlando::collectors::none;
/// use orlando::transducer::Identity;
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
/// use orlando::collectors::contains;
/// use orlando::transducer::Identity;
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
/// use orlando::collectors::zip;
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
/// use orlando::collectors::zip_with;
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
    fn test_contains() {
        use crate::transducer::Identity;
        let id = Identity::<i32>::new();
        assert!(contains(&id, vec![1, 2, 3, 4, 5], &3));
        assert!(!contains(&id, vec![1, 2, 4, 5], &3));
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
}
