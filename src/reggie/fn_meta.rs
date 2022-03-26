#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LocalRegCount {
    pub f: usize,
    pub i: usize,
    pub s: usize,
    pub t: usize,
    pub c: usize,
    pub d: usize,
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