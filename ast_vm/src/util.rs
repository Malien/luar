use std::cmp::Ordering;
use num::abs;

pub fn eq_with_nan(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        true
    } else if a.is_infinite() && b.is_infinite() {
        a.is_sign_negative() == b.is_sign_negative()
    } else {
        a == b
    }
}

#[allow(dead_code)]
pub fn close_relative_eq(a: f64, b: f64) -> bool {
    let absolute_value = partial_min(abs(a), abs(b)).unwrap();
    let magnitude = if absolute_value < 10f64 {
        1f64
    } else {
        absolute_value
    };
    close_eq(a, b, 0.0000001 * magnitude)
}

fn partial_min<T: PartialOrd>(v1: T, v2: T) -> Option<T> {
    PartialOrd::partial_cmp(&v1, &v2).map(|order| match order {
        Ordering::Less | Ordering::Equal => v1,
        Ordering::Greater => v2,
    })
}

fn close_eq(a: f64, b: f64, eps: f64) -> bool {
    if a.is_infinite() && b.is_infinite() {
        return true;
    }
    abs(a - b) < eps
}
