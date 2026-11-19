//! Process-wide resource limits for untrusted pages.
//!
//! [`LimitedAlloc`] counts heap bytes and ends the process cleanly when a
//! configured limit is crossed. JavaScript has no heap limit in Boa, so this is
//! the only way to stop an allocation bomb before the operating system's
//! out-of-memory handling does.
//!
//! The binary installs it with `#[global_allocator]` and sets the limit through
//! [`set_memory_limit`]. Until a limit is set, it only counts.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Exit status when the memory limit is exceeded.
pub const EXIT_MEMORY_LIMIT: i32 = 4;
/// Exit status when JavaScript does not finish within its wall-clock limit.
pub const EXIT_JS_TIMEOUT: i32 = 3;

static LIMIT: AtomicUsize = AtomicUsize::new(usize::MAX);
static IN_USE: AtomicUsize = AtomicUsize::new(0);

/// Limit live heap allocations to `bytes`. `None` removes the limit.
pub fn set_memory_limit(bytes: Option<usize>) {
    LIMIT.store(bytes.unwrap_or(usize::MAX), Ordering::Relaxed);
}

/// Bytes currently allocated on the heap through [`LimitedAlloc`].
pub fn heap_in_use() -> usize {
    IN_USE.load(Ordering::Relaxed)
}

/// The system allocator with a byte counter and an upper limit.
pub struct LimitedAlloc;

#[inline]
fn reserve(bytes: usize) {
    let before = IN_USE.fetch_add(bytes, Ordering::Relaxed);
    if before.saturating_add(bytes) > LIMIT.load(Ordering::Relaxed) {
        IN_USE.fetch_sub(bytes, Ordering::Relaxed);
        memory_limit_exceeded();
    }
}

#[inline]
fn release(bytes: usize) {
    IN_USE.fetch_sub(bytes, Ordering::Relaxed);
}

/// Ends the process. Runs inside the allocator, so it must not allocate,
/// unwind or run destructors.
#[cold]
fn memory_limit_exceeded() -> ! {
    const MSG: &[u8] = b"error: memory limit exceeded (see --max-memory)\n";
    #[cfg(unix)]
    unsafe {
        libc::write(2, MSG.as_ptr().cast(), MSG.len());
        libc::_exit(EXIT_MEMORY_LIMIT)
    }
    #[cfg(not(unix))]
    {
        let _ = MSG;
        std::process::abort()
    }
}

unsafe impl GlobalAlloc for LimitedAlloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        reserve(layout.size());
        let p = unsafe { System.alloc(layout) };
        if p.is_null() {
            release(layout.size());
        }
        p
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        reserve(layout.size());
        let p = unsafe { System.alloc_zeroed(layout) };
        if p.is_null() {
            release(layout.size());
        }
        p
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        release(layout.size());
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old = layout.size();
        if new_size > old {
            reserve(new_size - old);
        }
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if p.is_null() {
            if new_size > old {
                release(new_size - old);
            }
        } else if new_size < old {
            release(old - new_size);
        }
        p
    }
}
