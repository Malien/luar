macro_rules! wrap {
    ($name: ident, $type: ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub $type);

        impl TryFrom<usize> for $name {
            type Error = <$type as TryFrom<usize>>::Error;

            fn try_from(v: usize) -> Result<Self, Self::Error> {
                <$type>::try_from(v).map($name)
            }
        }

        impl From<$name> for usize {
            fn from($name(value): $name) -> Self {
                value as usize
            }
        }
    };
}

wrap!(ArgumentRegisterID, u16);
wrap!(LocalRegisterID, u16);
wrap!(GlobalCellID, u32);
wrap!(StringID, u16);
wrap!(JmpLabel, u16);
wrap!(BlockID, u32);
wrap!(LocalBlockID, u16);
wrap!(ModuleID, u32);
wrap!(SimpleBlockID, u16);
