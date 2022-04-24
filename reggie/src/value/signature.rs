#[derive(Debug)]
pub enum FunctionSignatureList {
    Finite(Vec<ArgumentType>),
    Unspecified,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ArgumentType {
    Dynamic,
    Int,
    Float,
    String,
}

impl ArgumentType {
    pub fn compatible_with(self, other: ArgumentType) -> bool {
        use ArgumentType::*;

        match (self, other) {
            (Dynamic, _) => true,
            (a, b) => a == b
        }
    }
}

// impl FunctionSignatureList {
//     pub fn compatible_with(&self, args: &[ArgumentType]) -> bool {
//         match self {
//             FunctionSignatureList::Unspecified => true,
//             FunctionSignatureList::Finite(self_args) => {

//             }
//         }
//     }
// }
