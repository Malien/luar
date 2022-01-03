mod flat_intersperse;
mod non_empty;

pub use flat_intersperse::*;
pub use non_empty::*;

pub fn eq_with_nan(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        true
    } else if a.is_infinite() && b.is_infinite() {
        a.is_sign_negative() == b.is_sign_negative()
    } else {
        a == b
    }
}
