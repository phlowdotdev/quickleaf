//! Prefetch hints for better memory access patterns and cache locality
//!
//! This module provides memory prefetch optimizations to improve cache performance
//! by giving the CPU hints about what memory will be accessed soon.

/// Prefetch operations for memory access optimization
pub struct Prefetch;

impl Prefetch {
    /// Prefetch memory for read access (non-temporal)
    ///
    /// This hints to the processor that the memory location will be read soon.
    /// Uses PREFETCH_T0 which loads data to all cache levels.
    #[inline(always)]
    pub fn read_hint<T>(ptr: *const T) {
        if cfg!(target_arch = "x86_64") || cfg!(target_arch = "x86") {
            unsafe {
                #[cfg(target_arch = "x86_64")]
                core::arch::x86_64::_mm_prefetch(ptr as *const i8, core::arch::x86_64::_MM_HINT_T0);

                #[cfg(target_arch = "x86")]
                core::arch::x86::_mm_prefetch(ptr as *const i8, core::arch::x86::_MM_HINT_T0);
            }
        }
    }

    /// Prefetch multiple sequential memory locations
    ///
    /// This is useful for prefetching array-like structures or linked data.
    /// Prefetches in 64-byte cache line chunks.
    #[inline(always)]
    pub fn sequential_read_hints<T>(start_ptr: *const T, count: usize) {
        if cfg!(target_arch = "x86_64") || cfg!(target_arch = "x86") {
            let stride = 64;
            let elem_size = std::mem::size_of::<T>();
            let total_bytes = count * elem_size;

            for offset in (0..total_bytes).step_by(stride) {
                unsafe {
                    let prefetch_ptr = (start_ptr as *const u8).add(offset);

                    #[cfg(target_arch = "x86_64")]
                    core::arch::x86_64::_mm_prefetch(
                        prefetch_ptr as *const i8,
                        core::arch::x86_64::_MM_HINT_T0,
                    );

                    #[cfg(target_arch = "x86")]
                    core::arch::x86::_mm_prefetch(
                        prefetch_ptr as *const i8,
                        core::arch::x86::_MM_HINT_T0,
                    );
                }
            }
        }
    }
}

/// Helper trait to add prefetch methods to common types
pub trait PrefetchExt<T> {
    /// Prefetch this memory location for read access
    fn prefetch_read(&self);
}

impl<T> PrefetchExt<T> for *const T {
    #[inline(always)]
    fn prefetch_read(&self) {
        Prefetch::read_hint(*self);
    }
}

impl<T> PrefetchExt<T> for *mut T {
    #[inline(always)]
    fn prefetch_read(&self) {
        Prefetch::read_hint(*self as *const T);
    }
}

impl<T> PrefetchExt<T> for &T {
    #[inline(always)]
    fn prefetch_read(&self) {
        Prefetch::read_hint(*self as *const T);
    }
}

impl<T> PrefetchExt<T> for &mut T {
    #[inline(always)]
    fn prefetch_read(&self) {
        Prefetch::read_hint(*self as *const T);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_hints() {
        let data = vec![1, 2, 3, 4, 5];

        Prefetch::read_hint(data.as_ptr());

        Prefetch::sequential_read_hints(data.as_ptr(), data.len());
    }

    #[test]
    fn test_prefetch_ext_trait() {
        let data = vec![1, 2, 3, 4, 5];
        let ptr = data.as_ptr();

        ptr.prefetch_read();

        let val = 42;
        (&val).prefetch_read();
    }
}
