use std::{cell::RefCell, fmt, ptr::NonNull, rc::Rc};

use crate::{eq_with_nan::eq_with_nan, ids::BlockID, LuaValue, NativeFunction, NativeFunctionKind, TableRef, TableValue};

use super::{string::{CompactString, SharedStringPtr}, FFIFunc, FromArgs};

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
// The current list is just the platforms I have (kinda) tested.
// Meaning the encode_pointer assertion didn't fail.
const fn is_compatible_with_48bit_pointers() -> bool {
    cfg!(all(target_os = "macos", target_arch = "x86_64")) || 
    cfg!(all(target_os = "linux", target_arch = "x86_64")) ||
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
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

    pub const fn int(x: i32) -> Self {
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
        Self::from_shared_string_ptr(SharedStringPtr::alloc_and_copy(str.as_ref()))
    }

    /// More efficient transformation than going through AsRef<str>
    pub fn from_compact_string(str: CompactString) -> Self {
        Self::from_shared_string_ptr(str.leak())
    }

    fn from_shared_string_ptr(ptr: SharedStringPtr) -> Self {
        let payload = unsafe { Self::encode_pointer(ptr.0) };
        Self(STRING_BITPATTERN | payload)
    }

    fn as_string_ptr(&self) -> Option<SharedStringPtr> {
        if self.is_string() {
            unsafe {
                Some(SharedStringPtr(self.decode_pointer().cast()))
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
        self.as_string_ptr().map(|ptr| unsafe { ptr.str_ref() })
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

    pub fn function<'a, F, Args>(func: F) -> Self
    where
        F: FFIFunc<Args> + 'static,
        Args: FromArgs<'a> + 'static,
    {
        Self::native_function(NativeFunction::new(func))
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

    pub fn coerce_to_f64(&self) -> Option<f64> {
        if let Some(int) = self.as_int() {
            Some(int as f64)
        } else if let Some(float) = self.as_float() {
            Some(float)
        } else if let Some(str) = self.as_str() {
            str.parse().ok()
        } else {
            None
        }
    }

    pub fn coerce_to_i32(&self) -> Option<i32> {
        if let Some(int) = self.as_int() {
            Some(int)
        } else if let Some(float) = self.as_float() {
            Some(float as i32)
        } else if let Some(str) = self.as_str() {
            str.parse().ok()
        } else {
            None
        }
    }

    pub const TRUE: Self = Self::int(1);
    pub const FALSE: Self = Self::NIL;

    pub fn from_bool(v: bool) -> Self {
        if v {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }

    pub fn is_falsy(&self) -> bool {
        self == &Self::NIL
    }

    pub fn total_eq(&self, other: &Self) -> bool {
        if self.is_nil() && other.is_nil() {
            true
        } else if let Some(lhs_int) = self.as_int() && let Some(rhs_int) = other.as_int() {
            lhs_int == rhs_int
        } else if let Some(lhs_float) = self.as_float() && let Some(rhs_float) = other.as_float() {
            eq_with_nan(lhs_float, rhs_float)
        } else if let Some(lhs) = self.as_str() && let Some(rhs) = other.as_str() {
            lhs == rhs
        } else if let Some(lhs) = self.as_table() && let Some(rhs) = other.as_table() {
            lhs == rhs
        } else if let Some(lhs) = self.as_native_function() && let Some(rhs) = other.as_native_function() {
            lhs == rhs
        } else if let Some(lhs) = self.as_lua_function() && let Some(rhs) = other.as_lua_function() {
            lhs == rhs
        } else {
            false
        }
    }
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
        } else if let Some(native_function_ptr) = self.as_native_function_ptr() {
            // SAFTEY: This is the only place where we decrement refcount.
            //         Every other access to the table ref should be guarded with 
            //         [CompactLuaValue::as_native_function]
            unsafe { Rc::decrement_strong_count(native_function_ptr.as_ptr()) };
        } else if let Some(str_ptr) = self.as_string_ptr() {
            unsafe { str_ptr.release() };
        }
    }
}

impl Clone for CompactLuaValue {
    fn clone(&self) -> Self {
        if let Some(table_ptr) = self.as_table_ptr() {
            unsafe { Rc::increment_strong_count(table_ptr.as_ptr()) };
        } else if let Some(native_function_ptr) = self.as_native_function_ptr() {
            unsafe { Rc::increment_strong_count(native_function_ptr.as_ptr()) };
        } else if let Some(str_ptr) = self.as_string_ptr() {
            unsafe { str_ptr.retain() };
        }
        Self(self.0)
    }
}

macro_rules! lmatch {
    (
        $value:expr; 
        nil => $nil_match:expr,
        int $int_ident:ident => $int_match:expr,
        float $float_ident:ident => $float_match:expr,
        string $string_ident:ident => $string_match:expr,
        table $table_ident:ident => $table_match:expr,
        native_function $native_function_ident:ident => $native_function_match:expr,
        lua_function $lua_function_ident:ident => $lua_function_match:expr$(,)?
    ) => {{
        let __value = $value;
        
        if __value.is_nil() {
            $nil_match
        } else if let Some($int_ident) = __value.as_int() {
            $int_match
        } else if let Some($float_ident) = __value.as_float() {
            $float_match
        } else if let Some($string_ident) = __value.as_str() {
            $string_match
        } else if let Some($table_ident) = __value.as_table() {
            $table_match
        } else if let Some($native_function_ident) = __value.as_native_function() {
            $native_function_match
        } else if let Some($lua_function_ident) = __value.as_lua_function() {
            $lua_function_match
        } else {
            unreachable!("CompactLuaValue repr cannot be anything else than nil, int, float, string, table, function")
        }}
    };
}

impl fmt::Debug for CompactLuaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CompactLuaValue::")?;

        lmatch! { self;
            nil => f.write_str("nil"),
            int x => write!(f, "int({x})"),
            float x => write!(f, "float({x})"),
            string x => write!(f, "string({x:?}"),
            table x => write!(f, "table({x:?})"),
            native_function x => write!(f, "native_function({x:?})"),
            lua_function block_id => write!(f, "lua_function({block_id:?})"),
        }
    }
}

impl std::fmt::Display for CompactLuaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        lmatch! { self;
            nil => f.write_str("nil"),
            int int => std::fmt::Display::fmt(&int, f),
            float float => std::fmt::Display::fmt(&float, f),
            string string => std::fmt::Debug::fmt(string, f),
            table table_ref => write!(f, "table: {:p}", table_ref.as_ptr()),
            native_function function => {
                write!(f, "native_function: {:p}", Rc::as_ptr(&function.0))
            },
            lua_function block_id => write!(f, "function: {:#x}", block_id.0),
        }
    }
}

