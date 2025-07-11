use std::{rc::Rc, ptr::NonNull, cell::RefCell, marker::PhantomData, alloc::{alloc, Layout}, slice, mem::size_of};
 
use nonzero_ext::NonZero;

use crate::{TableRef, TableValue, ids::BlockID, NativeFunction, NativeFunctionKind};

/// Here's the anatomy of the packed value:
/// ```text
///   ,- sign bit. 
///   |  When float value is a signaling nan, bit marks strings (1), or other lua values (0)
///   |
///   | ,- exponent. Should be all 1s for signaling nans.
///   | |
///   | |           ,- first matissa bit. Should be set to 0 to mark nan as signaling.
///   | |           |
///   | |           | ,- three bits reserved as a type tag for values other than floats. 
///   | |           | |  For small strings, it is the length of the string minus 1. 
///   | |           | |  If the length is 0b000, it is a heap allocated string.
///   | |           | | 
///   | |           | |   ,- 48 bits in which to pack the payload. For nil it is meaningless.
///   | |           | |   |  For integers, it is the bottom 32 bits that store it.
///   | |           | |   |  For function, it is the bottom 32 bits that store block id.
///   | |           | |   |  For tables and native functions it is the pointer to the heap
///   | |           | |   |  allocation. Assuming the pointer can be packed into 48 bits on the
///   | |           | |   |  respective platform.
///   | |           | |   |  Pointers can't be null. It would make float into an inf.
///   v v           v v   v
/// 0b0_00000000000_0_000_000000000000000000000000000000000000000000000000
/// ```
pub struct CompactLuaValue(u64);

// We can be able to pack pointers into 48 bits on:
//   - aarch64 without pointer signings
//   - x86-64 macos malloc'd heap pointers
//   - x86-64 linux glibc malloc'd heap pointers
const fn is_compatible_with_48bit_pointers() -> bool {
    cfg!(all(target_os = "macos", target_arch = "x86_64"))
}
const _: () = assert!(is_compatible_with_48bit_pointers(), "Compact Lua values (compact_value feature) are only supported on 64-bit x86 macos. For now.");


#[repr(u8)]
/// Tags values that are not floats nor small strings. Should fit in three bits.
/// Cannot be all zeros, since that would be a IEE754 inf, not signaling NaN
enum Tag {
    Table          = 0b000,
    Nil            = 0b001,
    Int            = 0b010,
    Function       = 0b100,
    NativeFunction = 0b101,
}

const SIGNALING_NAN_BITPATTERN: u64 = 0b0_11111111111_0_000_000000000000000000000000000000000000000000000000;
const           INF_BITPATTERN: u64 = 0b0_11111111111_0_000_000000000000000000000000000000000000000000000000;
const           INT_BITPATTERN: u64 = 0b0_11111111111_0_010_000000000000000000000000000000000000000000000000;
const       NEG_INF_BITPATTERN: u64 = 0b1_11111111111_0_000_000000000000000000000000000000000000000000000000;
const         TABLE_BITPATTERN: u64 = 0b0_11111111111_0_000_000000000000000000000000000000000000000000000000;
const           NIL_BITPATTERN: u64 = 0b0_11111111111_0_001_000000000000000000000000000000000000000000000000;
const        STRING_BITPATTERN: u64 = 0b1_11111111111_0_000_000000000000000000000000000000000000000000000000;
const      LUA_FUNC_BITPATTERN: u64 = 0b0_11111111111_0_100_000000000000000000000000000000000000000000000000;
const   NATIVE_FUNC_BITPATTERN: u64 = 0b0_11111111111_0_101_000000000000000000000000000000000000000000000000;
const      ANY_FUNC_BITPATTERN: u64 = 0b0_11111111111_0_100_000000000000000000000000000000000000000000000000;
const         ANY_FUNC_BITMASK: u64 = 0b1_11111111111_0_110_000000000000000000000000000000000000000000000000;

