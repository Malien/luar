use reggie::{stdlib::floor, LuaValue, TableRef, TableValue, NativeFunction};

// use crate::eq_with_nan;

// #[quickcheck]
// fn floor_floor_numbers(num: f64) {

//         floor(&LuaValue::Float(num))
//             .unwrap()

//     assert!(eq_with_nan(
//         num.as_f64().floor()
//     ));

//     assert!(eq_with_nan(
//         floor(&[LuaValue::string(format!("{}", num))])
//             .unwrap()
//             .unwrap_number()
//             .as_f64(),
//         num.as_f64().floor()
//     ));
// }

#[test]
fn flooring_unconvertible_values_is_an_error() {
    let unsupported = [
        LuaValue::Nil,
        LuaValue::string("hello"),
        LuaValue::Table(TableRef::from(TableValue::new())),
        LuaValue::native_function(|| ()),
    ];
    for value in unsupported {
        assert!(floor(&value).is_err())
    }
}
