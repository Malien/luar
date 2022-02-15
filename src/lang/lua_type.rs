#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LuaType {
    Nil,
    Number,
    String,
    Function,
    // Table
    // UserData
}

impl std::fmt::Display for LuaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => "nil",
            Self::Number => "number",
            Self::String => "string",
            Self::Function => "function"
        }.fmt(f)
    }
}