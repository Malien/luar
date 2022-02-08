use super::LuaValue;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Continue,
    Return(LuaValue),
}

impl ControlFlow {
    pub fn function_return(self) -> LuaValue {
        self.return_value().unwrap_or(LuaValue::Nil)
    }

    pub fn return_value(self) -> Option<LuaValue> {
        match self {
            Self::Continue => None,
            Self::Return(value) => Some(value),
        }

    }
}
