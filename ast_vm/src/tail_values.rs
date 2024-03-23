use crate::lang::{LuaValue, ReturnValue};

/// Expands multi-value lists into a single iterator of values.
pub struct TailValuesIter<I> {
    /// The iterator of return values.
    values: I,
    /// Held value to detect the last element of the multi-value iterator.
    hold: Option<ReturnValue>,
    /// Iterator over the last multi-value return.
    tail: Option<<ReturnValue as IntoIterator>::IntoIter>,
}

impl<I> Iterator for TailValuesIter<I>
where
    I: Iterator<Item = ReturnValue>,
{
    type Item = LuaValue;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.values.next(), &mut self.hold, &mut self.tail) {
                (Some(next), Some(hold), _) => {
                    return Some(std::mem::replace(hold, next).first_value())
                }
                (Some(next), hold, _) => *hold = Some(next),
                (None, _, Some(tail)) => return tail.next(),
                (None, hold, tail) => {
                    match hold.take() {
                        None => return None,
                        Some(ret) => *tail = Some(ret.into_iter()),
                    };
                }
            }
        }
    }
}

pub fn tail_values<I>(values: I) -> TailValuesIter<I::IntoIter>
where
    I: IntoIterator<Item = ReturnValue>,
{
    TailValuesIter {
        values: values.into_iter(),
        hold: None,
        tail: None,
    }
}

#[cfg(test)]
mod test {
    use smallvec::smallvec;

    use crate::lang::{LuaValue, ReturnValue};

    use super::tail_values;

    #[test]
    fn tail_values_empty() {
        let values = std::iter::empty();
        let mut tail_values = tail_values(values);
        assert!(tail_values.next().is_none());
    }

    #[test]
    fn tail_values_iter_simple_values() {
        let values = vec![
            LuaValue::Nil,
            LuaValue::number(42),
            LuaValue::String("hello".into()),
        ];
        let tail_values: Vec<_> =
            tail_values(values.iter().cloned().map(ReturnValue::from)).collect();
        assert_eq!(values, tail_values);
    }

    #[test]
    fn tail_values_expand_end_multi_value() {
        let multi_value = ReturnValue(smallvec![
            LuaValue::Nil,
            LuaValue::string("hello"),
            LuaValue::number(69),
        ]);
        let values = vec![ReturnValue::NIL, ReturnValue::number(42), multi_value];
        let tail_values: Vec<_> = tail_values(values).collect();
        let expected = vec![
            LuaValue::Nil,
            LuaValue::number(42),
            LuaValue::Nil,
            LuaValue::string("hello"),
            LuaValue::number(69),
        ];
        assert_eq!(tail_values, expected);
    }

    #[test]
    fn tail_values_does_not_expand_multi_value_in_the_middle() {
        let multi_value = ReturnValue(smallvec![
            LuaValue::Nil,
            LuaValue::string("hello"),
            LuaValue::number(69),
        ]);
        let values = vec![ReturnValue::NIL, multi_value, ReturnValue::number(42)];
        let expected: Vec<_> = values
            .iter()
            .cloned()
            .map(ReturnValue::first_value)
            .collect();
        let tail_values: Vec<_> = tail_values(values.clone()).collect();
        assert_eq!(tail_values, expected);
    }
}
