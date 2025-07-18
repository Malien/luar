pub mod native_function;
pub use native_function::*;

pub mod traits;
pub use traits::*;

pub mod signature;

pub mod table;
pub use table::*;

pub mod key;
pub use key::*;

#[cfg(feature = "compact_value")]
mod compact;
#[cfg(feature = "compact_value")]
mod string;

#[cfg(not(feature = "compact_value"))]
mod wide;

#[cfg(not(feature = "compact_value"))]
pub type LuaValue = wide::WideLuaValue;
#[cfg(feature = "compact_value")]
pub type LuaValue = compact::CompactLuaValue;

#[cfg(not(feature = "compact_value"))]
pub type LuaString = luar_string::LuaString;
#[cfg(feature = "compact_value")]
pub type LuaString = string::CompactString;

#[cfg(not(feature = "compact_value"))]
pub(crate) use wide::lmatch;
#[cfg(feature = "compact_value")]
pub(crate) use compact::lmatch;

#[cfg(not(feature = "compact_value"))]
pub(crate) use luar_string::lua_format;
#[cfg(feature = "compact_value")]
pub(crate) use string::compact_format as lua_format;

