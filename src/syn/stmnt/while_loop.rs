use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream, Token},
    syn::expr::Expression,
};

use super::Statement;

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

impl ToTokenStream for WhileLoop {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        let Self { condition, body } = self;
        Box::new(
            std::iter::once(Token::While)
                .chain(condition.to_tokens())
                .chain(std::iter::once(Token::Do))
                .chain(body.into_iter().flat_map(ToTokenStream::to_tokens))
                .chain(std::iter::once(Token::End)),
        )
    }
}

fmt_tokens!(WhileLoop);

#[cfg(test)]
mod test {
    use quickcheck::Arbitrary;

    use super::WhileLoop;
    use crate::{
        lex::{Ident, NumberLiteral, ToTokenStream, Token},
        syn::{expr::Expression, lua_parser, Declaration, Statement},
        test_util::GenExt,
        util::NonEmptyVec,
    };

    impl Arbitrary for WhileLoop {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            if g.size() <= 1 {
                return Self {
                    condition: Expression::Nil,
                    body: vec![],
                };
            }
            let mut inner_gen = g.next_iter();
            let body = std::iter::repeat_with(|| Arbitrary::arbitrary(&mut inner_gen))
                .take(g.size())
                .collect();
            Self {
                condition: Arbitrary::arbitrary(g),
                body
            }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let condition = self.condition.clone();
            let body = self.body.clone();
            Box::new(
                self.condition
                    .shrink()
                    .map(move |condition| Self {
                        condition,
                        body: body.clone(),
                    })
                    .chain(self.body.shrink().map(move |body| Self {
                        condition: condition.clone(),
                        body,
                    })),
            )
        }
    }

    macro_rules! input_parsing_expectation {
        ($name: tt, $input: expr, $expected: expr) => {
            #[test]
            fn $name() {
                use logos::Logos;
                let tokens: Vec<_> = crate::lex::Token::lexer($input).collect();
                let parsed = crate::syn::lua_parser::while_loop(&tokens).unwrap();
                assert_eq!(parsed, $expected)
            }
        };
    }

    input_parsing_expectation!(
        parses_empty_loop,
        "while 1 do end",
        WhileLoop {
            condition: Expression::Number(NumberLiteral(1f64)),
            body: vec![]
        }
    );

    input_parsing_expectation!(
        parses_single_statement_body,
        "while 1 do 
            local foo = 42 
        end",
        WhileLoop {
            condition: Expression::Number(NumberLiteral(1f64)),
            body: vec![Statement::LocalDeclaration(Declaration {
                names: NonEmptyVec::of_single(unsafe { Ident::new("foo") }),
                initial_values: vec![Expression::Number(NumberLiteral(42f64))]
            })]
        }
    );

    input_parsing_expectation!(
        parses_multiple_statement_body,
        "while 1 do
            local foo = 42
            local bar = 69
        end",
        WhileLoop {
            condition: Expression::Number(NumberLiteral(1f64)),
            body: vec![
                Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(unsafe { Ident::new("foo") }),
                    initial_values: vec![Expression::Number(NumberLiteral(42f64))]
                }),
                Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(unsafe { Ident::new("bar") }),
                    initial_values: vec![Expression::Number(NumberLiteral(69f64))]
                }),
            ]
        }
    );

    #[test]
    fn while_loop_without_condition_is_illegal() {
        let tokens = [Token::While, Token::Do, Token::End]; // while do end
        let res = lua_parser::while_loop(&tokens);
        assert!(res.is_err());
    }

    #[quickcheck]
    fn parses_empty_statement_body_with_arbitrary_condition(condition: Expression) {
        let while_loop = WhileLoop {
            condition,
            body: vec![],
        };
        let tokens: Vec<_> = while_loop.clone().to_tokens().collect();
        let parsed = lua_parser::while_loop(&tokens).unwrap();
        assert_eq!(while_loop, parsed);
    }

    #[quickcheck]
    fn parses_simple_loop_with_arbitrary_body(body: Vec<Statement>) {
        let while_loop = WhileLoop {
            condition: Expression::Nil,
            body,
        };
        let tokens: Vec<_> = while_loop.clone().to_tokens().collect();
        let parsed = lua_parser::while_loop(&tokens).unwrap();
        assert_eq!(while_loop, parsed);
    }

    #[quickcheck]
    fn parses_arbitrary_while_loop(while_loop: WhileLoop) {
        let tokens: Vec<_> = while_loop.clone().to_tokens().collect();
        let parsed = lua_parser::while_loop(&tokens).unwrap();
        assert_eq!(while_loop, parsed);
    }
}
