use luar_lex::{Ident, NumberLiteral, StringLiteral};
use luar_syn::{BinaryOperator, UnaryOperator};
use non_empty::NonEmptyVec;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Module {
    pub chunks: Vec<Chunk>,
    pub ret: Option<Return>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Chunk {
    FnDecl(FunctionDeclaration),
    Statement(Statement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: FunctionName,
    pub args: Vec<LocalValueID>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionName {
    Plain(Var),
    Method(Var, Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub ret: Option<Return>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    LocalDeclaration(Declaration),
    While(WhileLoop),
    Repeat(RepeatLoop),
    If(Conditional),
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Return(pub Vec<Expression>);

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub names: NonEmptyVec<Var>,
    pub values: NonEmptyVec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub names: NonEmptyVec<LocalValueID>,
    pub initial_values: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepeatLoop {
    pub body: Block,
    pub condition: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    pub condition: Expression,
    pub body: Block,
    pub tail: ConditionalTail,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalTail {
    End,
    Else(Block),
    ElseIf(Box<Conditional>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCall {
    Method {
        func: Var,
        method: Ident,
        args: FunctionCallArgs,
    },
    Function {
        func: Var,
        args: FunctionCallArgs,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCallArgs {
    Table(TableConstructor),
    Arglist(Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableConstructor {
    pub lfield: Vec<Expression>,
    pub ffield: Vec<(Ident, Expression)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Nil,
    String(StringLiteral),
    Number(NumberLiteral),
    Variable(Var),
    BinaryOperator {
        lhs: Box<Expression>,
        op: BinaryOperator,
        rhs: Box<Expression>,
    },
    UnaryOperator {
        op: UnaryOperator,
        exp: Box<Expression>,
    },
    TableConstructor(TableConstructor),
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalValueID(pub u16);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlobalValueID(pub u16);

impl TryFrom<usize> for GlobalValueID {
    type Error = <u16 as TryFrom<usize>>::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u16::try_from(value).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueID {
    Local(LocalValueID),
    Global(GlobalValueID),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Var {
    Named(ValueID),
    PropertyAccess {
        from: Box<Var>,
        property: Ident,
    },
    MemberLookup {
        from: Box<Var>,
        value: Box<Expression>,
    },
}

