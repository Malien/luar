#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LuaType {
    Nil,
    Number,
    String,
    Function,
    Table,
    // UserData
}

impl LuaType {
    pub fn is_comparable(self) -> bool {
        matches!(self, LuaType::Number | LuaType::String)
    }
}

impl std::fmt::Display for LuaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => "nil",
            Self::Number => "number",
            Self::String => "string",
            Self::Function => "function",
            Self::Table => "table",
        }
        .fmt(f)
    }
}
