use num::pow;
#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};
use std::{
    fmt::Formatter,
    iter,
    str::{Chars, FromStr},
};
use thiserror::Error;

use super::{ToTokenStream, Token};

#[derive(Debug, Clone, Copy, PartialOrd)]
pub struct NumberLiteral(pub f64);

impl NumberLiteral {
    pub fn is_integer(self) -> bool {
        ((self.0 as i32) as f64) == self.0
    }
    pub fn as_i32(self) -> i32 {
        self.0 as i32
    }
    pub fn as_f64(self) -> f64 {
        self.0
    }
}

impl PartialEq for NumberLiteral {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0;
        let b = other.0;
        if a.is_nan() && b.is_nan() {
            true
        } else if a.is_infinite() && b.is_infinite() {
            a.is_sign_negative() == b.is_sign_negative()
        } else {
            a == b
        }
    }
}

impl Eq for NumberLiteral {}

impl ToTokenStream for NumberLiteral {
    type Tokens = iter::Once<Token>;
    fn to_tokens(self) -> Self::Tokens {
        iter::once(Token::Number(self))
    }
}

impl std::fmt::Display for NumberLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Error)]
#[error("Invalid number format")]
pub struct NumberLiteralParseError;

impl FromStr for NumberLiteral {
    type Err = NumberLiteralParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let mut numerator = 0_f64;
        let mut sign = 1_f64;
        match chars.next().ok_or(NumberLiteralParseError)? {
            '-' => {
                sign = -1_f64;
            }
            '+' => {}
            '.' => match parse_denumerator(&mut chars)? {
                DenumeratorParsings::Denum(denum) => return Ok(NumberLiteral(sign * denum)),
                DenumeratorParsings::DenumWithExponent(denum, exponent) => {
                    let res = if exponent >= 0 {
                        denum * pow(10_f64, exponent as usize)
                    } else {
                        denum / pow(10_f64, (-exponent) as usize)
                    };
                    return Ok(NumberLiteral(sign * res));
                }
            },
            digit_char => {
                if let Some(digit) = digit_char.to_digit(10) {
                    numerator += digit as f64;
                } else {
                    return Err(NumberLiteralParseError);
                }
            }
        };

        loop {
            match chars.next() {
                Some('e') => {
                    return parse_exponent(&mut chars)
                        .map(|exponent| number_literal_with_exponent(sign, numerator, exponent));
                }
                Some('.') => {
                    return parse_denumerator(&mut chars)
                        .map(|denum| number_literal_from_denumerator(denum, sign, numerator))
                }
                Some(digit_char) => {
                    if let Some(digit) = digit_char.to_digit(10) {
                        numerator *= 10_f64;
                        numerator += digit as f64;
                    } else {
                        return Err(NumberLiteralParseError);
                    }
                }
                None => return Ok(NumberLiteral(numerator * sign)),
            }
        }
    }
}

fn number_literal_with_exponent(sign: f64, numerator: f64, exponent: i32) -> NumberLiteral {
    if exponent >= 0 {
        NumberLiteral(sign * numerator * pow(10f64, exponent as usize))
    } else {
        NumberLiteral(sign * numerator / pow(10f64, (-exponent) as usize))
    }
}

fn number_literal_from_denumerator(
    parsings: DenumeratorParsings,
    sign: f64,
    numerator: f64,
) -> NumberLiteral {
    match parsings {
        DenumeratorParsings::Denum(denum) => NumberLiteral(sign * (numerator as f64 + denum)),
        DenumeratorParsings::DenumWithExponent(denum, exponent) => {
            let res = if exponent > 0 {
                (numerator + denum) * pow(10f64, exponent as usize)
            } else {
                (numerator + denum) / pow(10f64, (-exponent) as usize)
            };
            NumberLiteral(sign * res)
        }
    }
}

enum DenumeratorParsings {
    Denum(f64),
    DenumWithExponent(f64, i32),
}

fn parse_denumerator(chars: &mut Chars) -> Result<DenumeratorParsings, NumberLiteralParseError> {
    let mut denumerator = 0f64;
    let mut decade = 10f64;
    loop {
        match chars.next() {
            Some('e') => {
                return Ok(DenumeratorParsings::DenumWithExponent(
                    denumerator,
                    parse_exponent(chars)?,
                ))
            }
            Some(digit_char) => {
                if let Some(digit) = digit_char.to_digit(10) {
                    denumerator += (digit as f64) / decade;
                    decade *= 10f64;
                } else {
                    return Err(NumberLiteralParseError);
                }
            }
            None => return Ok(DenumeratorParsings::Denum(denumerator)),
        }
    }
}

fn parse_exponent(chars: &mut Chars) -> Result<i32, NumberLiteralParseError> {
    let mut exponent = 0;
    let mut sign = 1;
    loop {
        match chars.next() {
            Some('+') => {}
            Some('-') => {
                sign = -1;
            }
            Some(digit_char) => {
                if let Some(digit) = digit_char.to_digit(10) {
                    exponent *= 10;
                    exponent += digit as i32;
                } else {
                    return Err(NumberLiteralParseError);
                }
            }
            None => return Ok(exponent * sign),
        }
    }
}

#[cfg(feature = "quickcheck")]
impl Arbitrary for NumberLiteral {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(f64::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(|v| NumberLiteral(v)))
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use num::pow;
    use quickcheck::TestResult;

    use test_util::{Finite, NonShrinkable};

