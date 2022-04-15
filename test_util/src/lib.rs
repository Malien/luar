use lazy_static::lazy_static;
use quickcheck::{Arbitrary, Gen};
use std::{
    cell::RefCell,
    env,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NonShrinkable<T>(pub T);

use luar_lex::Ident;

impl<T> Arbitrary for NonShrinkable<T>
where
    T: Arbitrary,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        NonShrinkable(T::arbitrary(g))
    }
}

macro_rules! deref_t {
    ($t:ident) => {
        impl<T> Deref for $t<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> DerefMut for $t<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

deref_t!(NonShrinkable);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Finite<T>(pub T);

impl<T> Arbitrary for Finite<T>
where
    T: num::Float + Arbitrary,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        loop {
            let val = T::arbitrary(g);
            if val.is_finite() {
                return Finite(val);
            }
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().filter(|v| v.is_finite()).map(|v| Finite(v)))
    }
}

deref_t!(Finite);

lazy_static! {
    pub static ref QUICKCHECK_RECURSIVE_DEPTH: usize = {
        let default = 4;
        match env::var("QUICKCHECK_RECURSIVE_DEPTH") {
            Ok(val) => val.parse().unwrap_or(default),
            Err(_) => default,
        }
    };
    pub static ref QC_GEN_SIZE: usize = {
        let default = 10;
        match env::var("QUICKCHECK_GENERATOR_SIZE") {
            Ok(val) => val.parse().unwrap_or(default),
            Err(_) => default,
        }
    };
}

thread_local! {
    pub static THREAD_GEN: RefCell<Gen> = RefCell::new(Gen::new(*QC_GEN_SIZE));
}

pub fn with_thread_gen<R>(func: impl FnOnce(&mut Gen) -> R) -> R {
    THREAD_GEN.with(|gen| func(&mut gen.borrow_mut()))
}

pub fn arbitrary_recursive_vec<T: Arbitrary>(gen: &mut Gen) -> Vec<T> {
    let size = with_thread_gen(|gen| usize::arbitrary(gen) % gen.size());
    (0..size + 1).map(|_| T::arbitrary(gen)).collect()
}

pub trait GenExt {
    fn next_iter(&self) -> Self;
}

impl GenExt for Gen {
    fn next_iter(&self) -> Self {
        Gen::new(std::cmp::min(*QUICKCHECK_RECURSIVE_DEPTH, self.size() - 1))
    }
}

pub fn vec_of_idents(len: usize, prefix: &str) -> Vec<Ident> {
    (0..len)
        .into_iter()
        .map(|i| format!("{}{}", prefix, i))
        .map(Ident::new)
        .collect()
}

#[macro_export]
macro_rules! run_lua_test {
    ($file: expr, $context: expr) => {{
        let mut context = $context;
        let existing_assert = crate::lang::GlobalContext::get(&context, "assert");
        if existing_assert.is_nil() {
            crate::lang::GlobalContext::set(
                &mut context,
                "assert",
                crate::lang::LuaValue::function(|_, args| {
                    crate::stdlib::fns::assert(args).map(crate::lang::ReturnValue::from)
                }),
            );
        }
        let test_module = crate::syn::lua_parser::module(include_str!($file))?;
        crate::ast_vm::eval_module(&test_module, &mut context)?;
        Ok(())
    }};
}
