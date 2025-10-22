//! SIMD-accelerated operations for transducers.
//!
//! This module provides vectorized implementations for common operations
//! when working with numeric data. It uses WASM SIMD instructions for
//! high performance.

#[cfg(target_arch = "wasm32")]
use std::arch::wasm32::*;

/// Threshold for using SIMD operations (in elements)
const SIMD_THRESHOLD: usize = 64;

/// SIMD-accelerated map for f64 arrays.
///
/// This function applies a transformation to each element of a f64 slice
/// using SIMD instructions when the data is large enough.
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn map_f64_simd<F>(data: &[f64], f: F) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    if data.len() < SIMD_THRESHOLD {
        // Fall back to scalar for small arrays
        return data.iter().map(|&x| f(x)).collect();
    }

    let mut result = Vec::with_capacity(data.len());
    let chunks = data.chunks_exact(2);
    let remainder = chunks.remainder();

    // Process 2 f64s at a time with SIMD
    for chunk in chunks {
        unsafe {
            // Load 2 f64 values
            let v = v128_load(chunk.as_ptr() as *const v128);
            
            // Extract lanes, apply function, rebuild vector
            let lane0 = f64x2_extract_lane::<0>(v);
            let lane1 = f64x2_extract_lane::<1>(v);
            
            let r0 = f(lane0);
            let r1 = f(lane1);
            
            result.push(r0);
            result.push(r1);
        }
    }

    // Process remainder
    for &x in remainder {
        result.push(f(x));
    }

    result
}

/// Non-SIMD fallback for map_f64
#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn map_f64_simd<F>(data: &[f64], f: F) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    data.iter().map(|&x| f(x)).collect()
}

/// SIMD-accelerated filter for f64 arrays.
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn filter_f64_simd<P>(data: &[f64], predicate: P) -> Vec<f64>
where
    P: Fn(f64) -> bool,
{
    if data.len() < SIMD_THRESHOLD {
        return data.iter().copied().filter(|&x| predicate(x)).collect();
    }

    // For filter, SIMD doesn't help much since we need to check each element
    // individually and build a variable-length result. Fall back to scalar.
    data.iter().copied().filter(|&x| predicate(x)).collect()
}

/// Non-SIMD fallback for filter_f64
#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn filter_f64_simd<P>(data: &[f64], predicate: P) -> Vec<f64>
where
    P: Fn(f64) -> bool,
{
    data.iter().copied().filter(|&x| predicate(x)).collect()
}

/// SIMD-accelerated sum for f64 arrays.
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn sum_f64_simd(data: &[f64]) -> f64 {
    if data.len() < SIMD_THRESHOLD {
        return data.iter().sum();
    }

    let chunks = data.chunks_exact(2);
    let remainder = chunks.remainder();

    unsafe {
        // Accumulator vector (2 f64s)
        let mut acc = f64x2_splat(0.0);

        for chunk in chunks {
            let v = v128_load(chunk.as_ptr() as *const v128);
            acc = f64x2_add(acc, v);
        }

        // Extract and sum the two lanes
        let lane0 = f64x2_extract_lane::<0>(acc);
        let lane1 = f64x2_extract_lane::<1>(acc);
        let mut total = lane0 + lane1;

        // Add remainder
        for &x in remainder {
            total += x;
        }

        total
    }
}

/// Non-SIMD fallback for sum_f64
#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn sum_f64_simd(data: &[f64]) -> f64 {
    data.iter().sum()
}

/// SIMD-accelerated multiply for f64 arrays (element-wise).
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn mul_f64_simd(a: &[f64], b: &[f64]) -> Vec<f64> {
    assert_eq!(a.len(), b.len());
    
    if a.len() < SIMD_THRESHOLD {
        return a.iter().zip(b.iter()).map(|(&x, &y)| x * y).collect();
    }

    let mut result = Vec::with_capacity(a.len());
    let chunks_a = a.chunks_exact(2);
    let chunks_b = b.chunks_exact(2);
    let remainder_a = chunks_a.remainder();
    let remainder_b = chunks_b.remainder();

    unsafe {
        for (chunk_a, chunk_b) in chunks_a.zip(chunks_b) {
            let va = v128_load(chunk_a.as_ptr() as *const v128);
            let vb = v128_load(chunk_b.as_ptr() as *const v128);
            let vc = f64x2_mul(va, vb);
            
            result.push(f64x2_extract_lane::<0>(vc));
            result.push(f64x2_extract_lane::<1>(vc));
        }
    }

    for (&x, &y) in remainder_a.iter().zip(remainder_b.iter()) {
        result.push(x * y);
    }

    result
}

/// Non-SIMD fallback for mul_f64
#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn mul_f64_simd(a: &[f64], b: &[f64]) -> Vec<f64> {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(&x, &y)| x * y).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_f64_simd() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = map_f64_simd(&data, |x| x * 2.0);
        assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_filter_f64_simd() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = filter_f64_simd(&data, |x| x > 2.5);
        assert_eq!(result, vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_sum_f64_simd() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = sum_f64_simd(&data);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_mul_f64_simd() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![2.0, 3.0, 4.0, 5.0];
        let result = mul_f64_simd(&a, &b);
        assert_eq!(result, vec![2.0, 6.0, 12.0, 20.0]);
    }
}
