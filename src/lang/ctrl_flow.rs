use super::ReturnValue;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Continue,
    Return(ReturnValue),
}

impl ControlFlow {
    pub fn function_return(self) -> ReturnValue {
        self.return_value().unwrap_or(ReturnValue::Nil)
    }

    pub fn return_value(self) -> Option<ReturnValue> {
        match self {
            Self::Continue => None,
            Self::Return(value) => Some(value),
        }
    }
}
