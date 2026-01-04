pub mod native_function;
pub use native_function::*;

pub mod traits;
pub use traits::*;

pub mod signature;

pub mod table;
pub use table::*;

pub mod key;
pub use key::*;

#[cfg(feature = "compact-value")]
mod compact;
#[cfg(feature = "compact-value")]
mod string;

#[cfg(not(feature = "compact-value"))]
mod wide;

#[cfg(not(feature = "compact-value"))]
pub type LuaValue = wide::WideLuaValue;
#[cfg(feature = "compact-value")]
pub type LuaValue = compact::CompactLuaValue;

#[cfg(not(feature = "compact-value"))]
pub type LuaString = luar_string::LuaString;
#[cfg(feature = "compact-value")]
pub type LuaString = string::CompactString;

#[cfg(not(feature = "compact-value"))]
pub(crate) use wide::lmatch;
#[cfg(feature = "compact-value")]
pub use compact::lmatch;

#[cfg(not(feature = "compact-value"))]
pub use luar_string::lua_format;
#[cfg(feature = "compact-value")]
pub use string::compact_format as lua_format;

