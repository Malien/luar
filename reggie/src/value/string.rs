use std::{alloc::Layout, fmt, hash::Hash, marker::PhantomData, mem::size_of, slice};

use luar_lex::Ident;

const INLINE_BUFFER_SIZE: usize = std::mem::size_of::<*const ()>();

#[repr(packed)]
pub struct LuaString {
    // since the maximum alingment of LuaValue associated
    // values is 8, LuaValue cannot be less than 16 bytes (for now)
    // the tag in enums usually is a u32, which gives us 32 bits
    // to store the length at no cost. This also means, that we can
    // potentially "borrow" std::ptr::Unique to make
    // sizeof(Option<LuaString>) == sizeof(LuaString>)
    len: u32,
    /// Either an inline string for strings that are shorter than
    /// INLINE_BUFFER_SIZE bytes, or a pointer to an allocation
    /// for longer strings. Points straight to the StrBlock.
    ptr_or_inline_data: SSOStorage,
    _unused: PhantomData<*const ()>,
}

/// Small String Optimized storage.
#[derive(Copy, Clone)]
union SSOStorage {
    inline_data: [u8; INLINE_BUFFER_SIZE],
    heap_allocation: *mut (),
}

#[repr(C)]
struct StrBlock {
    /// Number of outstanding references to this allocation, minus one.
    /// If refcount is zero, you are responsible for deallocating them.
    /// Sharing StrBlock between threads is not safe, since refcount
    /// is not atomic.
    refcount: usize,
    _unused: PhantomData<*const ()>,
    data: str,
}

impl Default for LuaString {
    fn default() -> Self {
        Self {
            len: 0,
            _unused: PhantomData,
            ptr_or_inline_data: SSOStorage {
                inline_data: [0; INLINE_BUFFER_SIZE],
            },
        }
    }
}

impl From<&str> for LuaString {
    fn from(str: &str) -> Self {
        // SAFETY: it is safe to store len of a string in u32, since
        //         if the value overflows u32, we will panic.
        let len: u32 = str
            .len()
            .try_into()
            .expect("size of string should not to exceed u32");

        if len == 0 {
            return Self::default();
        }

        if len <= INLINE_BUFFER_SIZE as u32 {
            let mut inline_data = [0; INLINE_BUFFER_SIZE];
            inline_data[..str.len()].copy_from_slice(str.as_bytes());
            return Self {
                len,
                _unused: PhantomData,
                ptr_or_inline_data: SSOStorage { inline_data },
            };
        }

        // SAFETY: Allocation size is enough to store the StrBlock with the data
        //         ```
        //         #[repr(C)]
        //         struct StrBlock {
        //             refcount: usize,
        //             _unused: PhantomData<*const ()>,
        //             data: str,
        //         }
        //         ```
        //         sizeof(refcount) + sizeof(PhantomData<*const ()>) + sizeof(str) == 8 + 0 + len
        //         This is the layout of StrBlock, since it is repr(C).
        //
        //         Data at allocation is uninitialized, but no matter, we write
        //         to it immediately afterwards.
        let block_ptr = unsafe {
            let allocation_size = len as usize + size_of::<usize>();
            // SAFETY:
            //       * `align` is not zero
            //       * `align` is a power of two
            //       * `size`, when rounded up to the nearest multiple of `align`,
            //          cannot not overflow isize (i.e., the rounded value must be
            //          less than or equal to `isize::MAX`).
            let allocation =
                std::alloc::alloc(Layout::from_size_align_unchecked(allocation_size, 1));
            let slice = slice::from_raw_parts_mut(allocation, len as usize);
            let block_ptr = slice as *mut [u8] as *mut StrBlock;
            let str_block = &mut *block_ptr;
            str_block.refcount = 0;
            str_block
                .data
                .as_bytes_mut()
                .copy_from_slice(str.as_bytes());
            block_ptr
        };

        Self {
            len,
            _unused: PhantomData,
            ptr_or_inline_data: SSOStorage {
                heap_allocation: block_ptr as *mut (),
            },
        }
    }
}

impl From<String> for LuaString {
    fn from(str: String) -> Self {
        Self::from(str.as_str())
    }
}

