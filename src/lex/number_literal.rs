use num::pow;
use std::str::{Chars, FromStr};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NumberLiteral {
    Integer(i64),
    Floating(f64),
}

#[derive(Debug, Error)]
#[error("Invalid number format")]
pub struct NumberLiteralParseError;

impl FromStr for NumberLiteral {
    type Err = NumberLiteralParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let mut numerator = 0i64;
        let mut sign = 1;
        match chars.next().ok_or(NumberLiteralParseError)? {
            '-' => {
                sign = -1;
            }
            '+' => {}
            '.' => match parse_denumerator(&mut chars)? {
                DenumeratorParsings::Denum(denum) => {
                    return Ok(NumberLiteral::Floating(sign as f64 * denum))
                }
                DenumeratorParsings::DenumWithExponent(denum, exponent) => {
                    let res = if exponent >= 0 {
                        denum * pow(10f64, exponent as usize)
                    } else {
                        denum / pow(10f64, (-exponent) as usize)
                    };
                    return Ok(NumberLiteral::Floating(sign as f64 * res));
                }
            },
            digit_char => {
                if let Some(digit) = digit_char.to_digit(10) {
                    numerator += digit as i64;
                } else {
                    return Err(NumberLiteralParseError);
                }
            }
        };

        loop {
            match chars.next() {
                Some('e') => {
                    let exponent = parse_exponent(&mut chars)?;
                    return Ok(if exponent >= 0 {
                        NumberLiteral::Integer(sign * numerator * pow(10, exponent as usize))
                    } else {
                        NumberLiteral::Floating(
                            ((sign * numerator) as f64) / pow(10f64, (-exponent) as usize),
                        )
                    });
                }
                Some('.') => match parse_denumerator(&mut chars)? {
                    DenumeratorParsings::Denum(denum) => {
                        return Ok(NumberLiteral::Floating(
                            sign as f64 * (numerator as f64 + denum),
                        ))
                    }
                    DenumeratorParsings::DenumWithExponent(denum, exponent) => {
                        let res = if exponent > 0 {
                            (numerator as f64 + denum) * pow(10f64, exponent as usize)
                        } else {
                            (numerator as f64 + denum) / pow(10f64, (-exponent) as usize)
                        };
                        return Ok(NumberLiteral::Floating(sign as f64 * res));
                    }
                },
                Some(digit_char) => {
                    if let Some(digit) = digit_char.to_digit(10) {
                        numerator *= 10;
                        numerator += digit as i64;
                    } else {
                        return Err(NumberLiteralParseError);
                    }
                }
                None => return Ok(NumberLiteral::Integer(numerator * sign)),
            }
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

#[cfg(test)]
mod tests {
    use num::{abs, pow};
    use quickcheck::TestResult;

    use super::NumberLiteral;

    #[quickcheck]
    fn parses_integers(input: i64) {
        let res: NumberLiteral = input.to_string().parse().unwrap();
        assert_eq!(NumberLiteral::Integer(input), res);
    }

    #[quickcheck]
    fn parses_with_dot(input: i64) {
        let res: NumberLiteral = format!("{}.", input).parse().unwrap();
        assert_eq!(NumberLiteral::Floating(input as f64), res);
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

    #[quickcheck]
    fn parses_floats(input: f64) {
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
        let expected = input * pow(10, exponent as usize);
        assert_eq!(NumberLiteral::Integer(expected), res);

        if exponent == 0 {
            return TestResult::passed()
        }

        let res: NumberLiteral = format!("{}e-{}", input, exponent).parse().unwrap();
        let expected = input as f64 / pow(10f64, exponent as usize);
        assert_eq_f64(expected, res);

        TestResult::passed()
    }

    #[quickcheck]
    fn parses_integers_with_negative_zero_exponent(input: i64) {
        let res: NumberLiteral = format!("{}e-0", input).parse().unwrap();
        assert_eq!(NumberLiteral::Integer(input), res);
    }

    #[quickcheck]
    fn parses_with_dot_with_exponent(input: i64, exponent: u8) -> TestResult {
        if exponent > 16 {
            return TestResult::discard();
        }

        let res: NumberLiteral = format!("{}.e{}", input, exponent).parse().unwrap();
        let expected = input * pow(10, exponent as usize);
        assert_eq!(NumberLiteral::Floating(expected as f64), res);

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

    #[quickcheck]
    fn parses_floats_with_exponent(input: f64, exponent: u8) -> TestResult {
        if exponent > 16 {
            return TestResult::discard();
        }

        let res: NumberLiteral = format!("{}e{}", input, exponent).parse().unwrap();
        assert_eq_f64_with_exponent(input, res, exponent);

        TestResult::passed()
    }

    fn assert_eq_f64(expected: f64, got: NumberLiteral) {
        if let NumberLiteral::Floating(got) = got {
            assert!(
                close_cmp(got, expected, 0.0000001),
                "Expected {:?}, got {:?}",
                NumberLiteral::Floating(expected),
                got
            );
        } else {
            panic!("Expected {:?} to be NumberLiteral::Floating", got);
        }
    }

    fn assert_eq_f64_with_exponent(expected: f64, got: NumberLiteral, exponent: u8) {
        if let NumberLiteral::Floating(got) = got {
            let decade = pow(10f64, exponent as usize);
            assert!(
                close_cmp(got, expected * decade, decade * 0.0000001),
                "Expected {:?}, got {:?}",
                NumberLiteral::Floating(expected),
                got
            );
        } else {
            panic!("Expected {:?} to be NumberLiteral::Floating", got);
        }
    }

    fn close_cmp(a: f64, b: f64, eps: f64) -> bool {
        abs(a - b) < eps
    }
}
