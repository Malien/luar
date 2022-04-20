macro_rules! wrap {
    ($name: ident, $type: ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(pub $type);
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
