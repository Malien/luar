use crate::ReturnRepresentable;

pub trait FFIFunc<Args> {
    // type Args: FromArgs;
    type Output: ReturnRepresentable;
    fn call(&self, args: Args) -> Self::Output;
}

macro_rules! ffi_func_impl {
    ($([$($arg:tt),*]);*$(;)?) => {
        $(
            impl<Func, $($arg,)* Ret> FFIFunc<( $($arg,)* )> for Func
            where 
                Func: Fn($($arg),*) -> Ret,
                Ret: ReturnRepresentable,
            {
                // type Args = ( $($arg,)* );
                type Output = Ret;
                #[allow(non_snake_case)]
                fn call(&self, ( $($arg,)* ): ( $($arg,)* )) -> Ret {
                    (self)( $($arg),* )
                }
            }
        )*
    }
}

ffi_func_impl! {
    [];
    [A0];
    [A0, A1];
    [A0, A1, A2];
}

