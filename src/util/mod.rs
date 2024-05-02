pub mod consts;
pub mod hashable_value;
pub mod atomic_vec;

pub use hashable_value::*;

pub const fn extended_slice<const A: usize, const B: usize, T: Copy>(base: &'static [T; A], new: [T; B]) -> [T; A + B]
where [T; A + B]: Sized {
    use std::mem::MaybeUninit;

    unsafe {
        let mut arr: [T; A + B] = MaybeUninit::uninit().assume_init();

        std::ptr::copy(base.as_ptr(), arr.as_mut_ptr(), A);
        std::ptr::copy(new.as_ptr(), arr.as_mut_ptr().add(A), B);

        arr
    }
}