impl From<&String> for LuaString {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<Ident> for LuaString {
    fn from(value: Ident) -> Self {
        Self::from(value.as_ref())
    }
}

impl Drop for LuaString {
    fn drop(&mut self) {
        if self.len > INLINE_BUFFER_SIZE as u32 {
            // SAFETY: Everything that is longer than INLINE_BUFFER_SIZE bytes in
            //         self.ptr_or_inline_data is a valid pointer to a valid allocation,
            //         allocated by Rc<str>.
            //         Ptr is saved into [u8; INLINE_BUFFER_SIZE] via usize::to_ne_bytes,
            //         and is brought back via usize::from_ne_bytes.
            //         [8; INLINE_BUFFER_SIZE] is guaranteed to have enough space to
            //         store a pointer. str pointer is brought back exactly as it was
            //         saved, including it's length in the fat pointer.
            //         Call to from_utf8_unchecked has no safety implications, since
            //         str is not accessed in any way.
            //         Decresing refcount is safe, since LuaString cannot be shared
            //         between threads.
            unsafe {
                let ptr = self.ptr_or_inline_data.heap_allocation;
                // Until we have std::ptr::from_raw_parts this is a workaround for
                // creating fat pointers to ?Sized structs
                let slice = std::slice::from_raw_parts_mut(ptr, self.len as usize);
                let block_ptr = slice as *mut [()] as *mut StrBlock;
                let block = &mut *block_ptr;

                if block.refcount == 0 {
                    // No need to call Drop, since StrBlock is trivially dropable.
                    let layout = Layout::from_size_align_unchecked(
                        size_of::<usize>() + self.len as usize,
                        1,
                    );
                    std::alloc::dealloc(block_ptr as *mut u8, layout)
                } else {
                    block.refcount -= 1;
                }
            }
        }
    }
}

impl AsRef<str> for LuaString {
    fn as_ref(&self) -> &str {
        // SAFETY: If self.len < INLINE_BUFFER_SIZE, then self.ptr_or_inline_data
        //         contains a inline string. inline_data is a valid UTF-8 string
        //         since the only way to construct LuaString is via a valid &str
        //
        //         Otherwise self.ptr_or_inline_data pointer to a valid allocation
        //         of StrBlock.
        //         StrBlock is valid, since we refcount outstanding references.
        unsafe {
            if self.len <= INLINE_BUFFER_SIZE as u32 {
                let byte_slice = &self.ptr_or_inline_data.inline_data[..self.len as usize];
                std::str::from_utf8_unchecked(byte_slice)
            } else {
                let block_ptr = self.ptr_or_inline_data.heap_allocation;
                // Until we have std::ptr::from_raw_parts this is a workaround for
                // creating fat pointers to ?Sized structs
                let slice = std::slice::from_raw_parts(block_ptr, self.len as usize);
                let block_ptr = slice as *const [()] as *const StrBlock;
                let block = &*block_ptr;
                &block.data
            }
        }
    }
}

impl std::ops::Deref for LuaString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl fmt::Display for LuaString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_ref(), f)
    }
}
impl fmt::Debug for LuaString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_ref(), f)
    }
}
impl PartialEq for LuaString {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl Eq for LuaString {}
impl PartialOrd for LuaString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}
impl Ord for LuaString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}
impl Hash for LuaString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}
impl Clone for LuaString {
    fn clone(&self) -> Self {
        if self.len > INLINE_BUFFER_SIZE as u32 {
            // SAFETY: Everything that is longer than INLINE_BUFFER_SIZE bytes is
            //         stored inline. Otherwise self.ptr_or_inline_data contains a
            //         pointer to a valid allocation of StrBlock, allocated by std::alloc.
            //
            //         We do not memcopy the allocation, but instead share it.
            //         StrBlock is refcounted, so we increase the refcount.
            //         It is safe to increase refcount, since LuaString cannot be shared
            //         between threads.
            unsafe {
                let ptr = self.ptr_or_inline_data.heap_allocation;
                let slice = std::slice::from_raw_parts_mut(ptr, self.len as usize);
                let block_ptr = slice as *mut [()] as *mut StrBlock;
                let block = &mut *block_ptr;
                block.refcount += 1;
            }
        }

        Self {
            len: self.len,
            _unused: PhantomData,
            ptr_or_inline_data: self.ptr_or_inline_data,
        }
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for LuaString {
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

#[macro_export]
macro_rules! lua_format {
    ($($t:expr),*) => {
        {
            let str = format!($($t),*);
            $crate::LuaString::from(str.as_str())
        }
    }
}
