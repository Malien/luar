use std::io::Write;

use luar_error::ExpectedType;

use crate::{lang::LuaValue, EvalError, TypeError};

pub fn tonumber(args: &[LuaValue]) -> LuaValue {
    if let Some(arg) = args.first() {
        arg.as_number()
            .map(LuaValue::Number)
            .unwrap_or(LuaValue::Nil)
    } else {
        LuaValue::Nil
    }
}

pub fn print_stdout(args: &[LuaValue]) -> Result<LuaValue, EvalError> {
    print(&mut std::io::stdout(), args)
}

pub fn print(writer: &mut impl Write, args: &[LuaValue]) -> Result<LuaValue, EvalError> {
    for arg in args {
        let res = match arg {
            LuaValue::Nil => writer.write("nil\n".as_bytes()).map(|_| ()),
            LuaValue::String(str) => writer
                .write(str.as_bytes())
                .and_then(|_| writer.write(&['\n' as u8]))
                .map(|_| ()),
            LuaValue::Number(num) => write!(writer, "{}\n", num),
            LuaValue::Function(func) => write!(writer, "function: {:p}\n", func.addr()),
            LuaValue::Table(table) => write!(writer, "table: {:p}\n", table.addr()),
        };
        if let Err(err) = res {
            return Err(EvalError::IO(err));
        }
    }
    Ok(LuaValue::Nil)
}

pub fn random(_: &[LuaValue]) -> LuaValue {
    // SAFETY: libc rand function should always be safe to call
    let int_value = unsafe { libc::rand() };
    let float_value = int_value as f64 / libc::INT_MAX as f64;
    return LuaValue::number(float_value);
}

pub fn floor(args: &[LuaValue]) -> Result<LuaValue, EvalError> {
    if let Some(arg) = args.first() {
        if let Some(num) = arg.as_number() {
            Ok(LuaValue::number(num.as_f64().floor()))
        } else {
            Err(TypeError::ArgumentType {
                position: 0,
                expected: ExpectedType::Number,
                got: arg.clone(),
            })
        }
    } else {
        Err(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::Number,
            got: LuaValue::Nil,
        })
    }
    .map_err(EvalError::from)
}

pub fn assert(args: &[LuaValue]) -> Result<LuaValue, EvalError> {
    match args.first() {
        None | Some(LuaValue::Nil) => {
            let message = args.get(1).and_then(LuaValue::coerce_to_string);
            Err(EvalError::AssertionError(message))
        }
        _ => Ok(LuaValue::Nil),
    }
}

pub fn strlen(args: &[LuaValue]) -> Result<LuaValue, EvalError> {
    if let Some(arg) = args.first() {
        if let Some(str) = arg.coerce_to_string() {
            Ok(LuaValue::Number(str.len().into()))
        } else {
            Err(TypeError::ArgumentType {
                position: 0,
                expected: ExpectedType::String,
                got: arg.clone(),
            })
        }
    } else {
        Err(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::Number,
            got: LuaValue::Nil,
        })
    }
    .map_err(EvalError::from)
}

pub fn strsub(args: &[LuaValue]) -> Result<LuaValue, EvalError> {
    match args {
        [] => Err(TypeError::ArgumentType {
            position: 0,
            expected: ExpectedType::String,
            got: LuaValue::Nil,
        }
        .into()),
        [_value] => Err(TypeError::ArgumentType {
            position: 1,
            expected: ExpectedType::Number,
            got: LuaValue::Nil,
        }
        .into()),
        [str, start] | [str, start, LuaValue::Nil, ..] => {
            let str = str
                .coerce_to_string()
                .ok_or_else(|| TypeError::ArgumentType {
                    position: 0,
                    expected: ExpectedType::String,
                    got: str.clone(),
                })?;
            let start = start.as_number().ok_or_else(|| TypeError::ArgumentType {
                position: 1,
                expected: ExpectedType::Number,
                got: start.clone(),
            })?;
            let len = str.len();
            strsub_inner(str, start.into(), len)
        }
        [str, start, end, ..] => {
            let str = str
                .coerce_to_string()
                .ok_or_else(|| TypeError::ArgumentType {
                    position: 0,
                    expected: ExpectedType::String,
                    got: str.clone(),
                })?;
            let start = start.as_number().ok_or_else(|| TypeError::ArgumentType {
                position: 1,
                expected: ExpectedType::Number,
                got: start.clone(),
            })?;
            let end = end.as_number().ok_or_else(|| TypeError::ArgumentType {
                position: 2,
                expected: ExpectedType::Number,
                got: end.clone(),
            })?;
            strsub_inner(str, start.into(), end.into())
        }
    }
}

