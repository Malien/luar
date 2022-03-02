use std::io::Write;

use crate::lang::{EvalError, LuaValue};

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

pub fn print<W: Write>(writer: &mut W, args: &[LuaValue]) -> Result<LuaValue, EvalError> {
    for arg in args {
        let res = match arg {
            LuaValue::Nil => writer.write("nil\n".as_bytes()).map(|_| ()),
            LuaValue::String(str) => writer
                .write(str.as_bytes())
                .and_then(|_| writer.write(&['\n' as u8]))
                .map(|_| ()),
            LuaValue::Number(num) => write!(writer, "{}\n", num),
            LuaValue::Function(func) => write!(writer, "function: {:p}\n", func.addr()),
            LuaValue::Table(table) => write!(writer, "table: {:p}\n", table.addr())
        };
        if let Err(err) = res {
            return Err(EvalError::IO(err));
        }
    }
    Ok(LuaValue::Nil)
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::{print, tonumber};
    use crate::{
        lang::{LuaFunction, LuaValue, LuaNumber, ReturnValue},
        util::close_relative_eq,
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
}