/// Pick which bits we are interested in
macro_rules! bitmask {
    () => { 0 };
    (sign) => {
        0b1_00000000000_0_000_000000000000000000000000000000000000000000000000
    };
    (exponent) => {
        0b0_11111111111_0_000_000000000000000000000000000000000000000000000000
    };
    (snan) => {
        0b0_00000000000_1_000_000000000000000000000000000000000000000000000000
    };
    (typetag) => {
        0b0_00000000000_0_111_000000000000000000000000000000000000000000000000
    };
    (ptrpayload) => {
        0b0_00000000000_0_000_111111111111111111111111111111111111111111111111
    };
    ($head:ident, $($tail:ident),*) => {
        bitmask!($head) | bitmask!($($tail),*)
    }
}

/// Mask out just the right bits of the IEE754 64-bit float. For the anatomy of the value refer
/// to the [CompactLuaValue]
macro_rules! pick {
    ($value:expr, $($rest:ident),*) => {
        ($value & bitmask!($($rest),*))
    }
}

impl CompactLuaValue {
    pub fn is_float(&self) -> bool {
        pick!(self.0, exponent, snan) != SIGNALING_NAN_BITPATTERN || 
            // aka. everything, except for the sign bit
            pick!(self.0, exponent, snan, typetag, ptrpayload) == INF_BITPATTERN
    }

    pub fn is_nil(&self) -> bool {
        self.0 == NIL_BITPATTERN
    }

    pub fn is_int(&self) -> bool {
        pick!(self.0, sign, exponent, snan, typetag) == INT_BITPATTERN
    }
    
    pub fn is_string(&self) -> bool {
        pick!(self.0, sign, exponent, snan) == STRING_BITPATTERN && self.0 != NEG_INF_BITPATTERN
    }

    pub fn is_table(&self) -> bool {
        pick!(self.0, sign, exponent, snan, typetag) == TABLE_BITPATTERN && self.0 != INF_BITPATTERN
    }

    pub fn is_lua_function(&self) -> bool {
        pick!(self.0, sign, exponent, snan, typetag) == LUA_FUNC_BITPATTERN
    }

    pub fn is_native_function(&self) -> bool {
        pick!(self.0, sign, exponent, snan, typetag) == NATIVE_FUNC_BITPATTERN
    }

    pub fn is_function(&self) -> bool {
        (self.0 & ANY_FUNC_BITMASK) == ANY_FUNC_BITPATTERN
    }

    pub const NIL: Self = Self(NIL_BITPATTERN);

    pub fn int(x: i32) -> Self {
        let low_bits = x as u32 as u64;
        Self(INT_BITPATTERN | low_bits)
    }

    pub fn as_int(&self) -> Option<i32> {
        if self.is_int() {
            Some(self.0 as i32)
        } else {
            None
        }
    }

    /// SAFETY: Make sure the pointer is valid. Later accesses to it depend on that.
    unsafe fn encode_pointer<T>(ptr: NonNull<T>) -> u64 {
        let ptr = ptr.as_ptr() as u64;
        assert_eq!(ptr & 0b1111111111110000000000000000000000000000000000000000000000001111, 0, "When encoding heap allocations into a CompactLuaValue we expect pointers to have top 12 and bottom 4 bits zeroed. Looks like our assumption (at least for this platform) was wrong.");
        ptr >> 4
    }

    unsafe fn decode_pointer(&self) -> NonNull<()> {
        let ptr_bits = pick!(self.0, ptrpayload);
        let ptr = ptr_bits << 4;
        NonNull::new(ptr as * mut ()).expect("This should never happen. We check for non-null pointers when encoding. Somehow this one splipped through. Also null-ptr-encoding would've resulted in a float Inf, not table encoded in nan.")
    }

    pub fn table(table: TableRef) -> Self {
        let ptr = Rc::into_raw(table.0).cast_mut();
        let ptr = NonNull::new(ptr).expect("Rc pointers should never be null");
        let ptr_bits = unsafe { Self::encode_pointer(ptr) };

        Self(TABLE_BITPATTERN | ptr_bits)
    }