impl PartialEq for CompactLuaValue {
    fn eq(&self, other: &Self) -> bool {
        // This could be a BIG shortcut for:
        // - nils
        // - ints
        // - lua functions
        // - native function pointers
        // - table pointers
        // - small strings
        // - heap strings that are the same allocation
        // - floats that are the same bit pattern
        // The only two cases that are not covered by bit-equivalence are:
        // - heap strings that are different allocations (but could be the same byte sequence)
        // - floats that are different bit patterns, but are numerically equal (thanks IEE754)
        //
        // if self.0 == other.0 {
        //     return true;
        // }

        if self.is_nil() && other.is_nil() {
            true
        } else if numeric_eq(self, other) {
            true
        } else if let Some(lhs) = self.as_str() && let Some(rhs) = other.as_str() {
            lhs == rhs
        } else if let Some(lhs) = self.as_table() && let Some(rhs) = other.as_table() {
            lhs == rhs
        } else if let Some(lhs) = self.as_native_function() && let Some(rhs) = other.as_native_function() {
            lhs == rhs
        } else if let Some(lhs) = self.as_lua_function() && let Some(rhs) = other.as_lua_function() {
            lhs == rhs
        } else {
            false
        }
    }
}

impl PartialOrd for CompactLuaValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let Some(lhs_int) = self.as_int() {
            if let Some(rhs_int) = other.as_int() {
                return lhs_int.partial_cmp(&rhs_int);
            }
            if let Some(rhs_float) = other.as_float() {
                return (lhs_int as f64).partial_cmp(&rhs_float);
            }
        } else if let Some(lhs_float) = self.as_float() {
            if let Some(rhs_float) = other.as_float() {
                return lhs_float.partial_cmp(&rhs_float);
            }
            if let Some(rhs_int) = other.as_int() {
                return lhs_float.partial_cmp(&(rhs_int as f64));
            }
        } else if let Some(lhs_str) = self.as_str() && let Some(rhs_str) = self.as_str() {
            return lhs_str.partial_cmp(rhs_str);
        } 

        return None;
    }
}

