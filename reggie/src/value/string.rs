use std::{fmt, hash::Hash, ptr::NonNull, rc::Rc};

use luar_lex::Ident;

#[repr(packed)]
pub struct LuaString {
    // since the maximum alingment of LuaValue associated
    // values is 8, LuaValue cannot be less than 16 bytes (for now)
    // the tag in enums usually is a u32, which gives us 32 bits
    // to store the length at no cost. This also means, that we can
    // potentially "borrow" std::ptr::Unique to make
    // sizeof(Option<LuaString>) == sizeof(LuaString>)
    len: u32,
    // This shoud've been NonNull<str>, but that is a fat pointer
    str: NonNull<()>,
}

impl Default for LuaString {
    fn default() -> Self {
        Self {
            len: 0,
            str: NonNull::dangling(),
        }
    }
}

impl From<&str> for LuaString {
    fn from(str: &str) -> Self {
        let allocation: Rc<str> = Rc::from(str);
        let ptr = Rc::into_raw(allocation);

        Self {
            len: str
                .len()
                .try_into()
                .expect("size of string should not to exceed u32"),
            // This could as well easily be NonNull::new_unchecked
            str: NonNull::new(ptr as *mut ()).expect("pointer from Rc should never be null"),
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
        if self.len != 0 {
            // SAFETY: This is safe, because we know that the pointer is not dangling
            //         and that the allocation is still alive
            unsafe {
                drop(Rc::from_raw(self.str.as_ptr()));
            }
        }
    }
}

impl AsRef<str> for LuaString {
    fn as_ref(&self) -> &str {
        if self.len == 0 {
            return "";
        }
        let ptr = self.str.as_ptr() as *const u8;
        // SAFETY: This is safe, because we know that the pointer is not dangling
        //         and that the allocation is still alive
        //         and that self.str points to a valid UTF-8 string, since
        //         LuaString::from(&str) is only implemented for valid &str
        //         self.len is not overflowing, since there is a check in
        //         impl From<&str> for LuaString
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, self.len as usize);
            std::str::from_utf8_unchecked(slice)
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
        Self::from(self.as_ref())
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