    fn as_table_ptr(&self) -> Option<NonNull<RefCell<TableValue>>> {
        if self.is_table() {
            Some(unsafe { self.decode_pointer().cast() })
        } else {
            None
        }
    }

    pub fn as_table(&self) -> Option<TableRef> {
        // SAFETY: the ptr is a valid pointer since the encoded one was valid and the
        //         decoding masked out float stuff, and brought it back to being a valid pointer.
        //         Every acess to the Rc::from_raw is preceeded with Rc::increment_strong_count.
        //         As a result, we give out properly refcounted table refs.
        self.as_table_ptr().map(|ptr| unsafe {
            let ptr = ptr.as_ptr();
            Rc::increment_strong_count(ptr);
            TableRef(Rc::from_raw(ptr))
        })
    }

    pub fn string(str: impl AsRef<str>) -> Self {
        // SAFETY: I mean. There is a lot to unpack here. I don't really want to.
        unsafe {
            let str = str.as_ref();
            let len = str.len().try_into().expect("Strings cannot exceed the length of u32::MAX bytes");
            let allocation_size = str.len() + size_of::<StringHeader>();
            let block = alloc(Layout::from_size_align_unchecked(allocation_size, 1));
            let block = NonNull::new(block).expect("Couldn't allocate memory for a lua string");
            let header_ref = block.cast::<StringHeader>().as_mut();
            header_ref.refcount = 0;
            header_ref.len = len;
            let data_ptr = block.as_ptr().byte_add(size_of::<StringHeader>());
            let target_slice = slice::from_raw_parts_mut(data_ptr, str.len());
            target_slice.copy_from_slice(str.as_bytes());

            Self(STRING_BITPATTERN | Self::encode_pointer(block))
        }
    }

