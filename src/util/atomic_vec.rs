#![allow(unused)]

use std::alloc::{ Layout, Allocator, Global };
use std::ops::{ Deref, DerefMut };
use std::sync::atomic::{ AtomicBool, AtomicPtr, AtomicUsize, AtomicU8, Ordering };

/// A thread-safe [`Vec`], allowing for immutable *and mutable* references to
/// singular elements that don't block access to other elements in the array.
/// 
/// # How it works
/// *Basically a lot of dark magic.*
/// 
/// It's basic structure resembles that of a [`Vec`]: A pointer to a
/// heap-allocated portion of memory for storing the elements and a the current
/// length and capacity. However, there also are some differences:
/// 
/// 1. Every element also stores an [`AtomicU8`] as an atomic reference counter.
/// That way, every element is basically it's own [`Arc`][std::sync::Arc]. This
/// allows for multiple mutable and immutable references to singular elements of
/// the array.
/// 
/// 2. The [`AtomicVec`] also has a counter of all total accessors (the
/// guard-like structs you use to interface with the array's elements), as well
/// as some more flags for concurrency-checks (like preventing pushing an
/// element to the array while also clearing it).
/// 
/// Generally, you can think of it as an `Arc<[Arc<RwLock<T>>]>` with extra
/// functionality (e.g. you wouldn't be able to add elements to the slice, while
/// the [`AtomicVec`] type allows doing that).
#[derive(Debug)]
pub struct AtomicVec<T> {
    pub(self) ptr: AtomicPtr<(T, AtomicU8)>,
    pub(self) len: AtomicUsize,
    pub(self) capacity: AtomicUsize,
    pub(self) accessors: AtomicUsize,
    layout: AtomicPtr<Layout>,
    resizing: AtomicBool,
    pushing: AtomicBool,
    clearing: AtomicBool,
}

impl<T> AtomicVec<T> {
    const LAYOUT: Layout = std::alloc::Layout::new::<(T, AtomicU8)>();

    pub fn new(initial_capacity: usize) -> anyhow::Result<Self> {
        // Minimum capacity of four elements due to how the resize() method works.
        let (layout, _) = Self::LAYOUT.repeat(initial_capacity.max(4))?;

        let nonnull = Global::default().allocate(layout)?;
        let capacity = nonnull.len() / std::mem::size_of::<(T, AtomicU8)>();
        let ptr = AtomicPtr::new(nonnull.as_mut_ptr() as *mut (T, AtomicU8));
        Ok(Self {
            ptr,
            len: AtomicUsize::new(0),
            capacity: AtomicUsize::new(capacity),
            accessors: AtomicUsize::new(0),
            layout: AtomicPtr::new(Box::leak(Box::new(layout)) as *mut Layout),
            resizing: AtomicBool::new(false),
            pushing: AtomicBool::new(false),
            clearing: AtomicBool::new(false),
        })
    }

    pub fn get_mut<'a>(&'a self, index: usize) -> Option<AtomicVecGuardMut<'a, T>> {
        while self.resizing.load(Ordering::Acquire) || self.clearing.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }

        if index>=self.len.load(Ordering::SeqCst) {
            None
        } else {
            let acc_atomic = unsafe { &self.get_elem_unsafe(index, Ordering::SeqCst).1 };
            let accessors = acc_atomic.load(Ordering::Acquire);
            // This is true if there are any number of immutable accessors or a mutable accessor.
            if accessors > 0 {
                None
            } else {
                acc_atomic.store(u8::from_le(0b10000000), Ordering::Release);
                Some(AtomicVecGuardMut::new(&self, index))
            }
        }
    }

    pub fn get<'a>(&'a self, index: usize) -> Option<AtomicVecGuard<'a, T>> {
        while self.resizing.load(Ordering::Acquire) || self.clearing.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }

        if index>=self.len.load(Ordering::SeqCst) {
            None
        } else {
            let acc_atomic = unsafe { &self.get_elem_unsafe(index, Ordering::SeqCst).1 };
            let accessors = acc_atomic.load(Ordering::Acquire);
            // This is true if there is a mutable accessor (or if there are
            // 128+ references to one singular value, but that hopefully
            // shouldn't happen).
            if accessors.to_le() & 0b10000000 > 0 {
                None
            } else {
                acc_atomic.fetch_add(1, Ordering::SeqCst);
                Some(AtomicVecGuard::new(&self, index))
            }
        }
    }

    fn mutable_operation_ongoing(&self) -> bool {
        self.resizing.load(Ordering::Acquire) || self.pushing.load(Ordering::Acquire) || self.clearing.load(Ordering::Acquire)
    }

    fn resize(&self) -> anyhow::Result<()> {
        if self.resizing.load(Ordering::Acquire) {
            // We're already resizing, no need to do it twice.
            return Ok(())
        }
        self.resizing.store(true, Ordering::Release);

        // Spin until no accessors to the vec exist.
        // If we wouldn't do this, we would have to somehow know that nobody is
        // dereferencing an AtomicVecGuard(Mut) while this function executes to
        // prevent data races and SAFE REFERENCES to POSSIBLY UNALLOCATED MEMORY.
        // Rust doesn't allow you to do that though, so this is the best way of
        // doing this. This means that you can't have references to items in the
        // array while pushing to exceed the vec's max capacity though, which
        // could slow things down (if you don't set the capacity smartly).
        while self.accessors.load(Ordering::Acquire)>0 {
            std::hint::spin_loop();
        }

        let newlayout = Self::LAYOUT.repeat(self.capacity.load(Ordering::Acquire) * 2)?.0;
        let oldlayout = unsafe { Box::from_raw(self.layout.load(Ordering::Acquire)) };
        let oldptr = self.ptr.load(Ordering::Acquire);
        let newnonnull = unsafe { Global::default().grow(std::ptr::NonNull::new(oldptr as *mut u8).unwrap(), *oldlayout, newlayout) }?;

        self.ptr.store(newnonnull.as_mut_ptr() as *mut (T, AtomicU8), Ordering::Release);
        self.layout.store(Box::leak(Box::new(newlayout)) as *mut _, Ordering::Release);
        
        let newcapacity = newnonnull.len();
        self.capacity.store(newcapacity, Ordering::Release);
        
        self.resizing.store(false, Ordering::Release);
        Ok(())
    }

    pub fn push(&self, value: T) -> anyhow::Result<usize> {
        while self.mutable_operation_ongoing() {
            std::hint::spin_loop();
        }
        self.pushing.store(true, Ordering::Release);

        if self.len.load(Ordering::SeqCst)+1 > self.capacity.load(Ordering::SeqCst) {
            self.resize()?;
        }
        let len = self.len.load(Ordering::Acquire);
        self.len.store(len+1, Ordering::Release);
        unsafe {
            self.ptr.load(Ordering::Acquire).add(len).write_volatile((value, AtomicU8::new(0)))
        }
        
        self.pushing.store(false, Ordering::Release);
        Ok(len)
    }

    /// Clears the whole [`AtomicVec`], returning a regular [`Vec`] of it's
    /// previous contents.
    pub fn clear(&self) -> Vec<T> {
        while self.accessors.load(Ordering::Acquire)>0 || self.mutable_operation_ongoing() {
            std::hint::spin_loop();
        }
        self.clearing.store(true, Ordering::Release);

        let mut vec = Vec::new();

        for i in 0..self.len.load(Ordering::Acquire) {
            let (elem, _) = Box::into_inner(unsafe { Box::from_raw(self.get_elem_unsafe(i, Ordering::Acquire) as *mut (T, AtomicU8)) });

            vec.push(elem);
        }

        self.len.store(0, Ordering::Release);
        self.clearing.store(false, Ordering::Release);

        return vec
    }

    pub(self) unsafe fn get_elem_unsafe(&self, index: usize, ordering: Ordering) -> &mut (T, AtomicU8) {
        self.ptr.load(ordering).add(index).as_mut().unwrap()
    }
}

