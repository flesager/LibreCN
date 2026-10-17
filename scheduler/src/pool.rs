//! Preallocated message buffers, one pool per mailbox.
//!
//! Each size class is one contiguous, 64-byte-aligned allocation written once
//! at creation (prefault), plus a free list of buffer indices. A body takes
//! the smallest class that fits; when that class is empty it falls back to
//! the next larger ones. Fallbacks and failures are counted on the class the
//! body asked for, so the metrics say which class is undersized.

use std::alloc::Layout;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::queue::Ring;
use crate::stats::BufferClassStats;
use crate::{Error, Result};

/// Alignment of every buffer; class sizes are rounded up to a multiple of it.
pub const BUFFER_ALIGN: usize = 64;

/// One size class of a mailbox's buffer pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferClassConfig {
    /// Buffer size in bytes, rounded up to a multiple of [`BUFFER_ALIGN`].
    pub size: usize,
    /// Number of buffers.
    pub count: u32,
}

/// Exclusive ownership of one buffer. Deliberately neither `Copy` nor `Clone`.
///
/// Only valid with the pool that produced it; every user is crate-internal and
/// keeps a `BufRef` next to (or inside) the mailbox that owns the pool.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BufRef {
    class: u8,
    index: u32,
}

struct BufferClass {
    size: usize,
    count: u32,
    base: NonNull<u8>,
    layout: Layout,
    free: Ring<u32>,
    in_use: AtomicU32,
    high_water: AtomicU32,
    fallbacks: AtomicU64,
    alloc_failures: AtomicU64,
}

pub(crate) struct BufferPool {
    classes: Box<[BufferClass]>,
}

// SAFETY: the raw buffer memory is only reached through a `BufRef`, and the free
// list hands each index to a single owner at a time.
unsafe impl Send for BufferPool {}
// SAFETY: see `Send`; every other field is atomic or immutable.
unsafe impl Sync for BufferPool {}

impl BufferPool {
    pub(crate) fn new(config: &[BufferClassConfig]) -> Result<Self> {
        let mut config = config.to_vec();
        for c in &mut config {
            if c.size == 0 || c.count == 0 {
                return Err(Error::InvalidConfig("buffer class size and count must be > 0"));
            }
            c.size = c.size.div_ceil(BUFFER_ALIGN) * BUFFER_ALIGN;
        }
        config.sort_by_key(|c| c.size);
        if config.windows(2).any(|w| w[0].size == w[1].size) {
            return Err(Error::InvalidConfig("two buffer classes round to the same size"));
        }
        if config.len() > usize::from(u8::MAX) {
            return Err(Error::InvalidConfig("too many buffer classes"));
        }
        let classes = config.iter().map(|c| BufferClass::new(c.size, c.count)).collect::<Result<_>>()?;
        Ok(BufferPool { classes })
    }

    /// Takes a buffer of at least `len` bytes from the smallest class that
    /// fits, or from the next larger class with a free buffer.
    pub(crate) fn alloc(&self, len: usize) -> Result<BufRef> {
        let first = self.classes.iter().position(|c| c.size >= len).ok_or(Error::BodyTooLarge(len))?;
        let requested = &self.classes[first];
        for (class_idx, class) in self.classes.iter().enumerate().skip(first) {
            if let Some(index) = class.free.pop() {
                if class_idx != first {
                    requested.fallbacks.fetch_add(1, Ordering::Relaxed);
                }
                let in_use = class.in_use.fetch_add(1, Ordering::Relaxed) + 1;
                class.high_water.fetch_max(in_use, Ordering::Relaxed);
                return Ok(BufRef { class: class_idx as u8, index });
            }
        }
        requested.alloc_failures.fetch_add(1, Ordering::Relaxed);
        Err(Error::PoolExhausted)
    }

    pub(crate) fn free(&self, buf: BufRef) {
        let class = &self.classes[usize::from(buf.class)];
        class.in_use.fetch_sub(1, Ordering::Relaxed);
        let pushed = class.free.push(buf.index);
        debug_assert!(pushed.is_ok(), "free list sized to the class count cannot overflow");
    }

    fn ptr(&self, buf: &BufRef, len: usize) -> *mut u8 {
        let class = &self.classes[usize::from(buf.class)];
        assert!(len <= class.size, "{len} bytes requested from a {}-byte buffer", class.size);
        debug_assert!(buf.index < class.count);
        // SAFETY: `index < count`, so the offset stays inside the class allocation.
        unsafe { class.base.as_ptr().add(buf.index as usize * class.size) }
    }

    /// First `len` bytes of `buf`.
    pub(crate) fn bytes<'a>(&'a self, buf: &'a BufRef, len: usize) -> &'a [u8] {
        // SAFETY: `ptr` checked the bounds; holding `&BufRef` means nobody writes the buffer.
        unsafe { std::slice::from_raw_parts(self.ptr(buf, len), len) }
    }

    /// First `len` bytes of `buf`, writable.
    pub(crate) fn bytes_mut<'a>(&'a self, buf: &'a mut BufRef, len: usize) -> &'a mut [u8] {
        // SAFETY: `ptr` checked the bounds; `&mut BufRef` is the only handle on this buffer.
        unsafe { std::slice::from_raw_parts_mut(self.ptr(buf, len), len) }
    }

    /// Buffer memory plus free lists.
    pub(crate) fn bytes_reserved(&self) -> usize {
        self.classes.iter().map(|c| c.layout.size() + c.free.bytes_reserved()).sum()
    }