    fn as_string_ptr(&self) -> Option<NonNull<StringHeader>> {
        if self.is_string() {
            unsafe {
                Some(self.decode_pointer().cast())
            }
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<CompactString> {
        // SAFETY: Pointer is valid
        self.as_string_ptr().map(|ptr| unsafe { CompactString::retain(ptr) })
    }

    pub fn as_str(&self) -> Option<&str> {
        // SAFETY: Pointer is valid. resulting ref lifetime is shorter than self that retains the
        //         string storage
        self.as_string_ptr().map(|ptr| unsafe { shared_string_ref(ptr) })
    }

    pub fn lua_function(code_block_id: BlockID) -> Self {
        let payload = code_block_id.0 as u64;
        Self(LUA_FUNC_BITPATTERN | payload)
    }

    pub fn as_lua_function(&self) -> Option<BlockID> {
        if self.is_lua_function() {
            // Strip just low 32 bits
            Some(BlockID(self.0 as u32))
        } else {
            None
        }
    }

    pub fn native_function(func: NativeFunction) -> Self {
        let ptr = Rc::into_raw(func.0).cast_mut();
        let ptr = NonNull::new(ptr).expect("Rc pointers should never be null");
        let ptr_bits = unsafe { Self::encode_pointer(ptr) };

        Self(NATIVE_FUNC_BITPATTERN | ptr_bits)
    }

    fn as_native_function_ptr(&self) -> Option<NonNull<NativeFunctionKind>> {
        if self.is_native_function() {
            Some(unsafe { self.decode_pointer().cast() })
        } else {
            None
        }
    }

    pub fn as_native_function(&self) -> Option<NativeFunction> {
        // SAFETY: the ptr is a valid pointer since the encoded one was valid and the
        //         decoding masked out float stuff, and brought it back to being a valid pointer.
        //         Every acess to the Rc::from_raw is preceeded with Rc::increment_strong_count.
        //         As a result, we give out properly refcounted table refs.
        self.as_native_function_ptr().map(|ptr| unsafe {
            let ptr = ptr.as_ptr();
            Rc::increment_strong_count(ptr);
            NativeFunction(Rc::from_raw(ptr))
        })
    }

    pub fn float(value: f64) -> Self {
        let bits = value.to_bits();
        let is_signaling_nan = pick!(bits, exponent, snan) == SIGNALING_NAN_BITPATTERN &&
            // aka. everything, except for the sign bit
            pick!(bits, exponent, snan, typetag, ptrpayload) != INF_BITPATTERN;
        if is_signaling_nan {
            if cfg!(debug_assertions) {
                panic!("Tried to construct a LuaValue float from a signaling nan. This is likely a bug. In release build, float will be coerced to a non-signaling nan")
            }
            Self(f64::NAN.to_bits())
        } else {
            Self(bits)
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if self.is_float() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }
}

// #[repr(transparent)]
// struct CompactString(NonZeroU64);

#[repr(transparent)]
struct CompactString(NonNull<StringHeader>);

/// SAFETY: Make sure that the lifetime of the string block is greater than the desired lifetime
unsafe fn shared_string_ref<'a>(ptr: NonNull<StringHeader>) -> &'a str {
    let data_ptr = ptr.cast::<u8>().as_ptr().byte_add(size_of::<StringHeader>()) as *const _;
    let slice = slice::from_raw_parts(data_ptr, ptr.as_ref().len as usize);
    std::str::from_utf8_unchecked(slice)
}

/// SAFETY: Make sure that the pointer is valid
unsafe fn release_shared_string(mut ptr: NonNull<StringHeader>) {
    let header = unsafe { ptr.as_mut() };
    if header.refcount == 0 {
        unsafe {
            // No need to call Drop, since StrBlock is trivially dropable.
            std::alloc::dealloc(
                ptr.as_ptr() as *mut u8,
                shared_string_layout(header.len)
            )
        }
    } else {
        header.refcount -= 1;
    }
}

fn shared_string_layout(body_len: u32) -> Layout {
    let (layout, offset) = Layout::new::<StringHeader>()
        .extend(Layout::array::<u8>(body_len as usize).unwrap())
        .unwrap();
    assert_eq!(offset, std::mem::size_of::<StringHeader>());
    layout
}

impl CompactString {
    pub fn new(str: impl AsRef<str>) -> Self {
        // SAFETY: I mean. There is a lot to unpack here. I don't really want to.
        unsafe {
            let str = str.as_ref();
            let len = str.len().try_into().expect("Strings cannot exceed the length of u32::MAX bytes");

            let block = alloc(shared_string_layout(len));
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

    // SAFETY: Make sure the pointer is valid
    unsafe fn retain(mut ptr: NonNull<StringHeader>) -> Self {
        ptr.as_mut().refcount += 1;
        Self(ptr)
    }

    // SAFETY: Make sure the pointer is valid
    unsafe fn unretained(ptr: NonNull<StringHeader>) -> Self {
        Self(ptr)
    }

    fn refcount(&self) -> u32 {
        unsafe { self.0.as_ref().refcount }
    }

}

impl AsRef<str> for CompactString {
    fn as_ref(&self) -> &str {
        // SAFETY: string ref is valid until self is valid
        unsafe { shared_string_ref(self.0) }
    }
}

impl Drop for CompactString {
    fn drop(&mut self) {
        unsafe { release_shared_string(self.0) };
    }
}

struct StringHeader {
    len: u32,
    refcount: u32,
    _unused: PhantomData<*const()>,
}

impl Default for CompactLuaValue {
    fn default() -> Self { Self::NIL }
}

impl Drop for CompactLuaValue {
    fn drop(&mut self) {
        if let Some(table_ptr) = self.as_table_ptr() {
            // SAFTEY: This is the only place where we decrement refcount.
            //         Every other access to the table ref should be guarded with
            //         [CompactLuaValue::as_table]
            unsafe { Rc::decrement_strong_count(table_ptr.as_ptr()) };
        }
        if let Some(native_function_ptr) = self.as_native_function_ptr() {
            // SAFTEY: This is the only place where we decrement refcount.
            //         Every other access to the table ref should be guarded with 
            //         [CompactLuaValue::as_native_function]
            unsafe { Rc::decrement_strong_count(native_function_ptr.as_ptr()) };
        }
        if let Some(str_ptr) = self.as_string_ptr() {
            unsafe { release_shared_string(str_ptr) };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{TableRef, NativeFunction};

    use super::CompactLuaValue;
    #[cfg(feature = "quickcheck")]
    use quickcheck::quickcheck;

    #[test]
    fn nil_is_encoded_correctly() {
        let value = CompactLuaValue::NIL;
        assert!(value.is_nil());
    }

    #[test]
    fn default_is_nil() {
        assert!(CompactLuaValue::default().is_nil());
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn ints_are_properly_stored(x: i32) {
        let value = CompactLuaValue::int(x);
        assert!(value.is_int());
        assert_eq!(value.as_int(), Some(x));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn tables_are_properly_stored(table_ref: TableRef) {
        let value = CompactLuaValue::table(table_ref.clone());
        assert!(value.is_table());
        assert_eq!(value.as_table(), Some(table_ref));
    }

    #[test]
    fn table_refcount_is_correctly_accounted_for() {
        let table_ref = TableRef::new();
        assert_eq!(Rc::strong_count(&table_ref.0), 1);

        let value = CompactLuaValue::table(table_ref.clone());
        assert_eq!(Rc::strong_count(&table_ref.0), 2);

        let accessed_table = value.as_table().unwrap();
        assert_eq!(Rc::strong_count(&table_ref.0), 3);

        drop(value);
        drop(accessed_table);
        assert_eq!(Rc::strong_count(&table_ref.0), 1);
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn strings_are_stored_properly(str: String) {
        let value = CompactLuaValue::string(&str);
        assert!(value.is_string());
        assert_eq!(value.as_str(), Some(str.as_ref()));
    }

    // #[test]
    // #[ignore = ".clone() isn't implemented yet. + this delves into quite a bit of an implementation details"]
    // fn strings_are_refcounted_properly() {
    //     let value = CompactLuaValue::string("A pretty long string to avoid fitting into possible SSO");
    //     let Some(header_ptr) = value.as_shared_string_ptr() else { panic!() };

    //     // SAFETY: Yes, this is going into the implementation details. Yes, I am relying on the
    //     //         fact that the StringHeader allocation is shared between instances of
    //     //         CompactLuaValue. Yes, I rely on that pointer to header isn't invalidated by the
    //     //         call to .clone().
    //     unsafe {
    //         assert_eq!(header_ptr.as_ref().refcount, 0);

    //         let copy = value.clone();
    //         assert_eq!(header_ptr.as_ref().refcount, 1);

    //         drop(copy);
    //         assert_eq!(header_ptr.as_ref().refcount, 0);
    //     }
    // }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn code_block_ids_are_stored_properly(raw_block_id: u32) {
        use crate::ids::BlockID;

        let block_id = BlockID(raw_block_id);
        let value = CompactLuaValue::lua_function(block_id);
        assert!(value.is_lua_function());
        assert!(value.is_function());
        assert_eq!(value.as_lua_function(), Some(block_id));
    }

    #[test]
    fn native_functions_are_stored_properly() {
        let func = NativeFunction::new(|| {});
        let value = CompactLuaValue::native_function(func.clone());
        assert!(value.is_native_function());
        assert!(value.is_function());
        assert_eq!(value.as_native_function(), Some(func));
    }

    #[test]
    fn native_function_refcount_is_correctly_accounted_for() {
        let func_ref = NativeFunction::new(|| {});
        assert_eq!(Rc::strong_count(&func_ref.0), 1);

        let value = CompactLuaValue::native_function(func_ref.clone());
        assert_eq!(Rc::strong_count(&func_ref.0), 2);

        let accessed_func = value.as_native_function().unwrap();
        assert_eq!(Rc::strong_count(&func_ref.0), 3);

        drop(value);
        drop(accessed_func);
        assert_eq!(Rc::strong_count(&func_ref.0), 1);
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn floats_are_stored_properly(float: f64) {
        let value = CompactLuaValue::float(float);
        assert!(value.is_float());

        let Some(inner) = value.as_float() else { panic!() };
        if float.is_nan() {
            assert!(inner.is_nan());
        } else {
            assert_eq!(float, inner);
        }
    }
}
