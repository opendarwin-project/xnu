use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

struct BumpAlloc {
    offset: AtomicUsize,
    heap: UnsafeCell<[u8; 128 * 1024]>,
}

unsafe impl Sync for BumpAlloc {}

unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let heap_ptr = self.heap.get() as *mut u8;
        loop {
            let cur = self.offset.load(Ordering::Relaxed);
            let heap_start = heap_ptr as usize;
            let current_ptr = heap_start + cur;
            let aligned_ptr = (current_ptr + align - 1) & !(align - 1);
            let new_offset = (aligned_ptr - heap_start) + size;
            if new_offset > 128 * 1024 {
                return core::ptr::null_mut();
            }
            if self
                .offset
                .compare_exchange_weak(cur, new_offset, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return aligned_ptr as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAlloc = BumpAlloc {
    offset: AtomicUsize::new(0),
    heap: UnsafeCell::new([0; 128 * 1024]),
};
