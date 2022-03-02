use std::{collections::HashMap, rc::Rc};

use super::{LuaValue, LuaKeyValue};

#[derive(Debug, Clone, Default)]
pub struct TableValue(HashMap<LuaKeyValue, LuaValue>);

pub struct TableRef(Rc<TableValue>);

impl TableValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn get(&self, key: &LuaKeyValue) -> &LuaValue {
        self.0.get(key).unwrap_or(&LuaValue::Nil)
    }

    pub fn set(&mut self, key: LuaKeyValue, value: LuaValue) {
        self.0.insert(key, value);
    }
}

impl std::ops::Index<&LuaKeyValue> for TableValue {
    type Output = LuaValue;

    fn index(&self, index: &LuaKeyValue) -> &Self::Output {
        self.get(index)
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use crate::lang::{LuaFunction, LuaNumber, LuaValue, ReturnValue};

    use super::{TableValue, LuaKeyValue};

    #[test]
    fn new_table_is_empty() {
        assert!(TableValue::new().empty());
    }

    #[test]
    fn default_table_is_empty() {
        let table: TableValue = Default::default();
        assert!(table.empty());
    }

    #[quickcheck]
    fn setting_number_key_can_be_retrieved_with_the_same_key(num: LuaNumber) -> TestResult {
        if num.as_f64().is_nan() {
            return TestResult::discard();
        }
        let num = LuaKeyValue::Number(num);
        let mut table = TableValue::new();
        table.set(num.clone(), LuaValue::number(1));
        assert_eq!(table[&num], LuaValue::number(1));
        TestResult::passed()
    }

    #[quickcheck]
    fn setting_string_key_can_be_retrieved_with_the_same_key(str: String) {
        let key = LuaKeyValue::String(str);
        let mut table = TableValue::new();
        table.set(key.clone(), LuaValue::number(1));
        assert_eq!(table.get(&key), &LuaValue::number(1));
    }

    #[test]
    fn setting_function_key_can_be_retrieved_only_with_the_same_fn_ref() {
        let mut table = TableValue::new();
        let func = LuaFunction::new(|_, _| Ok(ReturnValue::Nil));
        let func = LuaKeyValue::Function(func);
        table.set(func.clone(), LuaValue::number(42));
        assert_eq!(table.get(&func), &LuaValue::number(42));
        let func2 = LuaFunction::new(|_, _| Ok(ReturnValue::Nil));
        let func2 = LuaKeyValue::Function(func2);
        assert_eq!(table.get(&func2), &LuaValue::Nil);
    }
}
