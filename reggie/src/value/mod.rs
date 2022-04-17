#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    Nil,
    Int(i32),
    Float(f64),
    String(String),
}

impl Default for LuaValue {
    fn default() -> Self {
        Self::Nil
    }
}

impl LuaValue {
    pub fn string(string: impl Into<String>) -> Self {
        Self::String(string.into())
    }

    pub fn coerce_to_f64(&self) -> Option<f64> {
        match self {
            Self::Int(int) => Some(*int as f64),
            Self::Float(float) => Some(*float),
            Self::String(str) => str.parse().ok(),
            _ => None,
        }
    }

    pub fn true_value() -> Self {
        Self::Int(1)
    }

    pub fn false_value() -> Self {
        Self::Nil
    }

    pub fn from_bool(v: bool) -> Self {
        if v {
            Self::true_value()
        } else {
            Self::false_value()
        }
    }

    pub fn total_eq(&self, other: &LuaValue) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Int(lhs), Self::Int(rhs)) => lhs == rhs,
            (Self::Float(lhs), Self::Float(rhs)) => eq_with_nan(*lhs, *rhs),
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

fn eq_with_nan(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        true
    } else if a.is_infinite() && b.is_infinite() {
        a.is_sign_negative() == b.is_sign_negative()
    } else {
        a == b
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for LuaValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use test_util::with_thread_gen;

        match u8::arbitrary(g) % 4 {
            0 => LuaValue::Nil,
            1 => LuaValue::Int(with_thread_gen(i32::arbitrary)),
            2 => LuaValue::Float(with_thread_gen(f64::arbitrary)),
            3 => LuaValue::String(with_thread_gen(String::arbitrary)),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            LuaValue::Nil => quickcheck::empty_shrinker(),
            LuaValue::Int(int) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(int.shrink().map(LuaValue::Int)))
            }
            LuaValue::Float(float) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(float.shrink().map(LuaValue::Float)))
            }
            LuaValue::String(str) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(str.shrink().map(LuaValue::String)))
            }
        }
    }
}
