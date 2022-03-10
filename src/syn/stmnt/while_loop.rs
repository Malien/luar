use crate::{
    fmt_tokens,
    lex::{DynTokens, ToTokenStream, Token},
    syn::{expr::Expression, Block},
};

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    pub condition: Expression,
    pub body: Block,
}

impl ToTokenStream for WhileLoop {
    type Tokens = DynTokens;
    fn to_tokens(self) -> Self::Tokens {
        let Self { condition, body } = self;
        Box::new(
            std::iter::once(Token::While)
                .chain(condition.to_tokens())
                .chain(std::iter::once(Token::Do))
                .chain(body.to_tokens())
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
        syn::{expr::Expression, unspanned_lua_token_parser, Block, Declaration, Statement},
        util::NonEmptyVec, input_parsing_expectation,
    };

    impl Arbitrary for WhileLoop {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self {
                condition: Arbitrary::arbitrary(g),
                body: Arbitrary::arbitrary(g),
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

    input_parsing_expectation!(
        while_loop,
        parses_empty_loop,
        "while 1 do end",
        WhileLoop {
            condition: Expression::Number(NumberLiteral(1f64)),
            body: Block::default(),
        }
    );

    input_parsing_expectation!(
        while_loop,
        parses_single_statement_body,
        "while 1 do 
            local foo = 42 
        end",
        WhileLoop {
            condition: Expression::Number(NumberLiteral(1f64)),
            body: Block {
                statements: vec![Statement::LocalDeclaration(Declaration {
                    names: NonEmptyVec::of_single(Ident::new("foo")),
                    initial_values: vec![Expression::Number(NumberLiteral(42f64))]
                })],
                ret: None
            }
        }
    );

    input_parsing_expectation!(
        while_loop,
        parses_multiple_statement_body,
        "while 1 do
            local foo = 42
            local bar = 69
        end",
        WhileLoop {
            condition: Expression::Number(NumberLiteral(1f64)),
            body: Block {
                statements: vec![
                    Statement::LocalDeclaration(Declaration {
                        names: NonEmptyVec::of_single(Ident::new("foo")),
                        initial_values: vec![Expression::Number(NumberLiteral(42f64))]
                    }),
                    Statement::LocalDeclaration(Declaration {
                        names: NonEmptyVec::of_single(Ident::new("bar")),
                        initial_values: vec![Expression::Number(NumberLiteral(69f64))]
                    }),
                ],
                ret: None
            }
        }
    );

    #[test]
    fn while_loop_without_condition_is_illegal() {
        let tokens = [Token::While, Token::Do, Token::End]; // while do end
        let res = unspanned_lua_token_parser::while_loop(tokens);
        assert!(res.is_err());
    }

    #[quickcheck]
    fn parses_empty_statement_body_with_arbitrary_condition(condition: Expression) {
        let while_loop = WhileLoop {
            condition,
            body: Block::default(),
        };
        let tokens: Vec<_> = while_loop.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::while_loop(tokens).unwrap();
        assert_eq!(while_loop, parsed);
    }

    #[quickcheck]
    fn parses_simple_loop_with_arbitrary_body(body: Block) {
        let while_loop = WhileLoop {
            condition: Expression::Nil,
            body,
        };
        let tokens: Vec<_> = while_loop.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::while_loop(tokens).unwrap();
        assert_eq!(while_loop, parsed);
    }

    #[quickcheck]
    fn parses_arbitrary_while_loop(while_loop: WhileLoop) {
        let tokens: Vec<_> = while_loop.clone().to_tokens().collect();
        let parsed = unspanned_lua_token_parser::while_loop(tokens).unwrap();
        assert_eq!(while_loop, parsed);
    }
}
