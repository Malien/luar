
// #[repr(transparent)]
// struct CompactString(NonZeroU64);

use std::{alloc::{alloc, Layout}, fmt, marker::PhantomData, ptr::NonNull, slice};

pub(crate) struct StringHeader {
    len: u32,
    refcount: u32,
    _unused: PhantomData<*const()>,
}

/// A simple wrapper around a raw pointer. Does not manages it's lifetime.
/// Most operations are unsafe, as the type does not gurantee it's liveliness.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub(crate) struct SharedStringPtr(pub NonNull<StringHeader>);

impl SharedStringPtr {
    pub(crate) fn alloc_and_copy(str: &str) -> Self {
        // SAFETY: I mean. There is a lot to unpack here. I don't really want to.
        unsafe {
            let len = str.len().try_into().expect("Strings cannot exceed the length of u32::MAX bytes");

            let block = alloc(Self::layout(len));
            let block = NonNull::new(block).expect("Couldn't allocate memory for a lua string");
            let mut header_ptr = block.cast::<StringHeader>();
            header_ptr.as_mut().refcount = 0;
            header_ptr.as_mut().len = len;
            let data_ptr = block.as_ptr().byte_add(size_of::<StringHeader>());
            let target_slice = slice::from_raw_parts_mut(data_ptr, str.len());
            target_slice.copy_from_slice(str.as_bytes());

            Self(header_ptr)
        }
    }

    /// SAFETY: Make sure that the lifetime of the string block is greater than the desired lifetime
    pub(crate) unsafe fn str_ref<'a>(self) -> &'a str {
        let data_ptr = self.0.cast::<u8>().as_ptr().byte_add(size_of::<StringHeader>()) as *const _;
        let slice = slice::from_raw_parts(data_ptr, self.0.as_ref().len as usize);
        std::str::from_utf8_unchecked(slice)
    }

    /// SAFETY: Make sure that the pointer is valid
    pub(crate) unsafe fn release(mut self) {
        let header = unsafe { self.0.as_mut() };
        if header.refcount == 0 {
            unsafe {
                // No need to call Drop, since StrBlock is trivially dropable.
                std::alloc::dealloc(
                    self.0.as_ptr() as *mut u8,
                    Self::layout(header.len)
                )
            }
        } else {
            header.refcount -= 1;
        }
    }

    /// SAFETY: Make sure that the pointer is valid
    pub(crate) unsafe fn retain(mut self) {
        unsafe { 
            let (_, did_overflow) = self.0.as_mut().refcount.overflowing_add(1);
            assert!(!did_overflow);
        }
    }

    fn layout(body_len: u32) -> Layout {
        let (layout, offset) = Layout::new::<StringHeader>()
            .extend(Layout::array::<u8>(body_len as usize).unwrap())
            .unwrap();
        assert_eq!(offset, std::mem::size_of::<StringHeader>());
        layout
    }
}

#[repr(transparent)]
pub struct CompactString(SharedStringPtr);

impl CompactString {
    pub fn new(str: impl AsRef<str>) -> Self {
        Self(SharedStringPtr::alloc_and_copy(str.as_ref()))
    }

    // SAFETY: Make sure the pointer is valid
    pub(crate) unsafe fn retain(ptr: SharedStringPtr) -> Self {
        ptr.retain();
        Self(ptr)
    }

    // SAFETY: Make sure the pointer is valid
    pub(crate) unsafe fn unretained(ptr: SharedStringPtr) -> Self {
        Self(ptr)
    }

    fn refcount(&self) -> u32 {
        unsafe { self.0.0.as_ref().refcount }
    }

    pub(crate) fn leak(self) -> SharedStringPtr {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }

}

impl AsRef<str> for CompactString {
    fn as_ref(&self) -> &str {
        // SAFETY: string ref is valid until self is valid
        unsafe { self.0.str_ref() }
    }
}

impl Drop for CompactString {
    fn drop(&mut self) {
        unsafe { self.0.release() };
    }
}

impl Clone for CompactString {
    fn clone(&self) -> Self {
        unsafe { self.0.retain() };
        Self(self.0)
    }
}

impl fmt::Debug for CompactString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl fmt::Display for CompactString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

