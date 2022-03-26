#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgumentRegisterID(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalRegisterID(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalCellID(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringID(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JmpLabel(pub u16);
