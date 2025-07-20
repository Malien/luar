
// #[repr(transparent)]
// struct CompactString(NonZeroU64);

use std::{alloc::{alloc, handle_alloc_error, Layout}, fmt, hash::Hash, marker::PhantomData, ops::Deref, ptr::NonNull, slice};

pub(crate) struct StringHeader {
    len: u32,
    refcount: u32,
    _unused: PhantomData<*const()>,
}

/// A simple wrapper around a raw pointer. Does not manages it's lifetime.
/// Most operations are unsafe, as the type does not gurantee it's liveliness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct SharedStringPtr(pub NonNull<StringHeader>);

impl SharedStringPtr {
    pub(crate) fn alloc_and_copy(str: &str) -> Self {
        // SAFETY: I mean. There is a lot to unpack here. I don't really want to.
        unsafe {
            let len = str.len().try_into().expect("Strings cannot exceed the length of u32::MAX bytes");

            let block = alloc(Self::layout(len));
            #[cfg(feature = "trace-allocation")]
            eprintln!("[shared string] Alloc {:?} at {:p}", str, block);
            let Some(block) = NonNull::new(block) else {
                handle_alloc_error(Self::layout(len));
            };
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
        unsafe {
            let data_ptr = self.0.cast::<u8>().as_ptr().byte_add(size_of::<StringHeader>()) as *const _;
            let slice = slice::from_raw_parts(data_ptr, self.0.as_ref().len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }

    /// SAFETY: Make sure that the pointer is valid
    pub(crate) unsafe fn release(mut self) {
        let header = unsafe { self.0.as_mut() };
        #[cfg(feature = "trace-allocation")]
        eprintln!("[shared string] Release at {:p}. Refcount: {}", self.0.as_ptr(), header.refcount);
        if header.refcount == 0 {
            #[cfg(feature = "trace-allocation")]
            eprintln!("[shared string] Dealloc at {:p}", self.0.as_ptr());
            unsafe {
                // No need to call Drop, since StrBlock is trivially dropable.
                std::alloc::dealloc(
                    self.0.as_ptr() as *mut u8,
                    Self::layout(header.len)
                );
            }
        } else {
            header.refcount -= 1;
        }
    }

    /// SAFETY: Make sure that the pointer is valid
    pub(crate) unsafe fn retain(mut self) {
        #[cfg(feature = "trace-allocation")]
        eprintln!("[shared string] Retain at {:p}. Refcount: {}", self.0.as_ptr(), self.0.as_ref().refcount);
        let header = unsafe { self.0.as_mut() };
        let (refcount, did_overflow) = header.refcount.overflowing_add(1);
        assert!(!did_overflow);
        header.refcount = refcount;
    }

    /// SAFETY: Make sure that the pointer is valid
    pub(crate) unsafe fn refcount(self) -> u32 {
        unsafe { self.0.as_ref().refcount }
    }

    pub(crate) unsafe fn len(self) -> u32 {
        unsafe { self.0.as_ref().len }
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

    pub fn len(&self) -> u32 {
        unsafe { self.0.len() }
    }

    // SAFETY: Make sure the pointer is valid
    pub(crate) unsafe fn retain(ptr: SharedStringPtr) -> Self {
        unsafe { ptr.retain() };
        Self(ptr)
    }

    // SAFETY: Make sure the pointer is valid
    pub(crate) unsafe fn unretained(ptr: SharedStringPtr) -> Self {
        Self(ptr)
    }

    pub(crate) fn refcount(&self) -> u32 {
        unsafe { self.0.refcount() + 1 }
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

#[repr(transparent)]
struct UnsafeGlobalAllocation(StringHeader);
unsafe impl Sync for UnsafeGlobalAllocation {}

static EMPTY_STRING_ALLOCATION: UnsafeGlobalAllocation = UnsafeGlobalAllocation(StringHeader {
    len: 0,
    // There is a race condition here. Even though LuaString is not Sync, one could create two
    // empty strings in two threads at the same time, and both would try to change the count.
    //
    // This would be fixed by interning and/or small string optimization.
    // TODO: SSO
    refcount: 1,
    _unused: PhantomData,
});

impl Default for CompactString {
    fn default() -> Self {
        let ptr = &EMPTY_STRING_ALLOCATION.0 as *const StringHeader as *mut _;
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        let ptr = SharedStringPtr(ptr);
        Self(ptr)
    }
}

impl Deref for CompactString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Drop for CompactString {
    fn drop(&mut self) {
        // SAFETY: It is safe to release the string here, because we guarantee that
        // it is alive. And empty string will never be released.
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

impl PartialEq for CompactString {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for CompactString {}

impl PartialEq<&str> for CompactString {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl PartialOrd for CompactString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl Ord for CompactString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl Hash for CompactString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl From<&str> for CompactString {
    fn from(str: &str) -> Self {
        Self::new(str)
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for CompactString {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let str = String::arbitrary(g);
        Self::from(str.as_str())
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.as_ref()
                .to_string()
                .shrink()
                .map(|str| Self::from(str.as_str())),
        )
    }
}

macro_rules! compact_format {
    ($($t:expr),*) => {
        {
            let str = format!($($t),*);
            $crate::value::LuaString::from(str.as_str())
        }
    }
}

pub(crate) use compact_format;
