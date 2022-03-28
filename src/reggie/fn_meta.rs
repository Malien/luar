#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LocalRegCount {
    pub f: u16,
    pub i: u16,
    pub s: u16,
    pub t: u16,
    pub c: u16,
    pub d: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnID(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReturnCount {
    Known(usize),
    Unknown
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnMeta {
    // pub identity: FnID,
    // pub source: syn::FunctionDeclaration,
    pub arg_count: usize,
    pub local_count: LocalRegCount,
    pub return_count: ReturnCount,
    pub label_mappings: Vec<usize>,
    pub const_strings: Vec<String>,
    // pub global_deps: 
}