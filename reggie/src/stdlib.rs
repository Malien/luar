use std::{
    cmp::{max, min},
    io::{self, Write}, rc::Rc,
};

use crate::{lmatch, trace_execution, EvalError, ExpectedType, GlobalValues, LuaValue, TypeError};

pub fn assert(value: LuaValue, message: LuaValue) -> Result<(), EvalError> {
    trace_execution!("assert({:?}, {:?})", value, message);
    if value.is_truthy() {
        Ok(())
    } else if message.is_nil() {
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

fn is_within_int_range(float: f64) -> bool {
    float >= i32::MIN as f64 && float < i32::MAX as f64 + 1.0
}

pub fn floor(value: &LuaValue) -> Result<LuaValue, TypeError> {
    // TODO: Coerce to LuaValue::int if possible
    if let Some(int) = value.as_int() {
        Ok(LuaValue::int(int))
    } else if let Some(float) = value.as_float() {
        Ok(LuaValue::float(float.floor()))
    } else if let Some(string) = value.as_string() {
        match string.as_ref().parse::<f64>() {
            Ok(float) => Ok(LuaValue::float(float.floor())),
            Err(_) => Err(TypeError::ArgumentType {
                position: 0,
                expected: ExpectedType::Number,
                got: LuaValue::from_compact_string(string),
            }),
        }
    } else {
        Err(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::Number,
            got: value.clone(),
        })
    }

    // match value {
    //     LuaValue::Int(int) => Ok(LuaValue::Int(*int)),
    //     LuaValue::Float(float) if is_within_int_range(*float) => {
    //         Ok(LuaValue::Int(float.floor() as i32))
    //     }
    //     LuaValue::Float(float) => Ok(LuaValue::Float(float.floor())),
    //     LuaValue::String(str) => match str.parse::<f64>() {
    //         Ok(float) if is_within_int_range(float) => Ok(LuaValue::Int(float.floor() as i32)),
    //         Ok(float) => Ok(LuaValue::Float(float.floor())),
    //         Err(_) => Err(TypeError::ArgumentType {
    //             position: 0,
    //             expected: ExpectedType::Number,
    //             got: LuaValue::String(str.clone()),
    //         }),
    //     },
    //     value => Err(TypeError::ArgumentType {
    //         position: 0,
    //         expected: ExpectedType::Number,
    //         got: value.clone(),
    //     }),
    // }
}

pub fn random() -> LuaValue {
    // SAFETY: libc rand function should always be safe to call
    let int_value = unsafe { libc::rand() };
    let float_value = int_value as f64 / libc::INT_MAX as f64;
    return LuaValue::float(float_value);
}

pub fn lua_type(value: &LuaValue) -> LuaValue {
    lmatch! { value;
        nil => LuaValue::string("nil"),
        int _ => LuaValue::string("number"),
        float _ => LuaValue::string("number"),
        string _ => LuaValue::string("string"),
        table _ => LuaValue::string("table"),
        native_function _ => LuaValue::string("function"),
        lua_function _ => LuaValue::string("function"),
    }
}

pub fn strlen(value: &LuaValue) -> Result<LuaValue, TypeError> {
    if let Some(string) = value.as_str() {
        Ok(LuaValue::int(string.len().try_into().expect("String length exceeds i32 range")))
    } else if let Some(int) = value.as_int() {
        Ok(LuaValue::int(format!("{}", int).len() as i32))
    } else if let Some(float) = value.as_float() {
        Ok(LuaValue::int(format!("{}", float).len() as i32))
    } else {
        return Err(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::String,
            got: value.clone(),
        });
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
    let from = min(max(from - 1, 0) as usize, str.len() as usize);

    if to.is_nil() {
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
    let to = min(max(to, 0) as usize, str.len() as usize);

    if from > to {
        return Ok(LuaValue::string(""));
    }

    return Ok(LuaValue::string(&str[from..to]));
}

pub fn print_stdout(args: &[LuaValue]) -> Result<(), EvalError> {
    print(&mut std::io::stdout(), args)
}

fn print_repr(writer: &mut impl Write, value: &LuaValue) -> Result<(), io::Error> {
    lmatch! { value;
        nil => writer.write_all("nil".as_bytes()).map(|_| ()),
        int num => write!(writer, "{num}"),
        float num => write!(writer, "{num}"),
        string ref str => writer.write_all(str.as_bytes()).map(|_| ()),
        table table => write!(writer, "table: {:p}", table.as_ptr()),
        native_function func => write!(writer, "function: {:p}", Rc::as_ptr(&func.0)),
        lua_function func => write!(writer, "function: {func:?}"),
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
    global_values.set("assert", LuaValue::function(assert));
    global_values.set("floor", LuaValue::function(floor));
    global_values.set("random", LuaValue::function(random));
    global_values.set("type", LuaValue::function(lua_type));
    global_values.set("strlen", LuaValue::function(strlen));
    global_values.set("strsub", LuaValue::function(strsub));
    global_values.set("print", LuaValue::function(print_stdout));
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::TableRef;
    use std::io::Cursor;

    #[test]
    fn flooring_unconvertible_values_is_an_error() {
        let unsupported = [
            LuaValue::NIL,
            LuaValue::string("hello"),
            LuaValue::table(TableRef::new()),
            LuaValue::function(|| ()),
        ];
        for value in unsupported {
            assert!(floor(&value).is_err())
        }
    }

    #[test]
    fn printing_with_no_args_prints_newline() {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[]).unwrap();
        assert_eq!(buf.into_inner(), vec![b'\n']);
    }

    #[test]
    fn printing_nil_prints_nil() {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::NIL]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, "nil\n");
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn floor_floor_numbers(num: f64) {
        use crate::eq_with_nan::eq_with_nan;
        use luar_string::lua_format;

        let res = floor(&LuaValue::float(num)).unwrap();

        assert!(eq_with_nan(res.number_as_f64().unwrap(), num.floor()));

        let res = floor(&LuaValue::string(lua_format!("{num}"))).unwrap();
        assert!(eq_with_nan(res.number_as_f64().unwrap(), num.floor()));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn printing_string_prints_its_value(str: String) {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::string(&str)]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("{}\n", str));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn printing_int_prints_string_repr(num: i32) {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::int(num)]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("{}\n", num));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn printing_float_prints_string_repr(num: f64) {
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::float(num)]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("{}\n", num));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn printing_function_prints_it_address() {
        use crate::NativeFunction;
        use std::rc::Rc;

        let func = NativeFunction::new(|| ());
        let addr = Rc::as_ptr(&func.0);
        let mut buf = Cursor::new(Vec::new());
        print(&mut buf, &[LuaValue::native_function(func)]).unwrap();
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("function: {:p}\n", addr));
    }

    #[cfg(feature = "quickcheck")]
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
            let expected_str = String::from_utf8(expected_buf.into_inner()).unwrap();
            assert_eq!(res_str, expected_str);
        } else {
            assert_eq!(res_str, "\n");
        }
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn strlen_returns_the_number_of_bytes_in_a_string(str: luar_string::LuaString) {
        let len = str.len();
        let res = strlen(&LuaValue::string(str)).unwrap();
        assert_eq!(res, LuaValue::int(len as i32));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn strlen_returns_the_number_of_bytes_in_a_stringified_int(num: i32) {
        let str = format!("{}", num);
        let len = str.len();
        let res = strlen(&LuaValue::int(num)).unwrap();
        assert_eq!(res, LuaValue::int(len as i32));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn strlen_returns_the_number_of_bytes_in_a_stringified_float(num: f64) {
        let str = format!("{}", num);
        let len = str.len();
        let res = strlen(&LuaValue::float(num)).unwrap();
        assert_eq!(res, LuaValue::int(len as i32));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn strlen_on_non_stringable_errors_out(value: LuaValue) {
        if value.is_string() || value.is_int() || value.is_float() {
            return;
        }
        assert!(strlen(&value).is_err());
    }

    #[cfg(feature = "quickcheck")]
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
            &LuaValue::string(str),
            &LuaValue::int(start as i32),
            &LuaValue::NIL,
        );
        assert_eq!(res, Ok(expected_suffix));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn strsub_errors_out_on_non_stringables(value: LuaValue, start: u16, end: Option<u16>) {
        if value.is_string() || value.is_int() || value.is_float() {
            return;
        }
        if let Some(end) = end {
            assert!(dbg!(strsub(&value, &LuaValue::int(start as i32), &LuaValue::int(end as i32))).is_err());
        } else {
            assert!(strsub(&value, &LuaValue::int(start as i32), &LuaValue::NIL).is_err());
        }
    }
}