impl<T> Drop for AtomicVec<T> {
    fn drop(&mut self) {
        while self.accessors.load(Ordering::Acquire)>0 || self.mutable_operation_ongoing() {
            log::warn!("Spinning on AtomicVec drop! This shouldn't happen! (Lifetimes should prevent references to the contained value existing when it's being dropped.)");
            std::hint::spin_loop()
        }
        unsafe {
            let layout = *self.layout.get_mut();

            let nonnull = std::ptr::NonNull::new_unchecked((*self.ptr.get_mut()) as *mut u8);
            Global::default().deallocate(nonnull, *layout);
            drop(Box::from_raw(layout));
        }
    }
}

pub struct AtomicVecGuard<'a, T> {
    ptr: &'a AtomicVec<T>,
    item: usize
}

impl<'a, T> AtomicVecGuard<'a, T> {
    pub(self) fn new(vec: &'a AtomicVec<T>, item: usize) -> Self {
        vec.accessors.fetch_add(1, Ordering::AcqRel);
        Self { ptr: vec, item }
    }
}

impl<'a, T> Deref for AtomicVecGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(debug_assertions)]
        if self.item>=self.ptr.len.load(Ordering::SeqCst) {
            panic!("Tried accessing Index outside of AtomicVec!");
        }
        unsafe { &mut self.ptr.get_elem_unsafe(self.item, Ordering::SeqCst).0 }
    }
}

impl<'a, T> Drop for AtomicVecGuard<'a, T> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        let cond = self.item<self.ptr.len.load(Ordering::Acquire);
        #[cfg(not(debug_assertions))]
        let cond = true;
        if cond {
            unsafe { self.ptr.get_elem_unsafe(self.item, Ordering::SeqCst).1.fetch_sub(1, Ordering::SeqCst); }
        } else {
            panic!("Tried accessing Index outside of AtomicVec!");
        }
        self.ptr.accessors.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct AtomicVecGuardMut<'a, T> {
    ptr: &'a AtomicVec<T>,
    item: usize
}

impl<'a, T> AtomicVecGuardMut<'a, T> {
    pub(self) fn new(vec: &'a AtomicVec<T>, item: usize) -> Self {
        vec.accessors.fetch_add(1, Ordering::Acquire);
        Self { ptr: vec, item }
    }
}

impl<'a, T> Deref for AtomicVecGuardMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(debug_assertions)]
        if self.item>=self.ptr.len.load(Ordering::SeqCst) {
            panic!("Tried accessing Index outside of AtomicVec!");
        }
        unsafe { &mut self.ptr.get_elem_unsafe(self.item, Ordering::SeqCst).0 }
    }
}

impl<'a, T> DerefMut for AtomicVecGuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(debug_assertions)]
        if self.item>=self.ptr.len.load(Ordering::Acquire) {
            panic!("Tried accessing Index outside of AtomicVec!");
        }
        unsafe { &mut self.ptr.get_elem_unsafe(self.item, Ordering::SeqCst).0 }
    }
}

impl<'a, T> Drop for AtomicVecGuardMut<'a, T> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        let cond = self.item<self.ptr.len.load(Ordering::Acquire);
        #[cfg(not(debug_assertions))]
        let cond = true;
        if cond {
            unsafe { self.ptr.get_elem_unsafe(self.item, Ordering::SeqCst).1.store(0, Ordering::SeqCst) }
        } else {
            panic!("Tried accessing Index outside of AtomicVec!");
        }
        self.ptr.accessors.fetch_sub(1, Ordering::AcqRel);
    }
}