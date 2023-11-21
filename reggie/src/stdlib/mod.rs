use luar_error::ExpectedType;

use crate::{EvalError, GlobalValues, LuaValue, NativeFunction, TypeError, trace_execution};

// fn assert_none() -> Result<(), EvalError> {
//     Err(EvalError::AssertionError)
// }

pub fn assert(value: LuaValue, message: LuaValue) -> Result<(), EvalError> {
    trace_execution!("assert({:?}, {:?})", value, message);
    if value.is_truthy() {
        Ok(())
    } else if let LuaValue::Nil = message {
        Err(EvalError::AssertionError(None))
    } else if let Some(str) = message.coerce_to_string() {
        Err(EvalError::AssertionError(Some(str)))
    } else {
        Err(EvalError::from(TypeError::ArgumentType {
            position: 1,
            expected: ExpectedType::String,
            got: message,
        }))
    }
}

// pub fn assert_overload_set() -> OverloadSet {
//     OverloadSet::new(vec![
//         OverloadRule::from(assert_none as fn() -> Result<(), EvalError>),
//         OverloadRule::from(assert as fn(LuaValue) -> Result<(), EvalError>),
//     ])
// }

fn is_within_int_range(float: f64) -> bool {
    float >= i32::MIN as f64 && float < i32::MAX as f64 + 1.0
}

pub fn floor(value: &LuaValue) -> Result<LuaValue, EvalError> {
    match value {
        LuaValue::Int(int) => Ok(LuaValue::Int(*int)),
        LuaValue::Float(float) if is_within_int_range(*float) => Ok(LuaValue::Int(float.floor() as i32)),
        LuaValue::Float(float) => Ok(LuaValue::Float(float.floor())),
        LuaValue::String(str) => match str.parse::<f64>() {
            Ok(float) if is_within_int_range(float) => Ok(LuaValue::Int(float.floor() as i32)),
            Ok(float) => Ok(LuaValue::Float(float.floor())),
            Err(_) => Err(EvalError::from(TypeError::ArgumentType {
                position: 0,
                expected: ExpectedType::Number,
                got: LuaValue::String(str.clone()),
            })),
        },
        value => Err(EvalError::from(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::Number,
            got: value.clone(),
        })),
    }
}

pub fn random() -> LuaValue {
    // SAFETY: libc rand function should always be safe to call
    let int_value = unsafe { libc::rand() };
    let float_value = int_value as f64 / libc::INT_MAX as f64;
    return LuaValue::Float(float_value);
}

pub fn lua_type(value: &LuaValue) -> LuaValue {
    match value {
        LuaValue::Nil => LuaValue::string("nil"),
        LuaValue::Int(_) => LuaValue::string("number"),
        LuaValue::Float(_) => LuaValue::string("number"),
        LuaValue::String(_) => LuaValue::string("string"),
        LuaValue::NativeFunction(_) => LuaValue::string("function"),
        LuaValue::Function(_) => LuaValue::string("function"),
        LuaValue::Table(_) => LuaValue::string("table"),
    }
}

pub fn define_stdlib(global_values: &mut GlobalValues) {
    global_values.set(
        "assert",
        LuaValue::NativeFunction(NativeFunction::new(assert)),
    );
    global_values.set(
        "floor",
        LuaValue::NativeFunction(NativeFunction::new(floor))
    );
    global_values.set(
        "random",
        LuaValue::NativeFunction(NativeFunction::new(random))
    );
    global_values.set(
        "type",
        LuaValue::NativeFunction(NativeFunction::new(lua_type))
    );
}