    pub(crate) fn stats(&self, reset_peaks: bool) -> Vec<BufferClassStats> {
        self.classes
            .iter()
            .map(|c| {
                let in_use = c.in_use.load(Ordering::Relaxed);
                let high_water = if reset_peaks {
                    c.high_water.swap(in_use, Ordering::Relaxed)
                } else {
                    c.high_water.load(Ordering::Relaxed)
                };
                BufferClassStats {
                    size: c.size,
                    count: c.count,
                    in_use,
                    high_water,
                    fallbacks: c.fallbacks.load(Ordering::Relaxed),
                    alloc_failures: c.alloc_failures.load(Ordering::Relaxed),
                }
            })
            .collect()
    }
}

impl BufferClass {
    fn new(size: usize, count: u32) -> Result<Self> {
        let total = size.checked_mul(count as usize).ok_or(Error::InvalidConfig("buffer class too large"))?;
        let layout =
            Layout::from_size_align(total, BUFFER_ALIGN).map_err(|_| Error::InvalidConfig("buffer class too large"))?;
        // SAFETY: `layout` has a non-zero size (size and count are > 0).
        let raw = unsafe { std::alloc::alloc(layout) };
        let Some(base) = NonNull::new(raw) else { std::alloc::handle_alloc_error(layout) };
        // Prefault: touch every page now so the first messages never page-fault.
        // SAFETY: `base` points to `total` writable bytes.
        unsafe { std::ptr::write_bytes(base.as_ptr(), 0, total) };
        let free = Ring::new(count as usize);
        for i in 0..count {
            let _ = free.push(i);
        }
        Ok(BufferClass {
            size,
            count,
            base,
            layout,
            free,
            in_use: AtomicU32::new(0),
            high_water: AtomicU32::new(0),
            fallbacks: AtomicU64::new(0),
            alloc_failures: AtomicU64::new(0),
        })
    }
}

impl Drop for BufferClass {
    fn drop(&mut self) {
        // SAFETY: allocated in `new` with this exact layout.
        unsafe { std::alloc::dealloc(self.base.as_ptr(), self.layout) };
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    fn pool(classes: &[(usize, u32)]) -> BufferPool {
        let cfg: Vec<_> = classes.iter().map(|&(size, count)| BufferClassConfig { size, count }).collect();
        BufferPool::new(&cfg).unwrap()
    }

    #[test]
    fn picks_smallest_fitting_class_and_rounds_sizes() {
        let p = pool(&[(2048, 1), (100, 2)]);
        let stats = p.stats(false);
        assert_eq!((stats[0].size, stats[1].size), (128, 2048));
        let small = p.alloc(1).unwrap();
        let large = p.alloc(129).unwrap();
        assert_eq!((small.class, large.class), (0, 1));
        assert_eq!(p.alloc(4096), Err(Error::BodyTooLarge(4096)));
        p.free(small);
        p.free(large);
    }

    #[test]
    fn exhaustion_is_counted_and_buffers_come_back() {
        let p = pool(&[(64, 2)]);
        let a = p.alloc(10).unwrap();
        let b = p.alloc(10).unwrap();
        assert_eq!(p.alloc(10), Err(Error::PoolExhausted));
        let s = p.stats(false)[0];
        assert_eq!((s.in_use, s.high_water, s.alloc_failures), (2, 2, 1));
        p.free(a);
        p.free(b);
        let s = p.stats(true)[0];
        assert_eq!((s.in_use, s.high_water), (0, 2));
        assert_eq!(p.stats(false)[0].high_water, 0, "reset brings the peak down to the current use");
        assert!(p.alloc(10).is_ok());
    }

    #[test]
    fn falls_back_to_larger_classes_and_counts_it_on_the_requested_one() {
        let p = pool(&[(64, 1), (128, 1), (256, 1)]);
        let a = p.alloc(10).unwrap();
        let b = p.alloc(10).unwrap();
        let c = p.alloc(10).unwrap();
        assert_eq!((a.class, b.class, c.class), (0, 1, 2));
        assert_eq!(p.alloc(10), Err(Error::PoolExhausted));
        let s = p.stats(false);
        assert_eq!((s[0].fallbacks, s[0].alloc_failures, s[0].high_water), (2, 1, 1));
        assert_eq!((s[1].fallbacks, s[1].in_use, s[2].in_use), (0, 1, 1));
        p.free(b);
        assert_eq!(p.alloc(100).map(|r| r.class), Ok(1), "a 100-byte body never goes below its class");
        p.free(a);
        p.free(c);
    }

    #[test]
    fn buffers_are_aligned_and_disjoint() {
        let p = pool(&[(64, 4)]);
        let mut bufs: Vec<_> = (0..4).map(|_| p.alloc(64).unwrap()).collect();
        for (i, b) in bufs.iter_mut().enumerate() {
            let bytes = p.bytes_mut(b, 64);
            assert_eq!(bytes.as_ptr() as usize % BUFFER_ALIGN, 0);
            bytes.fill(i as u8);
        }
        for (i, b) in bufs.iter().enumerate() {
            assert!(p.bytes(b, 64).iter().all(|&x| x == i as u8));
        }
        bufs.into_iter().for_each(|b| p.free(b));
    }

    #[test]
    fn rejects_bad_config() {
        let cfg = |size, count| [BufferClassConfig { size, count }];
        assert!(BufferPool::new(&cfg(0, 1)).is_err());
        assert!(BufferPool::new(&cfg(64, 0)).is_err());
        let dup = [BufferClassConfig { size: 60, count: 1 }, BufferClassConfig { size: 64, count: 1 }];
        assert!(BufferPool::new(&dup).is_err());
        assert!(BufferPool::new(&[]).is_ok(), "an EO may accept header-only messages only");
    }
}
