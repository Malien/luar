use std::{collections::HashMap, hash::Hash, rc::Rc};

use super::{LuaKey, LuaValue};

#[derive(Debug, Clone, Default)]
pub struct TableValue(HashMap<LuaKey, LuaValue>);

#[derive(Debug, Clone, Default)]
pub struct TableRef(Rc<TableValue>);

impl TableRef {
    pub fn addr(&self) -> *const TableValue {
        Rc::as_ptr(&self.0)
    }
}

impl PartialEq for TableRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TableRef {}

impl Hash for TableRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state)
    }
}

impl From<TableValue> for TableRef {
    fn from(table: TableValue) -> Self {
        Self(Rc::new(table))
    }
}

impl TableValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn get(&self, key: &LuaKey) -> &LuaValue {
        self.0.get(key).unwrap_or(&LuaValue::Nil)
    }

    pub fn set(&mut self, key: LuaKey, value: LuaValue) {
        self.0.insert(key, value);
    }
}

impl std::ops::Index<&LuaKey> for TableValue {
    type Output = LuaValue;

    fn index(&self, index: &LuaKey) -> &Self::Output {
        self.get(index)
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use quickcheck::{Arbitrary, TestResult};

    use crate::lang::{LuaFunction, LuaNumber, LuaValue, ReturnValue};

    use super::{LuaKey, TableRef, TableValue};

    impl Arbitrary for TableValue {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            TableValue(Arbitrary::arbitrary(g))
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(self.0.shrink().map(TableValue))
        }
    }

    impl Arbitrary for TableRef {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            TableRef::from(TableValue::arbitrary(g))
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(self.0.shrink().map(Rc::new).map(TableRef))
        }
    }

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
    fn setting_number_key_can_be_retrieved_with_the_same_key(
        num: LuaNumber,
        value: LuaValue,
    ) -> TestResult {
        if num.as_f64().is_nan() {
            return TestResult::discard();
        }
        let num = LuaKey::Number(num);
        let mut table = TableValue::new();
        table.set(num.clone(), value.clone());
        assert!(table.get(&num).total_eq(&value));
        TestResult::passed()
    }

    #[quickcheck]
    fn setting_string_key_can_be_retrieved_with_the_same_key(str: String, value: LuaValue) {
        let key = LuaKey::String(str);
        let mut table = TableValue::new();
        table.set(key.clone(), value.clone());
        assert!(table.get(&key).total_eq(&value));
    }

    #[quickcheck]
    fn setting_function_key_can_be_retrieved_only_with_the_same_fn_ref(value: LuaValue) {
        let mut table = TableValue::new();
        let func = LuaFunction::new(|_, _| Ok(ReturnValue::Nil));
        let func = LuaKey::Function(func);
        table.set(func.clone(), value.clone());
        assert!(table.get(&func).total_eq(&value));
        let func2 = LuaFunction::new(|_, _| Ok(ReturnValue::Nil));
        let func2 = LuaKey::Function(func2);
        assert_eq!(table.get(&func2), &LuaValue::Nil);
    }

    #[quickcheck]
    fn setting_table_key_can_be_retrieved_only_with_the_same_table_ref(value: LuaValue) {
        let mut table = TableValue::new();
        let key_table = TableRef::from(TableValue::new());
        let key_table = LuaKey::Table(key_table);
        table.set(key_table.clone(), value.clone());
        assert!(table.get(&key_table).total_eq(&value));
        let key_table2 = TableRef::from(TableValue::new());
        let key_table2 = LuaKey::Table(key_table2);
        assert_eq!(table.get(&key_table2), &LuaValue::Nil);
    }
}
