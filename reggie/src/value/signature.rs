#[derive(Debug)]
pub enum FunctionSignatureList {
    Finite(Vec<ArgumentType>),
    Unspecified,
}

#[derive(Debug)]
pub enum ArgumentType {
    Dynamic,
    Int,
    Float,
    String,
}