fn strsub_inner(str: String, start: usize, end: usize) -> Result<LuaValue, EvalError> {
    if start > end {
        return Ok(LuaValue::String("".to_string()));
    }
    if start > str.len() {
        return Ok(LuaValue::String("".to_string()));
    }
    if !str.is_char_boundary(start) {
        return Err(EvalError::Utf8Error);
    }
    if end > str.len() {
        return Ok(LuaValue::String(str[start..].to_string()));
    }
    if !str.is_char_boundary(end) {
        return Err(EvalError::Utf8Error);
    }
    Ok(LuaValue::String(str[start..end].to_string()))
}

pub fn lua_type(args: &[LuaValue]) -> LuaValue {
    let val = args.first().unwrap_or(&LuaValue::Nil);
    LuaValue::string(match val {
        LuaValue::Nil => "nil",
        LuaValue::Number(_) => "number",
        LuaValue::String(_) => "string",
        LuaValue::Function(_) => "function",
        LuaValue::Table(_) => "table",
    })
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use quickcheck::TestResult;

    use super::{assert, floor, print, random, strlen, strsub, tonumber};
    use crate::{
        lang::{LuaFunction, LuaNumber, LuaValue, ReturnValue, TableValue},
        util::{close_relative_eq, eq_with_nan},
        EvalError,
    };

    #[test]
    fn tonumber_on_zero_args_returns_nil() {
        assert_eq!(tonumber(&[]), LuaValue::Nil);
    }

    #[quickcheck]
    fn tonumber_on_number_returns_self(num: f64) {
        assert!(tonumber(&[LuaValue::number(num)]).total_eq(&LuaValue::number(num)));
    }

    #[quickcheck]
    fn tonumber_on_number_looking_string_returns_parsed_number(num: f64) {
        let str = num.to_string();
        let res = tonumber(&[LuaValue::String(str)]);
        assert!(res.is_number());
        let resnum = res.unwrap_number().as_f64();
        if num.is_nan() {
            assert!(resnum.is_nan())
        } else {
            assert!(close_relative_eq(resnum, num));
        }
    }

    #[test]
    fn printing_with_no_args_prints_nothing() {
        let mut buf = Cursor::new(Vec::new());
        let res = print(&mut buf, &[]).unwrap();
        assert_eq!(res, LuaValue::Nil);
        assert!(buf.into_inner().is_empty());
    }

    #[test]
    fn printing_nil_prints_nil() {
        let mut buf = Cursor::new(Vec::new());
        let res = print(&mut buf, &[LuaValue::Nil]).unwrap();
        assert_eq!(res, LuaValue::Nil);
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, "nil\n");
    }

    #[quickcheck]
    fn printing_string_prints_its_value(str: String) {
        let mut buf = Cursor::new(Vec::new());
        let res = print(&mut buf, &[LuaValue::String(str.clone())]).unwrap();
        assert_eq!(res, LuaValue::Nil);
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("{}\n", str));
    }

    #[quickcheck]
    fn printing_number_prints_string_repr(num: LuaNumber) {
        let mut buf = Cursor::new(Vec::new());
        let res = print(&mut buf, &[LuaValue::Number(num)]).unwrap();
        assert_eq!(res, LuaValue::Nil);
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("{}\n", num));
    }

    #[quickcheck]
    fn printing_function_prints_it_address() {
        let func = LuaFunction::new(|_, _| Ok(ReturnValue::Nil));
        let addr = func.addr();
        let mut buf = Cursor::new(Vec::new());
        let res = print(&mut buf, &[LuaValue::Function(func)]).unwrap();
        assert_eq!(res, LuaValue::Nil);
        let res = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(res, format!("function: {:p}\n", addr));
    }

    #[quickcheck]
    fn multiple_values_are_printed_on_separate_lines(values: Vec<LuaValue>) {
        let mut buf = Cursor::new(Vec::new());
        let res = print(&mut buf, &values).unwrap();
        assert_eq!(res, LuaValue::Nil);
        let res_str = String::from_utf8(buf.into_inner()).unwrap();

        let mut expected_buf = Cursor::new(Vec::new());
        for value in values {
            print(&mut expected_buf, std::slice::from_ref(&value)).unwrap();
        }
        let expected_str = String::from_utf8(expected_buf.into_inner()).unwrap();
        assert_eq!(res_str, expected_str);
    }

    #[test]
    fn random_produces_values_from_0_to_1() {
        for _ in 0..1000 {
            let res = random(&[]).unwrap_number().as_f64();
            assert!((0.0..=1.0).contains(&res));
        }
    }

    #[quickcheck]
    fn floor_floor_numbers(num: LuaNumber) {
        assert!(eq_with_nan(
            floor(&[LuaValue::number(num)])
                .unwrap()
                .unwrap_number()
                .as_f64(),
            num.as_f64().floor()
        ));

        assert!(eq_with_nan(
            floor(&[LuaValue::string(format!("{}", num))])
                .unwrap()
                .unwrap_number()
                .as_f64(),
            num.as_f64().floor()
        ));
    }

    #[test]
    fn flooring_unconvertible_values_is_an_error() {
        let unsupported = [
            LuaValue::Nil,
            LuaValue::string("hello"),
            LuaValue::table(TableValue::new()),
            LuaValue::function(|_, _| Ok(ReturnValue::Nil)),
        ];
        for value in unsupported {
            assert!(floor(&[value]).is_err())
        }
    }

    #[quickcheck]
    fn asserting_truthy_value_does_nothing(value: LuaValue) -> TestResult {
        if value.is_falsy() {
            return TestResult::discard();
        }
        assert_eq!(assert(&[value]).unwrap(), LuaValue::Nil);
        TestResult::passed()
    }

    #[test]
    fn asserting_falsy_value_produces_error() {
        assert!(matches!(assert(&[]), Err(EvalError::AssertionError(None))));
        assert!(matches!(
            assert(&[LuaValue::Nil]),
            Err(EvalError::AssertionError(None))
        ));
    }

    // #[test]
    // fn format_errors_when_no_format_string_is_passed() {
    //     let res = format(&[]);
    //     assert!(res.is_err());
    // }

    // #[quickcheck]
    // fn formatting_string_without_escapes_results_in_the_same_string(string: String) -> TestResult {
    //     if let Some(_) = string
    //         .matches("%c|%d|%E|%e|%f|%g|%i|%o|%u|%X|%x|%q|%s")
    //         .next()
    //     {
    //         return TestResult::discard();
    //     }
    //     let res = format(&[LuaValue::String(string.clone())]);
    //     assert!(res.is_ok());
    //     let value = res.unwrap();
    //     assert_eq!(value, LuaValue::String(string));
    //     TestResult::passed()
    // }

    #[quickcheck]
    fn strlen_returns_the_number_of_bytes_in_a_string(str: String) {
        let len = str.len();
        let res = strlen(&[LuaValue::String(str)]).unwrap();
        assert_eq!(res, LuaValue::Number(LuaNumber::from(len)));
    }

    #[quickcheck]
    fn strlen_returns_the_number_of_bytes_in_a_stringified_number(num: LuaNumber) {
        let str = format!("{}", num);
        let len = str.len();
        let res = strlen(&[LuaValue::Number(num)]).unwrap();
        assert_eq!(res, LuaValue::Number(LuaNumber::from(len)));
    }

    #[quickcheck]
    fn strlen_on_non_stringable_errors_out(value: LuaValue) {
        if value.is_string() || value.is_number() {
            return;
        }
        assert!(strlen(&[value]).is_err());
    }

    #[quickcheck]
    fn strsub_slices_string_suffix(str: String, start: usize) {
        let suffix = if str.len() < start {
            Some("".to_string())
        } else if str.is_char_boundary(start) {
            Some(str[start..].to_string())
        } else {
            None
        };

        let res = strsub(&[LuaValue::String(str), LuaValue::Number(start.into())]);
        if let Some(suffix) = suffix {
            if let Ok(LuaValue::String(res)) = res {
                assert_eq!(res, suffix);
            } else {
                panic!("Expected string, got {:?}", res);
            }
        } else {
            assert!(res.is_err());
        }
    }

    #[quickcheck]
    fn strsub_errors_out_on_non_stringables(value: LuaValue, start: usize, end: Option<usize>) {
        if value.is_string() || value.is_number() {
            return;
        }
        if let Some(end) = end {
            assert!(strsub(&[
                LuaValue::String("".to_string()),
                LuaValue::Number(start.into()),
                LuaValue::Number(end.into())
            ])
            .is_err());
        } else {
            assert!(strsub(&[value, LuaValue::Number(start.into())]).is_err());
        }
    }

    // God! I hate implicit conversions, and the hell I have to go through to support them
    // I'm not going to write these tests anymore. I just don't care about being spec compliant
    // at this point.
}
