use std::{
    fmt::Display,
    iter::{Chain, Flatten, Once},
};

use crate::{
    lex::{
        format::{format_single_token, format_tokens, FormattingStyle, IndentationChange},
        DynTokens, Ident, ToTokenStream, Token,
    },
    util::FlatIntersperseExt,
};

use super::{expr::Var, Block};

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionName {
    Plain(Var),
    Method(Var, Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: FunctionName,
    pub args: Vec<Ident>,
    pub body: Block,
}

fn header_tokens(name: FunctionName, args: Vec<Ident>) -> impl Iterator<Item = Token> {
    use std::iter::once;
    use Token::*;

    once(Function)
        .chain(name.to_tokens())
        .chain(once(OpenRoundBracket))
        .chain(
            args.into_iter()
                .map(ToTokenStream::to_tokens)
                .flat_intersperse(Comma),
        )
        .chain(once(CloseRoundBracket))
}

impl ToTokenStream for FunctionDeclaration {
    // Once again. impl Tokens is ideal, but I can't statically type this. Too painful
    type Tokens = DynTokens;

    fn to_tokens(self) -> Self::Tokens {
        Box::new(
            header_tokens(self.name, self.args)
                .chain(self.body.to_tokens())
                .chain(std::iter::once(Token::End)),
        )
    }
}

impl ToTokenStream for FunctionName {
    type Tokens = Chain<
        <Var as ToTokenStream>::Tokens,
        Flatten<std::option::IntoIter<Chain<Once<Token>, Once<Token>>>>,
    >;

    fn to_tokens(self) -> Self::Tokens {
        let (var, method) = match self {
            FunctionName::Plain(var) => (var, None),
            FunctionName::Method(var, ident) => (var, Some(ident)),
        };
        var.to_tokens().chain(
            method
                .map(|method| std::iter::once(Token::Colon).chain(method.to_tokens()))
                .into_iter()
                .flatten(),
        )
    }
}

impl Display for FunctionDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { name, args, body } = self.clone();
        format_tokens(&mut header_tokens(name, args), f)?;

        let mut indent = 0;
        let mut current_format = FormattingStyle::Indent(IndentationChange::Increase);
        for token in body.to_tokens() {
            format_single_token(token, &mut indent, &mut current_format, f)?;
        }
        format_single_token(Token::End, &mut indent, &mut current_format, f)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use non_empty::NonEmptyVec;
    use quickcheck::{Arbitrary, Gen};

    use crate::{
        assert_parses, input_parsing_expectation,
        lex::Ident,
        syn::{
            expr::{op::BinaryOperator, Expression, Var},
            Block, Conditional, ConditionalTail, Declaration, FunctionName, Return, Statement,
        },
        
    };

    use super::FunctionDeclaration;

    impl Arbitrary for FunctionDeclaration {
        fn arbitrary(g: &mut Gen) -> Self {
            FunctionDeclaration {
                name: Arbitrary::arbitrary(g),
                args: Arbitrary::arbitrary(g),
                body: Arbitrary::arbitrary(g),
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(
                self.name
                    .shrink()
                    .map({
                        let args = self.args.clone();
                        let body = self.body.clone();
                        move |name| Self {
                            name,
                            args: args.clone(),
                            body: body.clone(),
                        }
                    })
                    .chain(self.args.shrink().map({
                        let name = self.name.clone();
                        let body = self.body.clone();
                        move |args| Self {
                            name: name.clone(),
                            args,
                            body: body.clone(),
                        }
                    }))
                    .chain(self.body.shrink().map({
                        let name = self.name.clone();
                        let args = self.args.clone();
                        move |body| Self {
                            name: name.clone(),
                            args: args.clone(),
                            body,
                        }
                    })),
            )
        }
    }

    impl Arbitrary for FunctionName {
        fn arbitrary(g: &mut Gen) -> Self {
            if bool::arbitrary(g) {
                FunctionName::Plain(Var::arbitrary(g))
            } else {
                FunctionName::Method(Var::arbitrary(g), Ident::arbitrary(g))
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match self {
                FunctionName::Plain(var) => Box::new(var.shrink().map(FunctionName::Plain)),
                FunctionName::Method(var, name) => Box::new(
                    std::iter::once(FunctionName::Plain(var.clone()))
                        .chain(var.shrink().map({
                            let name = name.clone();
                            move |var| FunctionName::Method(var, name.clone())
                        }))
                        .chain(name.shrink().map({
                            let var = var.clone();
                            move |name| FunctionName::Method(var.clone(), name)
                        })),
                ),
            }
        }
    }

    macro_rules! empty_decl {
        () => {
            FunctionDeclaration {
                args: vec![],
                body: Block::default(),
                name: FunctionName::Plain(Var::Named(Ident::new("foo"))),
            }
        };
    }

    macro_rules! complex_decl {
        () => {
            FunctionDeclaration {
                name: FunctionName::Method(
                    Var::PropertyAccess {
                        from: Box::new(Var::Named(Ident::new("foo"))),
                        property: Ident::new("bar"),
                    },
                    Ident::new("baz"),
                ),
                args: vec![Ident::new("self"), Ident::new("x"), Ident::new("y")],
                body: Block {
                    statements: vec![Statement::If(Conditional {
                        condition: Expression::Variable(Var::PropertyAccess {
                            from: Box::new(Var::Named(Ident::new("self"))),
                            property: Ident::new("condition"),
                        }),
                        body: Block {
                            statements: vec![],
                            ret: Some(Return::single(Expression::BinaryOperator {
                                lhs: Box::new(Expression::Variable(Var::Named(Ident::new("x")))),
                                op: BinaryOperator::Minus,
                                rhs: Box::new(Expression::Variable(Var::Named(Ident::new("y")))),
                            })),
                        },
                        tail: ConditionalTail::End,
                    })],
                    ret: Some(Return::single(Expression::BinaryOperator {
                        lhs: Box::new(Expression::Variable(Var::Named(Ident::new("x")))),
                        op: BinaryOperator::Plus,
                        rhs: Box::new(Expression::Variable(Var::Named(Ident::new("y")))),
                    })),
                },
            }
        };
    }

    #[test]
    fn displays_correctly_empty_func() {
        assert_eq!(format!("{}", empty_decl!()), "function foo()\nend");
    }

    #[test]
    fn displays_correctly() {
        assert_eq!(
            format!("{}", complex_decl!()).replace("\t", "    "),
            indoc! {"
                function foo.bar:baz(self, x, y)
                    if self.condition then
                        return (x - y)
                    end
                    return (x + y)
                end"}
        )
    }

    fn parses(decl: FunctionDeclaration) {
        assert_parses!(function_declaration, decl);
    }

    #[test]
    fn parses_empty_decl() {
        parses(empty_decl!());
    }

    #[quickcheck]
    fn parses_arbitrary_plain_name(name: Var) {
        parses(FunctionDeclaration {
            name: FunctionName::Plain(name),
            args: vec![],
            body: Block::default(),
        });
    }

    #[quickcheck]
    fn parses_arbitrary_arglist(args: Vec<Ident>) {
        parses(FunctionDeclaration {
            name: FunctionName::Plain(Var::Named(Ident::new("foo"))),
            args,
            body: Block::default(),
        });
    }

    #[quickcheck]
    fn parses_arbitrary_method_name(name: Var, method: Ident) {
        parses(FunctionDeclaration {
            name: FunctionName::Method(name, method),
            args: vec![],
            body: Block::default(),
        });
    }

    #[test]
    fn parses_complex_fn() {
        parses(complex_decl!());
    }

    #[quickcheck]
    fn parses_arbitrary_body(body: Block) {
        parses(FunctionDeclaration {
            name: FunctionName::Plain(Var::Named(Ident::new("foo"))),
            args: vec![],
            body,
        });
    }

    input_parsing_expectation!(
        function_declaration,
        parses_fn_with_decl,
        "function foo()
            local x, y
        end",
        FunctionDeclaration {
            name: FunctionName::Plain(Var::Named(Ident::new("foo"))),
            args: vec![],
            body: Block {
                statements: vec![Statement::LocalDeclaration(Declaration {
                    names: unsafe {
                        NonEmptyVec::new_unchecked(vec![Ident::new("x"), Ident::new("y")])
                    },
                    initial_values: vec![]
                })],
                ret: None
            }
        }
    );

    #[quickcheck]
    fn parses_arbitrary_func_decl(expected: FunctionDeclaration) {
        parses(expected);
    }
}
