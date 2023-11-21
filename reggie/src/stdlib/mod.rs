use luar_error::ExpectedType;

use crate::{trace_execution, EvalError, GlobalValues, LuaValue, TypeError};

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

pub fn floor(value: &LuaValue) -> Result<LuaValue, TypeError> {
    match value {
        LuaValue::Int(int) => Ok(LuaValue::Int(*int)),
        LuaValue::Float(float) if is_within_int_range(*float) => {
            Ok(LuaValue::Int(float.floor() as i32))
        }
        LuaValue::Float(float) => Ok(LuaValue::Float(float.floor())),
        LuaValue::String(str) => match str.parse::<f64>() {
            Ok(float) if is_within_int_range(float) => Ok(LuaValue::Int(float.floor() as i32)),
            Ok(float) => Ok(LuaValue::Float(float.floor())),
            Err(_) => Err(TypeError::ArgumentType {
                position: 0,
                expected: ExpectedType::Number,
                got: LuaValue::String(str.clone()),
            }),
        },
        value => Err(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::Number,
            got: value.clone(),
        }),
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

pub fn strlen(value: &LuaValue) -> Result<LuaValue, TypeError> {
    match value {
        LuaValue::String(str) => Ok(LuaValue::Int(str.len().try_into().unwrap())),
        LuaValue::Int(int) => Ok(LuaValue::Int(format!("{}", int).len().try_into().unwrap())),
        LuaValue::Float(float) => Ok(LuaValue::Int(
            (format!("{}", float).len()).try_into().unwrap(),
        )),
        value => Err(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::String,
            got: value.clone(),
        }),
    }
}

pub fn define_stdlib(global_values: &mut GlobalValues) {
    global_values.set("assert", LuaValue::native_function(assert));
    global_values.set("floor", LuaValue::native_function(floor));
    global_values.set("random", LuaValue::native_function(random));
    global_values.set("type", LuaValue::native_function(lua_type));
    global_values.set("strlen", LuaValue::native_function(strlen));
}

#[cfg(test)]
#[cfg(feature = "quickcheck")]
mod test {
    use super::*;

    #[quickcheck]
    fn strlen_returns_the_number_of_bytes_in_a_string(str: String) {
        let len = str.len();
        let res = strlen(&LuaValue::String(str)).unwrap();
        assert_eq!(res, LuaValue::Int(len as i32));
    }

    #[quickcheck]
    fn strlen_returns_the_number_of_bytes_in_a_stringified_int(num: i32) {
        let str = format!("{}", num);
        let len = str.len();
        let res = strlen(&LuaValue::Int(num)).unwrap();
        assert_eq!(res, LuaValue::Int(len as i32));
    }
    
    #[quickcheck]
    fn strlen_returns_the_number_of_bytes_in_a_stringified_float(num: f64) {
        let str = format!("{}", num);
        let len = str.len();
        let res = strlen(&LuaValue::Float(num)).unwrap();
        assert_eq!(res, LuaValue::Int(len as i32));
    }

    #[quickcheck]
    fn strlen_on_non_stringable_errors_out(value: LuaValue) {
        if value.is_string() || value.is_int() || value.is_float() {
            return;
        }
        assert!(strlen(&value).is_err());
    }
}
