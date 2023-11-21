use luar_error::ExpectedType;
use std::{
    cmp::{max, min},
    io::{self, Write}, rc::Rc,
};

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
        LuaValue::String(str) => Ok(LuaValue::int(str.len())),
        LuaValue::Int(int) => Ok(LuaValue::int(format!("{}", int).len())),
        LuaValue::Float(float) => Ok(LuaValue::int(format!("{}", float).len())),
        value => Err(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::String,
            got: value.clone(),
        }),
    }
}

pub fn strsub(value: &LuaValue, from: &LuaValue, to: &LuaValue) -> Result<LuaValue, TypeError> {
    let str = value
        .coerce_to_string()
        .ok_or_else(|| TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::String,
            got: value.clone(),
        })?;
    let from = from
        .coerce_to_i32()
        .ok_or_else(|| TypeError::ArgumentType {
            position: 1,
            expected: ExpectedType::Number,
            got: from.clone(),
        })?;

    if from < 1 {
        return Ok(value.clone());
    }
    let from = min(max(from - 1, 0) as usize, str.len());

    if let LuaValue::Nil = to {
        return Ok(LuaValue::string(&str[from..]));
    }

    let to = to.coerce_to_i32().ok_or_else(|| TypeError::ArgumentType {
        position: 2,
        expected: ExpectedType::Number,
        got: to.clone(),
    })?;

    if to < 1 {
        return Ok(LuaValue::string(""));
    }
    let to = min(max(to, 0) as usize, str.len());

    if from > to {
        return Ok(LuaValue::string(""));
    }

    return Ok(LuaValue::string(&str[from..to]));
}

pub fn print_stdout(args: &[LuaValue]) -> Result<(), EvalError> {
    print(&mut std::io::stdout(), args)
}

fn print_repr(writer: &mut impl Write, value: &LuaValue) -> Result<(), io::Error> {
    match value {
        LuaValue::Nil => writer.write_all("nil".as_bytes()).map(|_| ()),
        LuaValue::String(str) => writer.write_all(str.as_bytes()).map(|_| ()),
        LuaValue::Int(num) => write!(writer, "{num}"),
        LuaValue::Float(num) => write!(writer, "{num}"),
        LuaValue::Function(func) => write!(writer, "function: {func:?}"),
        LuaValue::NativeFunction(func) => write!(writer, "function: {:p}", Rc::as_ptr(&func.0)),
        LuaValue::Table(table) => write!(writer, "table: {:p}", table.as_ptr()),
    }
}

pub fn print(writer: &mut impl Write, args: &[LuaValue]) -> Result<(), EvalError> {
    if let Some((first, rest)) = args.split_first() {
        print_repr(writer, first).map_err(EvalError::IO)?;
        for arg in rest {
            writer.write_all(b"\t").map_err(EvalError::IO)?;
            print_repr(writer, arg).map_err(EvalError::IO)?;
        }
    }
    writer.write_all(b"\n").map_err(EvalError::IO)?;
    Ok(())
}

pub fn define_stdlib(global_values: &mut GlobalValues) {
    global_values.set("assert", LuaValue::native_function(assert));
    global_values.set("floor", LuaValue::native_function(floor));
    global_values.set("random", LuaValue::native_function(random));
    global_values.set("type", LuaValue::native_function(lua_type));
    global_values.set("strlen", LuaValue::native_function(strlen));
    global_values.set("strsub", LuaValue::native_function(strsub));
    global_values.set("print", LuaValue::native_function(print_stdout));
}

#[cfg(test)]
#[cfg(feature = "quickcheck")]
mod test {
    use std::{io::Cursor, rc::Rc};

    use crate::NativeFunction;

    use super::*;

    #[test]
    fn printing_with_no_args_prints_newline() {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[]).unwrap();
        assert_eq!(buf.into_inner(), vec![b'\n']);
    }

    #[test]
    fn printing_nil_prints_nil() {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::Nil]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, "nil\n");
    }

    #[quickcheck]
    fn printing_string_prints_its_value(str: String) {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::String(str.clone())]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("{}\n", str));
    }

    #[quickcheck]
    fn printing_int_prints_string_repr(num: i32) {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::Int(num)]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("{}\n", num));
    }

    #[quickcheck]
    fn printing_float_prints_string_repr(num: f64) {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::Float(num)]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("{}\n", num));
    }

    #[quickcheck]
    fn printing_function_prints_it_address() {
        let func = NativeFunction::new(|| ());
        let addr = Rc::as_ptr(&func.0);
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::NativeFunction(func)]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("function: {:p}\n", addr));
    }

    #[quickcheck]
    fn multiple_values_are_printed_separated_by_tabs(values: Vec<LuaValue>) {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &values).unwrap();
        let res_str = String::from_utf8(buf.into_inner()).unwrap();

        let mut expected_buf = Cursor::new(Vec::new());
        if let Some((first, rest)) = values.split_first() {
            print(&mut expected_buf, std::slice::from_ref(&first)).unwrap();
            for value in rest {
                *expected_buf.get_mut().last_mut().unwrap() = b'\t';
                print(&mut expected_buf, std::slice::from_ref(&value)).unwrap();
            }
        }
        let expected_str = String::from_utf8(expected_buf.into_inner()).unwrap();
        assert_eq!(res_str, expected_str);
    }

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

    #[quickcheck]
    fn strsub_slices_string_suffix(str: String, start: i32) {
        let suffix_start = if start <= 1 {
            0
        } else if start as usize >= str.len() + 1 {
            str.len()
        } else {
            start as usize - 1
        };

        let expected_suffix = LuaValue::string(&str[suffix_start..]);

        let res = strsub(
            &LuaValue::String(str),
            &LuaValue::Int(start as i32),
            &LuaValue::Nil,
        );
        assert_eq!(res, Ok(expected_suffix));
    }

    #[quickcheck]
    fn strsub_errors_out_on_non_stringables(value: LuaValue, start: u16, end: Option<u16>) {
        if value.is_string() || value.is_int() || value.is_float() {
            return;
        }
        if let Some(end) = end {
            assert!(dbg!(strsub(&value, &LuaValue::int(start), &LuaValue::int(end))).is_err());
        } else {
            assert!(strsub(&value, &LuaValue::int(start), &LuaValue::Nil).is_err());
        }
    }
}