    pub fn close_relative_eq(a: f64, b: f64) -> bool {
        let absolute_value = partial_min(a.abs(), b.abs()).unwrap();
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
        (a - b).abs() < eps
    }

    use super::NumberLiteral;

    #[quickcheck]
    fn parses_integers(input: i64) {
        let res: NumberLiteral = input.to_string().parse().unwrap();
        assert_eq_f64(input as f64, res);
    }

    #[quickcheck]
    fn parses_with_dot(input: i64) {
        let res: NumberLiteral = format!("{}.", input).parse().unwrap();
        assert_eq_f64(input as f64, res);
    }

    #[quickcheck]
    fn parses_with_dot_before(input: u64) {
        let res: NumberLiteral = format!(".{}", input).parse().unwrap();
        let expected: f64 = format!("0.{}", input).parse().unwrap();
        assert_eq_f64(expected, res);

        let res: NumberLiteral = format!("-.{}", input).parse().unwrap();
        let expected: f64 = format!("-0.{}", input).parse().unwrap();
        assert_eq_f64(expected, res);
    }

    // Shrinking algorithm is recursive and on parses_floats and parses_floats_with_exponents
    // I'm blowing my stack on tests. Yeah, this is worse for testing, but at least tests
    // finish without getting stack overflow.
    #[quickcheck]
    fn parses_floats(input: NonShrinkable<Finite<f64>>) {
        let input = **input;
        let res: NumberLiteral = input.to_string().parse().unwrap();
        assert_eq_f64(input, res);
    }

    #[quickcheck]
    fn parses_integers_with_exponent(input: i64, exponent: u8) -> TestResult {
        // TODO: fix overflows with exponents and/or long numbers
        if exponent > 16 {
            return TestResult::discard();
        }

        let res: NumberLiteral = format!("{}e{}", input, exponent).parse().unwrap();
        let power = pow(10f64, exponent as usize);
        assert_eq_f64(input as f64 * power, res);

        if exponent == 0 {
            return TestResult::passed();
        }

        let res: NumberLiteral = format!("{}e-{}", input, exponent).parse().unwrap();
        let expected = input as f64 / pow(10f64, exponent as usize);
        assert_eq_f64(expected, res);

        TestResult::passed()
    }

    #[quickcheck]
    fn parses_integers_with_negative_zero_exponent(input: i64) {
        let res: NumberLiteral = format!("{}e-0", input).parse().unwrap();
        assert_eq_f64(input as f64, res);
    }

    #[quickcheck]
    fn parses_with_dot_with_exponent(input: i64, exponent: u8) -> TestResult {
        if exponent > 16 {
            return TestResult::discard();
        }

        let res: NumberLiteral = format!("{}.e{}", input, exponent).parse().unwrap();
        let expected = input as f64 * pow(10f64, exponent as usize);
        assert_eq_f64(expected as f64, res);

        let res: NumberLiteral = format!("{}.e-{}", input, exponent).parse().unwrap();
        let expected = input as f64 / pow(10f64, exponent as usize);
        assert_eq_f64(expected, res);

        TestResult::passed()
    }

    #[quickcheck]
    fn parses_with_dot_before_with_exponent(input: u64, exponent: u8) -> TestResult {
        if exponent > 16 {
            return TestResult::discard();
        }

        let base_number: f64 = format!("0.{}", input).parse().unwrap();

        let res: NumberLiteral = format!(".{}e{}", input, exponent).parse().unwrap();
        assert_eq_f64_with_exponent(base_number, res, exponent);

        let res: NumberLiteral = format!("-.{}e{}", input, exponent).parse().unwrap();
        assert_eq_f64_with_exponent(-base_number, res, exponent);

        let res: NumberLiteral = format!(".{}e-{}", input, exponent).parse().unwrap();
        let expected = base_number / pow(10f64, exponent as usize);
        // TODO: reverse exponent
        assert_eq_f64(expected, res);

        let res: NumberLiteral = format!("-.{}e-{}", input, exponent).parse().unwrap();
        let expected: f64 = -base_number / pow(10f64, exponent as usize);
        // TODO: reverse exponent
        assert_eq_f64(expected, res);

        TestResult::passed()
    }

    // Shrinking algorithm is recursive and on parses_floats and parses_floats_with_exponents
    // I'm blowing my stack on tests. Yeah, this is worse for testing, but at least tests
    // finish without getting stack overflow.
    #[quickcheck]
    fn parses_floats_with_exponent(input: NonShrinkable<Finite<f64>>, exponent: u8) -> TestResult {
        let input = **input;
        if exponent > 16 {
            return TestResult::discard();
        }

        let str = format!("{}e{}", input, exponent);
        let res: NumberLiteral = str.parse().unwrap();
        assert_eq_f64_with_exponent(input, res, exponent);

        TestResult::passed()
    }

    fn assert_eq_f64(expected: f64, got: NumberLiteral) {
        assert!(
            close_relative_eq(got.0, expected),
            "Expected {:?}, got {:?}",
            NumberLiteral(expected),
            got
        );
    }

    fn assert_eq_f64_with_exponent(expected: f64, got: NumberLiteral, exponent: u8) {
        let decade = pow(10f64, exponent as usize);
        assert!(
            close_relative_eq(got.0, expected * decade),
            "Expected {:?}, got {:?}",
            NumberLiteral(expected * decade),
            got
        );
    }
}