fn numeric_eq(lhs: &CompactLuaValue, rhs: &CompactLuaValue) -> bool {
    if let Some(lhs_int) = lhs.as_int() {
        if let Some(rhs_int) = rhs.as_int() {
            return lhs_int == rhs_int;
        } else if let Some(rhs_float) = rhs.as_float() {
            return lhs_int as f64 == rhs_float;
        }
    } else if let Some(lhs_float) = lhs.as_float() {
        if let Some(rhs_float) = rhs.as_float() {
            return lhs_float == rhs_float;
        } else if let Some(rhs_int) = rhs.as_int() {
            return lhs_float == rhs_int as f64;
        }
    } 
    return false;
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for LuaValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use test_util::{with_thread_gen, GenExt};

        match u8::arbitrary(g) % 6 {
            0 => LuaValue::NIL,
            1 => LuaValue::int(with_thread_gen(i32::arbitrary)),
            2 => LuaValue::float(with_thread_gen(f64::arbitrary)),
            3 => LuaValue::string(with_thread_gen(String::arbitrary)),
            4 => LuaValue::table(TableRef::arbitrary(&mut g.next_iter())),
            5 => LuaValue::native_function(|| {}),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            LuaValue::Nil => quickcheck::empty_shrinker(),
            LuaValue::Int(int) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(int.shrink().map(LuaValue::Int)))
            }
            LuaValue::Float(float) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(float.shrink().map(LuaValue::Float)))
            }
            LuaValue::String(str) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(str.shrink().map(LuaValue::String)))
            }
            LuaValue::NativeFunction(_) | LuaValue::Function(_) => {
                Box::new(std::iter::once(LuaValue::Nil))
            }
            LuaValue::Table(table) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(table.shrink().map(LuaValue::Table)))
            }
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

    #[test]
    fn cloning_tables_does_retain_them() {
        let table = CompactLuaValue::table(TableRef::new());
        assert_eq!(Rc::strong_count(&table.as_table().unwrap().0), 2);

        let clone = table.clone();
        assert_eq!(Rc::strong_count(&table.as_table().unwrap().0), 3);
        assert_eq!(table.as_table_ptr(), clone.as_table_ptr());

        drop(table);
        assert_eq!(Rc::strong_count(&clone.as_table().unwrap().0), 2);
    }

    #[test]
    fn cloning_native_functions_does_retain_them() {
        let func = CompactLuaValue::function(|| {});
        assert_eq!(Rc::strong_count(&func.as_native_function().unwrap().0), 2);

        let clone = func.clone();
        assert_eq!(Rc::strong_count(&func.as_native_function().unwrap().0), 3);
        assert_eq!(func.as_native_function_ptr(), clone.as_native_function_ptr());

        drop(func);
        assert_eq!(Rc::strong_count(&clone.as_native_function().unwrap().0), 2);
    }

    #[test]
    fn cloning_strings_does_retain_them() {
        let str = CompactLuaValue::string("A long-enough string not to fit into SSO");
        assert_eq!(str.as_string().unwrap().refcount(), 2);

        let clone = str.clone();
        assert_eq!(str.as_string().unwrap().refcount(), 3);
        assert_eq!(str.as_string_ptr(), clone.as_string_ptr());

        drop(str);
        assert_eq!(clone.as_string().unwrap().refcount(), 2);
    }

    // #[cfg(feature = "quickcheck")]
    // #[quickcheck]
    // fn cloning_lua_value_yields_the_same_value(value: CompactLuaValue) {
    //     let clone = value.clone();
    //     assert_eq!(value, clone);
    // }
}
