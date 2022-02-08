use crate::lang::LuaValue;

pub fn tonumber(args: &[LuaValue]) -> LuaValue {
    if let Some(arg) = args.first() {
        arg.as_number()
            .map(LuaValue::Number)
            .unwrap_or(LuaValue::Nil)
    } else {
        LuaValue::Nil
    }
}

#[cfg(test)]
mod test {
    use crate::{lang::LuaValue, stdlib::fns::tonumber, util::close_relative_eq};

    #[test]
    fn tonumber_on_zero_args_returns_nil() {
        assert_eq!(tonumber(&[]), LuaValue::Nil);
    }

    #[quickcheck]
    fn tonumber_on_number_returns_self(num: f64) {
        assert!(tonumber(&[LuaValue::Number(num)]).total_eq(&LuaValue::Number(num)));
    }

    #[quickcheck]
    fn tonumber_on_number_looking_string_returns_parsed_number(num: f64) {
        let str = num.to_string();
        let res = tonumber(&[LuaValue::String(str)]);
        assert!(res.is_number());
        let resnum = res.unwrap_number();
        if num.is_nan() {
            assert!(resnum.is_nan())
        } else {
            assert!(close_relative_eq(resnum, num));
        }
    }
}
