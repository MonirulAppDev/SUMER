//! Parser implementation for the SUMER programming language.
//!
//! Converts a stream of [`Token`](sumer_lexer::Token)s produced by `sumer-lexer`
//! into an Abstract Syntax Tree ([`Program`](sumer_ast::Program)) defined in `sumer-ast`.

pub mod attribute;
pub mod cursor;
pub mod declaration;
pub mod error;
pub mod expression;
pub mod parser;
pub mod pattern;
pub mod statement;
pub mod types;

pub use cursor::TokenCursor;
pub use error::{ParseError, ParseErrorKind, ParseResult};
pub use expression::{parse_expression, parse_expression_with_precedence};
pub use parser::{Parser, parse};
pub use pattern::{parse_match_arm, parse_pattern};

#[cfg(test)]
mod tests {
    use super::*;
    use sumer_ast::*;
    use sumer_lexer::{Lexer, Token, TokenKind};
    use sumer_span::SourceId;

    /// Helper function to lex source code into tokens for parser testing.
    fn tokenize(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(SourceId::new(1), source);
        let mut tokens = Vec::new();
        loop {
            match lexer.next_token() {
                Ok(tok) => {
                    let is_eof = tok.kind() == &TokenKind::Eof;
                    tokens.push(tok);
                    if is_eof {
                        break;
                    }
                }
                Err(err) => panic!("lexing error in test input: {err}"),
            }
        }
        tokens
    }

    /// Helper function to parse an expression from a source string.
    fn parse_expr_str(source: &str) -> ParseResult<Expr> {
        let tokens = tokenize(source);
        let mut cursor = TokenCursor::new(&tokens);
        parse_expression(&mut cursor)
    }

    // =========================================================================
    // TASK 5 REGRESSION TESTS
    // =========================================================================

    #[test]
    fn test_1_simple_function() {
        let tokens = tokenize("fn main() {}");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "main");
                assert_eq!(f.visibility, Visibility::Private);
                assert!(!f.is_async);
                assert!(f.parameters.is_empty());
                assert!(f.return_type.is_none());
                assert!(f.body.statements.is_empty());
            }
            _ => panic!("expected function declaration"),
        }
    }

    #[test]
    fn test_2_pub_function() {
        let tokens = tokenize("pub fn main() {}");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "main");
                assert_eq!(f.visibility, Visibility::Public);
                assert!(!f.is_async);
            }
            _ => panic!("expected function declaration"),
        }
    }

    #[test]
    fn test_3_async_function() {
        let tokens = tokenize("async fn main() {}");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "main");
                assert_eq!(f.visibility, Visibility::Private);
                assert!(f.is_async);
            }
            _ => panic!("expected function declaration"),
        }
    }

    #[test]
    fn test_4_pub_async_function() {
        let tokens = tokenize("pub async fn main() {}");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "main");
                assert_eq!(f.visibility, Visibility::Public);
                assert!(f.is_async);
            }
            _ => panic!("expected function declaration"),
        }
    }

    #[test]
    fn test_5_function_with_parameters_and_return_type() {
        let tokens = tokenize("fn add(a: Int, b: Int) -> Int {}");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "add");
                assert_eq!(f.parameters.len(), 2);
                assert_eq!(f.parameters[0].name.name(), "a");
                assert_eq!(f.parameters[1].name.name(), "b");
                assert!(f.return_type.is_some());
                if let Some(ret) = &f.return_type {
                    match &ret.kind {
                        TypeKind::Named(path) => {
                            assert_eq!(path.segments[0].name(), "Int");
                        }
                        _ => panic!("expected named return type"),
                    }
                }
            }
            _ => panic!("expected function declaration"),
        }
    }

    #[test]
    fn test_6_import_declaration() {
        let tokens = tokenize("import user.User");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Import(imp) => {
                assert_eq!(imp.path.segments.len(), 2);
                assert_eq!(imp.path.segments[0].name(), "user");
                assert_eq!(imp.path.segments[1].name(), "User");
                assert_eq!(imp.visibility, Visibility::Private);
            }
            _ => panic!("expected import declaration"),
        }

        let tokens_three = tokenize("import foo.bar.Baz;");
        let program_three = parse(&tokens_three).expect("parse failed");
        assert_eq!(program_three.declarations.len(), 1);
        match &program_three.declarations[0] {
            Declaration::Import(imp) => {
                assert_eq!(imp.path.segments.len(), 3);
                assert_eq!(imp.path.segments[0].name(), "foo");
                assert_eq!(imp.path.segments[1].name(), "bar");
                assert_eq!(imp.path.segments[2].name(), "Baz");
            }
            _ => panic!("expected import declaration"),
        }
    }

    #[test]
    fn test_7_let_variable_declaration() {
        let tokens = tokenize("let name = \"Monir\"");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Variable(v) => {
                assert_eq!(v.name.name(), "name");
                assert!(!v.is_mutable);
                assert!(v.explicit_type.is_none());
                match &v.initializer.kind {
                    ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "Monir"),
                    _ => panic!("expected string literal initializer"),
                }
            }
            _ => panic!("expected variable declaration"),
        }
    }

    #[test]
    fn test_8_var_variable_declaration_with_type() {
        let tokens = tokenize("var age: Int = 30");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Variable(v) => {
                assert_eq!(v.name.name(), "age");
                assert!(v.is_mutable);
                assert!(v.explicit_type.is_some());
                match &v.initializer.kind {
                    ExprKind::Literal(Literal::Integer(i)) => assert_eq!(i, "30"),
                    _ => panic!("expected integer literal initializer"),
                }
            }
            _ => panic!("expected variable declaration"),
        }
    }

    #[test]
    fn test_9_const_declaration() {
        let tokens = tokenize("const MAX: Int = 100");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Constant(c) => {
                assert_eq!(c.name.name(), "MAX");
                assert!(c.explicit_type.is_some());
                match &c.value.kind {
                    ExprKind::Literal(Literal::Integer(i)) => assert_eq!(i, "100"),
                    _ => panic!("expected integer literal value"),
                }
            }
            _ => panic!("expected constant declaration"),
        }
    }

    #[test]
    fn test_10_attribute_on_function() {
        let tokens = tokenize("@inline\nfn main() {}");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "main");
                assert_eq!(f.attributes.len(), 1);
                assert_eq!(f.attributes[0].name.name(), "inline");
                assert!(f.attributes[0].arguments.is_empty());
            }
            _ => panic!("expected function declaration"),
        }
    }

    #[test]
    fn test_11_derive_attribute_on_struct() {
        let tokens = tokenize("@derive(Debug, Json)\nstruct User {}");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Struct(s) => {
                assert_eq!(s.name.name(), "User");
                assert_eq!(s.attributes.len(), 1);
                let attr = &s.attributes[0];
                assert_eq!(attr.name.name(), "derive");
                assert_eq!(attr.arguments.len(), 2);
                match &attr.arguments[0] {
                    AttributeArg::Identifier(id) => assert_eq!(id.name(), "Debug"),
                    _ => panic!("expected identifier arg"),
                }
                match &attr.arguments[1] {
                    AttributeArg::Identifier(id) => assert_eq!(id.name(), "Json"),
                    _ => panic!("expected identifier arg"),
                }
            }
            _ => panic!("expected struct declaration"),
        }
    }

    #[test]
    fn test_12_invalid_syntax_missing_function_name() {
        let tokens = tokenize("fn {}");
        let result = parse(&tokens);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err.kind {
            ParseErrorKind::ExpectedIdentifier { .. } => {}
            _ => panic!("expected ExpectedIdentifier error, got: {:?}", err.kind),
        }
    }

    #[test]
    fn test_13_missing_closing_brace() {
        let tokens = tokenize("fn main() {");
        let result = parse(&tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_14_unexpected_eof_does_not_panic() {
        let empty_tokens = vec![];
        let result = parse(&empty_tokens);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(program.declarations.is_empty());

        let tokens_only_eof = tokenize("");
        let result2 = parse(&tokens_only_eof);
        assert!(result2.is_ok());
        assert!(result2.unwrap().declarations.is_empty());
    }

    #[test]
    fn test_15_hello_sm_milestone_verification() {
        let tokens = tokenize("fn main() {\n    print(\"Hello, SUMER!\")\n}");
        let program = parse(&tokens).expect("failed to parse hello.sm");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "main");
                assert_eq!(f.body.statements.len(), 1);
                match &f.body.statements[0].kind {
                    StmtKind::Expression(expr_stmt) => match &expr_stmt.expr.kind {
                        ExprKind::Call { callee, arguments } => {
                            match &callee.kind {
                                ExprKind::Identifier(id) => assert_eq!(id.name(), "print"),
                                _ => panic!("expected print identifier"),
                            }
                            assert_eq!(arguments.len(), 1);
                            match &arguments[0].value.kind {
                                ExprKind::Literal(Literal::String(s)) => {
                                    assert_eq!(s, "Hello, SUMER!");
                                }
                                _ => panic!("expected string literal argument"),
                            }
                        }
                        _ => panic!("expected call expr"),
                    },
                    _ => panic!("expected expression statement"),
                }
            }
            _ => panic!("expected function declaration"),
        }
    }

    #[test]
    fn test_16_return_statement_with_expression() {
        let tokens = tokenize("fn add(a: Int, b: Int) -> Int {\n    return a + b\n}");
        let program = parse(&tokens).expect("parse failed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.body.statements.len(), 1);
                match &f.body.statements[0].kind {
                    StmtKind::Return(ret) => {
                        assert!(ret.value.is_some());
                    }
                    _ => panic!("expected return statement"),
                }
            }
            _ => panic!("expected function declaration"),
        }
    }

    #[test]
    fn test_17_token_cursor_navigation() {
        let tokens = tokenize("let x = 42;");
        let mut cursor = TokenCursor::new(&tokens);

        assert_eq!(cursor.current().kind(), &TokenKind::Let);
        assert_eq!(cursor.peek().kind(), &TokenKind::Let);
        assert_eq!(
            cursor.peek_n(1).kind(),
            &TokenKind::Identifier("x".to_string())
        );
        assert_eq!(cursor.peek_n(2).kind(), &TokenKind::Equal);
        assert!(!cursor.is_at_end());

        assert!(cursor.check(&TokenKind::Let));
        assert!(!cursor.check(&TokenKind::Var));

        assert!(cursor.match_token(&TokenKind::Let));
        assert_eq!(
            cursor.current().kind(),
            &TokenKind::Identifier("x".to_string())
        );

        let ident = cursor.parse_identifier().expect("failed ident");
        assert_eq!(ident.name(), "x");

        let eq = cursor.expect(&TokenKind::Equal).expect("failed equal");
        assert_eq!(eq.kind(), &TokenKind::Equal);

        cursor.advance(); // 42
        cursor.consume_semicolon_if_present();
        assert!(cursor.is_at_end());
    }

    #[test]
    fn test_18_error_recovery_and_reporting() {
        let tokens = tokenize("fn main(");
        assert!(parse(&tokens).is_err());

        let tokens2 = tokenize("let : Int = 10");
        assert!(parse(&tokens2).is_err());

        let tokens3 = tokenize("import");
        assert!(parse(&tokens3).is_err());

        let tokens4 = tokenize("fn foo() { let x = ; }");
        let err = parse(&tokens4).unwrap_err();
        match err.kind {
            ParseErrorKind::UnexpectedToken { .. } => {}
            _ => panic!("expected UnexpectedToken for malformed statement"),
        }
    }

    #[test]
    fn test_19_optional_semicolons_and_empty_blocks() {
        let tokens = tokenize("let a = 1; let b = 2\nconst C = 3;\n");
        let program = parse(&tokens).expect("parse failed");
        assert_eq!(program.declarations.len(), 3);
    }

    // =========================================================================
    // TASK 6 PRATT EXPRESSION PARSER TESTS
    // =========================================================================

    #[test]
    fn test_pratt_mandatory_precedence_arithmetic_1() {
        // a + b * c  =>  a + (b * c)
        let expr = parse_expr_str("a + b * c").expect("parse failed");
        match expr.kind {
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(operator, BinaryOperator::Add);
                match left.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "a"),
                    _ => panic!("expected left identifier a"),
                }
                match right.kind {
                    ExprKind::Binary {
                        left: r_left,
                        operator: r_op,
                        right: r_right,
                    } => {
                        assert_eq!(r_op, BinaryOperator::Multiply);
                        match r_left.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "b"),
                            _ => panic!("expected b"),
                        }
                        match r_right.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "c"),
                            _ => panic!("expected c"),
                        }
                    }
                    _ => panic!("expected right child to be multiplication b * c"),
                }
            }
            _ => panic!("expected addition binary expression"),
        }
    }

    #[test]
    fn test_pratt_mandatory_precedence_arithmetic_2() {
        // a * b + c  =>  (a * b) + c
        let expr = parse_expr_str("a * b + c").expect("parse failed");
        match expr.kind {
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(operator, BinaryOperator::Add);
                match left.kind {
                    ExprKind::Binary {
                        left: l_left,
                        operator: l_op,
                        right: l_right,
                    } => {
                        assert_eq!(l_op, BinaryOperator::Multiply);
                        match l_left.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "a"),
                            _ => panic!("expected a"),
                        }
                        match l_right.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "b"),
                            _ => panic!("expected b"),
                        }
                    }
                    _ => panic!("expected left child to be multiplication a * b"),
                }
                match right.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "c"),
                    _ => panic!("expected c"),
                }
            }
            _ => panic!("expected addition binary expression"),
        }
    }

    #[test]
    fn test_pratt_mandatory_precedence_arithmetic_3() {
        // a + b * c - d  =>  (a + (b * c)) - d
        let expr = parse_expr_str("a + b * c - d").expect("parse failed");
        match expr.kind {
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(operator, BinaryOperator::Subtract);
                match left.kind {
                    ExprKind::Binary {
                        left: l_left,
                        operator: l_op,
                        right: l_right,
                    } => {
                        assert_eq!(l_op, BinaryOperator::Add);
                        match l_left.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "a"),
                            _ => panic!("expected a"),
                        }
                        match l_right.kind {
                            ExprKind::Binary { operator: r_op, .. } => {
                                assert_eq!(r_op, BinaryOperator::Multiply);
                            }
                            _ => panic!("expected multiplication b * c"),
                        }
                    }
                    _ => panic!("expected left child to be a + (b * c)"),
                }
                match right.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "d"),
                    _ => panic!("expected d"),
                }
            }
            _ => panic!("expected subtraction binary expression"),
        }
    }

    #[test]
    fn test_pratt_mandatory_precedence_equality_logical() {
        // a == b && c != d  =>  (a == b) && (c != d)
        let expr = parse_expr_str("a == b && c != d").expect("parse failed");
        match expr.kind {
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(operator, BinaryOperator::LogicalAnd);
                match left.kind {
                    ExprKind::Binary { operator: l_op, .. } => {
                        assert_eq!(l_op, BinaryOperator::Equal);
                    }
                    _ => panic!("expected a == b"),
                }
                match right.kind {
                    ExprKind::Binary { operator: r_op, .. } => {
                        assert_eq!(r_op, BinaryOperator::NotEqual);
                    }
                    _ => panic!("expected c != d"),
                }
            }
            _ => panic!("expected LogicalAnd expression"),
        }
    }

    #[test]
    fn test_pratt_mandatory_precedence_logical_or_and() {
        // a || b && c  =>  a || (b && c)
        let expr = parse_expr_str("a || b && c").expect("parse failed");
        match expr.kind {
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(operator, BinaryOperator::LogicalOr);
                match left.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "a"),
                    _ => panic!("expected a"),
                }
                match right.kind {
                    ExprKind::Binary {
                        operator: r_op,
                        left: r_l,
                        right: r_r,
                    } => {
                        assert_eq!(r_op, BinaryOperator::LogicalAnd);
                        match r_l.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "b"),
                            _ => panic!("expected b"),
                        }
                        match r_r.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "c"),
                            _ => panic!("expected c"),
                        }
                    }
                    _ => panic!("expected b && c"),
                }
            }
            _ => panic!("expected LogicalOr expression"),
        }
    }

    #[test]
    fn test_pratt_mandatory_precedence_unary_multiplication() {
        // -a * b  =>  (-a) * b
        let expr = parse_expr_str("-a * b").expect("parse failed");
        match expr.kind {
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(operator, BinaryOperator::Multiply);
                match left.kind {
                    ExprKind::Unary {
                        operator: u_op,
                        operand,
                    } => {
                        assert_eq!(u_op, UnaryOperator::Negate);
                        match operand.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "a"),
                            _ => panic!("expected a"),
                        }
                    }
                    _ => panic!("expected unary negate -a"),
                }
                match right.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "b"),
                    _ => panic!("expected b"),
                }
            }
            _ => panic!("expected multiplication expression"),
        }
    }

    #[test]
    fn test_pratt_mandatory_parentheses_override() {
        // (a + b) * c
        let expr = parse_expr_str("(a + b) * c").expect("parse failed");
        match expr.kind {
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(operator, BinaryOperator::Multiply);
                match left.kind {
                    ExprKind::Binary {
                        left: l_l,
                        operator: l_op,
                        right: l_r,
                    } => {
                        assert_eq!(l_op, BinaryOperator::Add);
                        match l_l.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "a"),
                            _ => panic!("expected a"),
                        }
                        match l_r.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "b"),
                            _ => panic!("expected b"),
                        }
                    }
                    _ => panic!("expected (a + b)"),
                }
                match right.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "c"),
                    _ => panic!("expected c"),
                }
            }
            _ => panic!("expected multiplication"),
        }
    }

    #[test]
    fn test_pratt_assignment_right_associativity() {
        // a = b = c  =>  a = (b = c)
        let expr = parse_expr_str("a = b = c").expect("parse failed");
        match expr.kind {
            ExprKind::Assignment {
                target,
                operator,
                value,
            } => {
                assert_eq!(operator, AssignmentOperator::Assign);
                match target.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "a"),
                    _ => panic!("expected a"),
                }
                match value.kind {
                    ExprKind::Assignment {
                        target: v_target,
                        operator: v_op,
                        value: v_value,
                    } => {
                        assert_eq!(v_op, AssignmentOperator::Assign);
                        match v_target.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "b"),
                            _ => panic!("expected b"),
                        }
                        match v_value.kind {
                            ExprKind::Identifier(id) => assert_eq!(id.name(), "c"),
                            _ => panic!("expected c"),
                        }
                    }
                    _ => panic!("expected right child assignment b = c"),
                }
            }
            _ => panic!("expected assignment expression"),
        }
    }

    #[test]
    fn test_pratt_compound_assignments() {
        // x += 10
        let expr1 = parse_expr_str("x += 10").expect("parse failed");
        match expr1.kind {
            ExprKind::Assignment { operator, .. } => {
                assert_eq!(operator, AssignmentOperator::AddAssign);
            }
            _ => panic!("expected AddAssign"),
        }

        // x -= 5, x *= 2, x /= 3, x %= 4
        assert!(matches!(
            parse_expr_str("x -= 5").unwrap().kind,
            ExprKind::Assignment {
                operator: AssignmentOperator::SubAssign,
                ..
            }
        ));
        assert!(matches!(
            parse_expr_str("x *= 2").unwrap().kind,
            ExprKind::Assignment {
                operator: AssignmentOperator::MulAssign,
                ..
            }
        ));
        assert!(matches!(
            parse_expr_str("x /= 3").unwrap().kind,
            ExprKind::Assignment {
                operator: AssignmentOperator::DivAssign,
                ..
            }
        ));
        assert!(matches!(
            parse_expr_str("x %= 4").unwrap().kind,
            ExprKind::Assignment {
                operator: AssignmentOperator::ModAssign,
                ..
            }
        ));
    }

    #[test]
    fn test_pratt_basic_literals() {
        let e1 = parse_expr_str("42").unwrap();
        assert!(matches!(
            e1.kind,
            ExprKind::Literal(Literal::Integer(ref s)) if s == "42"
        ));

        let e2 = parse_expr_str("3.14").unwrap();
        assert!(matches!(
            e2.kind,
            ExprKind::Literal(Literal::Float(ref s)) if s == "3.14"
        ));

        let e3 = parse_expr_str("\"hello\"").unwrap();
        assert!(matches!(
            e3.kind,
            ExprKind::Literal(Literal::String(ref s)) if s == "hello"
        ));

        let e4 = parse_expr_str("'c'").unwrap();
        assert!(matches!(e4.kind, ExprKind::Literal(Literal::Char('c'))));

        let e5 = parse_expr_str("true").unwrap();
        assert!(matches!(e5.kind, ExprKind::Literal(Literal::Bool(true))));

        let e6 = parse_expr_str("false").unwrap();
        assert!(matches!(e6.kind, ExprKind::Literal(Literal::Bool(false))));

        let e7 = parse_expr_str("None").unwrap();
        assert!(matches!(e7.kind, ExprKind::Literal(Literal::None)));
    }

    #[test]
    fn test_pratt_unary_and_references() {
        let e1 = parse_expr_str("!active").unwrap();
        match e1.kind {
            ExprKind::Unary { operator, operand } => {
                assert_eq!(operator, UnaryOperator::Not);
                match operand.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "active"),
                    _ => panic!("expected active"),
                }
            }
            _ => panic!("expected Unary Not"),
        }

        let e2 = parse_expr_str("+value").unwrap();
        match e2.kind {
            ExprKind::Unary { operator, .. } => assert_eq!(operator, UnaryOperator::Positive),
            _ => panic!("expected Unary Positive"),
        }

        let e3 = parse_expr_str("&user").unwrap();
        match e3.kind {
            ExprKind::Reference { mutable, operand } => {
                assert!(!mutable);
                match operand.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "user"),
                    _ => panic!("expected user"),
                }
            }
            _ => panic!("expected immutable reference"),
        }

        let e4 = parse_expr_str("&mut user").unwrap();
        match e4.kind {
            ExprKind::Reference { mutable, operand } => {
                assert!(mutable);
                match operand.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "user"),
                    _ => panic!("expected user"),
                }
            }
            _ => panic!("expected mutable reference"),
        }

        // Chained unary: `!!x`
        let e5 = parse_expr_str("!!x").unwrap();
        match e5.kind {
            ExprKind::Unary {
                operator: op1,
                operand: inner,
            } => {
                assert_eq!(op1, UnaryOperator::Not);
                match inner.kind {
                    ExprKind::Unary { operator: op2, .. } => {
                        assert_eq!(op2, UnaryOperator::Not);
                    }
                    _ => panic!("expected second unary not"),
                }
            }
            _ => panic!("expected chained unary"),
        }
    }

    #[test]
    fn test_pratt_comparison_and_equality() {
        let e1 = parse_expr_str("age >= 18").unwrap();
        match e1.kind {
            ExprKind::Binary { operator, .. } => {
                assert_eq!(operator, BinaryOperator::GreaterEqual);
            }
            _ => panic!("expected GreaterEqual"),
        }

        let e2 = parse_expr_str("x < 10").unwrap();
        match e2.kind {
            ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::Less),
            _ => panic!("expected Less"),
        }

        let e3 = parse_expr_str("name == \"Monir\"").unwrap();
        match e3.kind {
            ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::Equal),
            _ => panic!("expected Equal"),
        }

        let e4 = parse_expr_str("a != b").unwrap();
        match e4.kind {
            ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::NotEqual),
            _ => panic!("expected NotEqual"),
        }
    }

    #[test]
    fn test_pratt_function_calls_positional_and_named() {
        let e1 = parse_expr_str("add(1, 2)").unwrap();
        match e1.kind {
            ExprKind::Call { callee, arguments } => {
                match callee.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "add"),
                    _ => panic!("expected add"),
                }
                assert_eq!(arguments.len(), 2);
                assert!(arguments[0].name.is_none());
                assert!(arguments[1].name.is_none());
            }
            _ => panic!("expected Call"),
        }

        let e2 = parse_expr_str("createUser(name: \"Monir\", age: 30)").unwrap();
        match e2.kind {
            ExprKind::Call { callee, arguments } => {
                match callee.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "createUser"),
                    _ => panic!("expected createUser"),
                }
                assert_eq!(arguments.len(), 2);
                assert_eq!(arguments[0].name.as_ref().unwrap().name(), "name");
                assert_eq!(arguments[1].name.as_ref().unwrap().name(), "age");
            }
            _ => panic!("expected Call"),
        }

        let e3 = parse_expr_str("foo()").unwrap();
        match e3.kind {
            ExprKind::Call { arguments, .. } => assert!(arguments.is_empty()),
            _ => panic!("expected empty call"),
        }
    }

    #[test]
    fn test_pratt_member_and_optional_member() {
        let e1 = parse_expr_str("user.name").unwrap();
        match e1.kind {
            ExprKind::Member { object, member } => {
                match object.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "user"),
                    _ => panic!("expected user"),
                }
                assert_eq!(member.name(), "name");
            }
            _ => panic!("expected Member"),
        }

        let e2 = parse_expr_str("user?.name").unwrap();
        match e2.kind {
            ExprKind::OptionalMember { object, member } => {
                match object.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "user"),
                    _ => panic!("expected user"),
                }
                assert_eq!(member.name(), "name");
            }
            _ => panic!("expected OptionalMember"),
        }

        // user.name == "Monir"  =>  (user.name) == "Monir"
        let e3 = parse_expr_str("user.name == \"Monir\"").unwrap();
        match e3.kind {
            ExprKind::Binary { left, operator, .. } => {
                assert_eq!(operator, BinaryOperator::Equal);
                match left.kind {
                    ExprKind::Member { .. } => {}
                    _ => panic!("expected member access on left"),
                }
            }
            _ => panic!("expected Equal binary"),
        }
    }

    #[test]
    fn test_pratt_indexing() {
        let e1 = parse_expr_str("items[0]").unwrap();
        match e1.kind {
            ExprKind::Index { object, index } => {
                match object.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "items"),
                    _ => panic!("expected items"),
                }
                match index.kind {
                    ExprKind::Literal(Literal::Integer(ref s)) => assert_eq!(s, "0"),
                    _ => panic!("expected index 0"),
                }
            }
            _ => panic!("expected Index"),
        }

        // items[index + 1]
        let e2 = parse_expr_str("items[index + 1]").unwrap();
        match e2.kind {
            ExprKind::Index { index, .. } => match index.kind {
                ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::Add),
                _ => panic!("expected binary index expr"),
            },
            _ => panic!("expected Index"),
        }

        // matrix[i][j]
        let e3 = parse_expr_str("matrix[i][j]").unwrap();
        match e3.kind {
            ExprKind::Index { object, index } => {
                match index.kind {
                    ExprKind::Identifier(id) => assert_eq!(id.name(), "j"),
                    _ => panic!("expected j"),
                }
                match object.kind {
                    ExprKind::Index {
                        index: inner_idx, ..
                    } => match inner_idx.kind {
                        ExprKind::Identifier(id) => assert_eq!(id.name(), "i"),
                        _ => panic!("expected i"),
                    },
                    _ => panic!("expected inner index"),
                }
            }
            _ => panic!("expected 2D index"),
        }
    }

    #[test]
    fn test_pratt_force_unwrap() {
        let e1 = parse_expr_str("user!").unwrap();
        match e1.kind {
            ExprKind::ForceUnwrap(inner) => match inner.kind {
                ExprKind::Identifier(id) => assert_eq!(id.name(), "user"),
                _ => panic!("expected user"),
            },
            _ => panic!("expected ForceUnwrap"),
        }

        // user!.name  =>  (user!).name
        let e2 = parse_expr_str("user!.name").unwrap();
        match e2.kind {
            ExprKind::Member { object, member } => {
                assert_eq!(member.name(), "name");
                match object.kind {
                    ExprKind::ForceUnwrap(inner) => match inner.kind {
                        ExprKind::Identifier(id) => assert_eq!(id.name(), "user"),
                        _ => panic!("expected user"),
                    },
                    _ => panic!("expected ForceUnwrap object"),
                }
            }
            _ => panic!("expected Member"),
        }
    }

    #[test]
    fn test_pratt_postfix_chaining() {
        // users[0]?.address?.city
        let e1 = parse_expr_str("users[0]?.address?.city").expect("parse failed");
        match e1.kind {
            ExprKind::OptionalMember { object, member } => {
                assert_eq!(member.name(), "city");
                match object.kind {
                    ExprKind::OptionalMember {
                        object: inner_obj,
                        member: inner_mem,
                    } => {
                        assert_eq!(inner_mem.name(), "address");
                        match inner_obj.kind {
                            ExprKind::Index { .. } => {}
                            _ => panic!("expected index"),
                        }
                    }
                    _ => panic!("expected optional member address"),
                }
            }
            _ => panic!("expected optional member city"),
        }

        // getUser(id)!.name
        let e2 = parse_expr_str("getUser(id)!.name").expect("parse failed");
        match e2.kind {
            ExprKind::Member { object, member } => {
                assert_eq!(member.name(), "name");
                match object.kind {
                    ExprKind::ForceUnwrap(inner) => match inner.kind {
                        ExprKind::Call { .. } => {}
                        _ => panic!("expected Call inside unwrap"),
                    },
                    _ => panic!("expected ForceUnwrap"),
                }
            }
            _ => panic!("expected Member"),
        }

        // user.name()
        let e3 = parse_expr_str("user.name()").expect("parse failed");
        match e3.kind {
            ExprKind::Call { callee, .. } => match callee.kind {
                ExprKind::Member { member, .. } => assert_eq!(member.name(), "name"),
                _ => panic!("expected Member callee"),
            },
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_pratt_ranges() {
        let e1 = parse_expr_str("0..10").unwrap();
        match e1.kind {
            ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::Range),
            _ => panic!("expected Range"),
        }

        let e2 = parse_expr_str("0..=10").unwrap();
        match e2.kind {
            ExprKind::Binary { operator, .. } => {
                assert_eq!(operator, BinaryOperator::RangeInclusive);
            }
            _ => panic!("expected RangeInclusive"),
        }

        // Ensure 1..10 and 1..=10 do not parse as floats
        let e3 = parse_expr_str("1..10").unwrap();
        match e3.kind {
            ExprKind::Binary { left, operator, .. } => {
                assert_eq!(operator, BinaryOperator::Range);
                match left.kind {
                    ExprKind::Literal(Literal::Integer(ref s)) => assert_eq!(s, "1"),
                    _ => panic!("expected integer 1 on left"),
                }
            }
            _ => panic!("expected Range"),
        }
    }

    #[test]
    fn test_pratt_array_and_map_literals() {
        let e1 = parse_expr_str("[]").unwrap();
        match e1.kind {
            ExprKind::Array(elems) => assert!(elems.is_empty()),
            _ => panic!("expected empty array"),
        }

        let e2 = parse_expr_str("[1, 2, 3]").unwrap();
        match e2.kind {
            ExprKind::Array(elems) => assert_eq!(elems.len(), 3),
            _ => panic!("expected array of 3"),
        }

        let e3 = parse_expr_str("[1 + 2, 3 * 4]").unwrap();
        match e3.kind {
            ExprKind::Array(elems) => {
                assert_eq!(elems.len(), 2);
                match elems[0].kind {
                    ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::Add),
                    _ => panic!("expected add"),
                }
                match elems[1].kind {
                    ExprKind::Binary { operator, .. } => {
                        assert_eq!(operator, BinaryOperator::Multiply);
                    }
                    _ => panic!("expected multiply"),
                }
            }
            _ => panic!("expected array of binary exprs"),
        }

        // Map literal: `{"name": "Monir", "age": 30}`
        let e4 = parse_expr_str("{\"name\": \"Monir\", \"age\": 30}").unwrap();
        match e4.kind {
            ExprKind::Map(entries) => {
                assert_eq!(entries.len(), 2);
                match &entries[0].key.kind {
                    ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "name"),
                    _ => panic!("expected string key"),
                }
                match &entries[0].value.kind {
                    ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "Monir"),
                    _ => panic!("expected string value"),
                }
            }
            _ => panic!("expected Map"),
        }
    }

    #[test]
    fn test_pratt_await_and_spawn() {
        let e1 = parse_expr_str("await task").unwrap();
        match e1.kind {
            ExprKind::Await(operand) => match operand.kind {
                ExprKind::Identifier(id) => assert_eq!(id.name(), "task"),
                _ => panic!("expected task"),
            },
            _ => panic!("expected Await"),
        }

        let e2 = parse_expr_str("spawn download(url)").unwrap();
        match e2.kind {
            ExprKind::Spawn(operand) => match operand.kind {
                ExprKind::Call { .. } => {}
                _ => panic!("expected Call inside spawn"),
            },
            _ => panic!("expected Spawn"),
        }
    }

    #[test]
    fn test_pratt_lambdas() {
        // (a, b) => a + b
        let e1 = parse_expr_str("(a, b) => a + b").unwrap();
        match e1.kind {
            ExprKind::Lambda { parameters, body } => {
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0].name(), "a");
                assert_eq!(parameters[1].name(), "b");
                match body.kind {
                    ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::Add),
                    _ => panic!("expected body a + b"),
                }
            }
            _ => panic!("expected Lambda"),
        }

        // x => x * 2
        let e2 = parse_expr_str("x => x * 2").unwrap();
        match e2.kind {
            ExprKind::Lambda { parameters, body } => {
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters[0].name(), "x");
                match body.kind {
                    ExprKind::Binary { operator, .. } => {
                        assert_eq!(operator, BinaryOperator::Multiply);
                    }
                    _ => panic!("expected multiply body"),
                }
            }
            _ => panic!("expected Lambda"),
        }

        // () => 42
        let e3 = parse_expr_str("() => 42").unwrap();
        match e3.kind {
            ExprKind::Lambda { parameters, body } => {
                assert!(parameters.is_empty());
                match body.kind {
                    ExprKind::Literal(Literal::Integer(ref s)) => assert_eq!(s, "42"),
                    _ => panic!("expected 42"),
                }
            }
            _ => panic!("expected Lambda"),
        }
    }

    #[test]
    fn test_pratt_statement_integration() {
        // let x = 10 + 20 * 3
        let tokens1 = tokenize("let x = 10 + 20 * 3");
        let prog1 = parse(&tokens1).expect("parse failed");
        match &prog1.declarations[0] {
            Declaration::Variable(v) => match v.initializer.kind {
                ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::Add),
                _ => panic!("expected binary initializer"),
            },
            _ => panic!("expected variable decl"),
        }

        // fn test() { return user.name }
        let tokens2 = tokenize("fn test() { return user.name }");
        let prog2 = parse(&tokens2).expect("parse failed");
        match &prog2.declarations[0] {
            Declaration::Function(f) => match &f.body.statements[0].kind {
                StmtKind::Return(r) => match &r.value.as_ref().unwrap().kind {
                    ExprKind::Member { member, .. } => assert_eq!(member.name(), "name"),
                    _ => panic!("expected member return"),
                },
                _ => panic!("expected return"),
            },
            _ => panic!("expected function"),
        }

        // foo(a + b, c * d) inside statement
        let prog3 = parse(&tokenize("fn run() { foo(a + b, c * d) }")).expect("parse failed");
        assert_eq!(prog3.declarations.len(), 1);
        let expr3 = parse_expr_str("foo(a + b, c * d)").unwrap();
        match expr3.kind {
            ExprKind::Call { arguments, .. } => {
                assert_eq!(arguments.len(), 2);
                match arguments[0].value.kind {
                    ExprKind::Binary { operator, .. } => assert_eq!(operator, BinaryOperator::Add),
                    _ => panic!("expected add"),
                }
                match arguments[1].value.kind {
                    ExprKind::Binary { operator, .. } => {
                        assert_eq!(operator, BinaryOperator::Multiply);
                    }
                    _ => panic!("expected multiply"),
                }
            }
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_pratt_error_handling_no_panics() {
        assert!(parse_expr_str("a +").is_err());
        assert!(parse_expr_str("(a + b").is_err());
        assert!(parse_expr_str("foo(").is_err());
        assert!(parse_expr_str("items[").is_err());
        assert!(parse_expr_str("user.").is_err());
        assert!(parse_expr_str("user?.").is_err());
        assert!(parse_expr_str("a =").is_err());
    }

    #[test]
    fn test_pratt_spans_preservation() {
        let expr = parse_expr_str("a + b").unwrap();
        assert_eq!(expr.span.start, 0);
        assert_eq!(expr.span.end, 5);

        let expr2 = parse_expr_str("user.name").unwrap();
        assert_eq!(expr2.span.start, 0);
        assert_eq!(expr2.span.end, 9);

        let expr3 = parse_expr_str("foo(a, b)").unwrap();
        assert_eq!(expr3.span.start, 0);
        assert_eq!(expr3.span.end, 9);

        let expr4 = parse_expr_str("-a").unwrap();
        assert_eq!(expr4.span.start, 0);
        assert_eq!(expr4.span.end, 2);
    }

    // =========================================================================
    // TASK 7 FULL PARSER INTEGRATION TESTS
    // =========================================================================

    #[test]
    fn test_task7_trait_declaration_signatures_and_defaults() {
        let src = r#"
trait Comparable<T>: Clone + Debug {
    fn compare(other: T) -> Int;
    fn is_equal(other: T) -> Bool {
        return compare(other) == 0
    }
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse trait");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Trait(t) => {
                assert_eq!(t.name.name(), "Comparable");
                assert_eq!(t.generics.params.len(), 1);
                assert_eq!(t.generics.params[0].name.name(), "T");
                assert_eq!(t.bounds.len(), 2);
                assert_eq!(t.bounds[0].name.name(), "Clone");
                assert_eq!(t.bounds[1].name.name(), "Debug");
                assert_eq!(t.members.len(), 2);
                match &t.members[0] {
                    TraitMember::FunctionSignature(sig) => {
                        assert_eq!(sig.name.name(), "compare");
                        assert!(sig.return_type.is_some());
                    }
                    _ => panic!("expected abstract function signature"),
                }
                match &t.members[1] {
                    TraitMember::Function(f) => {
                        assert_eq!(f.name.name(), "is_equal");
                        assert_eq!(f.body.statements.len(), 1);
                    }
                    _ => panic!("expected default function implementation"),
                }
            }
            _ => panic!("expected Trait declaration"),
        }
    }

    #[test]
    fn test_task7_impl_declarations_inherent_and_trait() {
        let src = r#"
impl User {
    fn greet() {
        print("hello")
    }
}

impl Printable for User {
    fn print() {
        print("user")
    }
}

impl<T: Printable> Box<T> {
    fn show() {}
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse impls");
        assert_eq!(program.declarations.len(), 3);

        // 1. Inherent impl User
        match &program.declarations[0] {
            Declaration::Impl(i) => {
                assert!(i.trait_type.is_none());
                match &i.target_type.kind {
                    TypeKind::Named(path) => assert_eq!(path.segments[0].name(), "User"),
                    _ => panic!("expected User"),
                }
                assert_eq!(i.members.len(), 1);
                assert_eq!(i.members[0].name.name(), "greet");
            }
            _ => panic!("expected Inherent Impl"),
        }

        // 2. Trait impl Printable for User
        match &program.declarations[1] {
            Declaration::Impl(i) => {
                assert!(i.trait_type.is_some());
                match &i.trait_type.as_ref().unwrap().kind {
                    TypeKind::Named(path) => assert_eq!(path.segments[0].name(), "Printable"),
                    _ => panic!("expected Printable"),
                }
                match &i.target_type.kind {
                    TypeKind::Named(path) => assert_eq!(path.segments[0].name(), "User"),
                    _ => panic!("expected User"),
                }
                assert_eq!(i.members.len(), 1);
                assert_eq!(i.members[0].name.name(), "print");
            }
            _ => panic!("expected Trait Impl"),
        }

        // 3. Generic impl<T: Printable> Box<T>
        match &program.declarations[2] {
            Declaration::Impl(i) => {
                assert_eq!(i.generics.params.len(), 1);
                assert_eq!(i.generics.params[0].bounds.len(), 1);
                assert_eq!(i.generics.params[0].bounds[0].name.name(), "Printable");
            }
            _ => panic!("expected Generic Impl"),
        }
    }

    #[test]
    fn test_task7_enum_variants_unit_tuple_struct() {
        let src = r#"
enum Status {
    Active,
    Inactive,
    Suspended
}

enum Payment {
    Cash,
    Card(String),
    Mobile(String)
}

enum Message {
    Text {
        content: String
    },
    Image {
        url: String
    }
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse enums");
        assert_eq!(program.declarations.len(), 3);

        // Unit variants
        match &program.declarations[0] {
            Declaration::Enum(e) => {
                assert_eq!(e.name.name(), "Status");
                assert_eq!(e.variants.len(), 3);
                assert!(matches!(e.variants[0].data, VariantData::Unit));
                assert_eq!(e.variants[0].name.name(), "Active");
            }
            _ => panic!("expected enum Status"),
        }

        // Tuple variants
        match &program.declarations[1] {
            Declaration::Enum(e) => {
                assert_eq!(e.name.name(), "Payment");
                assert_eq!(e.variants.len(), 3);
                assert!(matches!(e.variants[0].data, VariantData::Unit));
                match &e.variants[1].data {
                    VariantData::Tuple(types) => {
                        assert_eq!(types.len(), 1);
                    }
                    _ => panic!("expected Tuple variant"),
                }
            }
            _ => panic!("expected enum Payment"),
        }

        // Struct variants
        match &program.declarations[2] {
            Declaration::Enum(e) => {
                assert_eq!(e.name.name(), "Message");
                assert_eq!(e.variants.len(), 2);
                match &e.variants[0].data {
                    VariantData::Struct(fields) => {
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0].name.name(), "content");
                    }
                    _ => panic!("expected Struct variant"),
                }
            }
            _ => panic!("expected enum Message"),
        }
    }

    #[test]
    fn test_task7_struct_with_generics_fields_methods() {
        let src = r#"
@derive(Debug)
pub struct Box<T> {
    id: Int
    value: T

    fn get_value() -> T {
        return value
    }
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse struct");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Struct(s) => {
                assert_eq!(s.name.name(), "Box");
                assert_eq!(s.visibility, Visibility::Public);
                assert_eq!(s.attributes.len(), 1);
                assert_eq!(s.generics.params.len(), 1);
                assert_eq!(s.generics.params[0].name.name(), "T");
                assert_eq!(s.members.len(), 3);
                assert!(matches!(s.members[0], StructMember::Field(_)));
                assert!(matches!(s.members[1], StructMember::Field(_)));
                match &s.members[2] {
                    StructMember::Method(m) => {
                        assert_eq!(m.name.name(), "get_value");
                        assert!(m.return_type.is_some());
                    }
                    _ => panic!("expected method member"),
                }
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn test_task7_function_generics_bounds_default_values() {
        let src = r#"
pub async fn download<T: Serializable + Clone>(value: T = default_val) -> Result<Data, Error> {
    return value
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse function");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "download");
                assert_eq!(f.visibility, Visibility::Public);
                assert!(f.is_async);
                assert_eq!(f.generics.params.len(), 1);
                assert_eq!(f.generics.params[0].bounds.len(), 2);
                assert_eq!(f.generics.params[0].bounds[0].name.name(), "Serializable");
                assert_eq!(f.generics.params[0].bounds[1].name.name(), "Clone");
                assert_eq!(f.parameters.len(), 1);
                assert!(f.parameters[0].default_value.is_some());
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_task7_if_else_and_else_if_statements() {
        let src = r#"
fn check_score(score: Int) {
    if score >= 80 {
        print("A")
    } else if score >= 60 {
        print("B")
    } else {
        print("C")
    }
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse if-else-if");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.body.statements.len(), 1);
                match &f.body.statements[0].kind {
                    StmtKind::If(if_stmt) => {
                        assert!(if_stmt.else_branch.is_some());
                        match if_stmt.else_branch.as_ref().unwrap() {
                            ElseStmtBranch::ElseIf(nested_if) => {
                                assert!(nested_if.else_branch.is_some());
                                match nested_if.else_branch.as_ref().unwrap() {
                                    ElseStmtBranch::Block(b) => {
                                        assert_eq!(b.statements.len(), 1);
                                    }
                                    _ => panic!("expected final block"),
                                }
                            }
                            _ => panic!("expected chained ElseIf"),
                        }
                    }
                    _ => panic!("expected if statement"),
                }
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_task7_while_for_loop_statements() {
        let src = r#"
fn loop_test() {
    while count < 10 {
        count += 1
    }

    for item in items {
        print(item)
    }

    for i in 0..10 {
        if i == 5 {
            continue
        }
        print(i)
    }

    for i in 0..=10 {
        print(i)
    }

    loop {
        break
    }
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse loops");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.body.statements.len(), 5);
                assert!(matches!(f.body.statements[0].kind, StmtKind::While(_)));
                assert!(matches!(f.body.statements[1].kind, StmtKind::For(_)));
                assert!(matches!(f.body.statements[2].kind, StmtKind::For(_)));
                assert!(matches!(f.body.statements[3].kind, StmtKind::For(_)));
                assert!(matches!(f.body.statements[4].kind, StmtKind::Loop(_)));
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_task7_match_statements_and_expressions() {
        let src = r#"
fn test_match(payment: Payment, value: Option<Int>) -> Int {
    match payment {
        Payment.Cash => {
            print("cash")
        }
        Payment.Card(number) => {
            print(number)
        }
    }

    let result = match value {
        Some(x) => x
        None => 0
    }

    return result
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse match");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.body.statements.len(), 3);
                // 1. match statement
                match &f.body.statements[0].kind {
                    StmtKind::Match(m) => {
                        assert_eq!(m.arms.len(), 2);
                        match &m.arms[0].pattern.kind {
                            PatternKind::Enum { variant, data, .. } => {
                                assert_eq!(variant.name(), "Cash");
                                assert!(data.is_none());
                            }
                            _ => panic!("expected Payment.Cash enum pattern"),
                        }
                        match &m.arms[1].pattern.kind {
                            PatternKind::Enum { variant, data, .. } => {
                                assert_eq!(variant.name(), "Card");
                                assert_eq!(data.as_ref().unwrap().len(), 1);
                            }
                            _ => panic!("expected Payment.Card(number) enum pattern"),
                        }
                    }
                    _ => panic!("expected match statement"),
                }

                // 2. match expression inside let
                match &f.body.statements[1].kind {
                    StmtKind::Variable(v) => match &v.initializer.kind {
                        ExprKind::Match { arms, .. } => {
                            assert_eq!(arms.len(), 2);
                            match &arms[0].pattern.kind {
                                PatternKind::Enum { variant, data, .. } => {
                                    assert_eq!(variant.name(), "Some");
                                    assert_eq!(data.as_ref().unwrap().len(), 1);
                                }
                                _ => panic!("expected Some(x) pattern"),
                            }
                            match &arms[1].pattern.kind {
                                PatternKind::Literal(Literal::None) => {}
                                _ => panic!("expected None literal pattern"),
                            }
                        }
                        _ => panic!("expected match expression"),
                    },
                    _ => panic!("expected variable binding"),
                }
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_task7_if_expressions() {
        let expr = parse_expr_str("if active { \"Active\" } else { \"Inactive\" }")
            .expect("failed if expression");
        match expr.kind {
            ExprKind::If(i) => {
                assert!(i.else_branch.is_some());
                match i.else_branch.unwrap() {
                    ElseExprBranch::Block(b) => assert_eq!(b.statements.len(), 1),
                    _ => panic!("expected else block"),
                }
            }
            _ => panic!("expected If expr"),
        }
    }

    #[test]
    fn test_task7_patterns_comprehensive() {
        let p1 = parse_pattern(&mut TokenCursor::new(&tokenize("_"))).unwrap();
        assert!(matches!(p1.kind, PatternKind::Wildcard));

        let p2 = parse_pattern(&mut TokenCursor::new(&tokenize("user"))).unwrap();
        match p2.kind {
            PatternKind::Identifier(id) => assert_eq!(id.name(), "user"),
            _ => panic!("expected identifier"),
        }

        let p3 = parse_pattern(&mut TokenCursor::new(&tokenize("(a, b, c)"))).unwrap();
        match p3.kind {
            PatternKind::Tuple(elems) => assert_eq!(elems.len(), 3),
            _ => panic!("expected tuple"),
        }

        let p4 = parse_pattern(&mut TokenCursor::new(&tokenize(
            "User { name, age: userAge }",
        )))
        .unwrap();
        match p4.kind {
            PatternKind::Struct { path, fields } => {
                assert_eq!(path.segments[0].name(), "User");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name.name(), "name");
                assert!(fields[0].pattern.is_none());
                assert_eq!(fields[1].name.name(), "age");
                assert!(fields[1].pattern.is_some());
            }
            _ => panic!("expected struct pattern"),
        }

        let p5 = parse_pattern(&mut TokenCursor::new(&tokenize("-42"))).unwrap();
        match p5.kind {
            PatternKind::Literal(Literal::Integer(ref s)) => assert_eq!(s, "-42"),
            _ => panic!("expected negative integer pattern"),
        }
    }

    #[test]
    fn test_task7_types_nesting_and_fixed_array() {
        let t1 = crate::types::parse_type(&mut TokenCursor::new(&tokenize("List<User?>"))).unwrap();
        match t1.kind {
            TypeKind::Generic { name, arguments } => {
                assert_eq!(name.segments[0].name(), "List");
                assert_eq!(arguments.len(), 1);
                assert!(matches!(arguments[0].kind, TypeKind::Optional { .. }));
            }
            _ => panic!("expected List<User?>"),
        }

        let t2 = crate::types::parse_type(&mut TokenCursor::new(&tokenize("[Int; 4]"))).unwrap();
        match t2.kind {
            TypeKind::Array { element, size } => {
                assert!(matches!(element.kind, TypeKind::Named(_)));
                assert!(size.is_some());
            }
            _ => panic!("expected [Int; 4]"),
        }

        let t3 =
            crate::types::parse_type(&mut TokenCursor::new(&tokenize("&mut List<Int>"))).unwrap();
        match t3.kind {
            TypeKind::MutableReference { inner } => {
                assert!(matches!(inner.kind, TypeKind::Generic { .. }));
            }
            _ => panic!("expected &mut List<Int>"),
        }
    }

    #[test]
    fn test_task7_controlled_error_handling() {
        assert!(parse(&tokenize("fn main( {}")).is_err());
        assert!(parse(&tokenize("struct User { name }")).is_err());
        assert!(parse(&tokenize("fn foo(a Int) {}")).is_err());
        assert!(parse(&tokenize("if { }")).is_err());
        assert!(parse(&tokenize("for item items { }")).is_err());
        assert!(parse(&tokenize("match value { }")).is_err());
        assert!(parse(&tokenize("import")).is_err());
        assert!(parse(&tokenize("fn main() { let x = }")).is_err());
    }

    #[test]
    fn test_task7_full_realistic_source_golden_test() {
        let src = r#"
@derive(Debug)
pub struct User<T> {
    id: Int
    value: T

    fn greet() {
        print("Hello")
    }
}

enum Status {
    Active
    Inactive
}

trait Printable {
    fn print()
}

impl Printable for User<Int> {
    fn print() {
        print("User")
    }
}

pub async fn main<T: Printable>(value: T) -> Result<Int, Error> {
    let age: Int = 30
    var count = 0

    if age >= 18 && count == 0 {
        count += 1
    } else {
        count = 0
    }

    for i in 0..10 {
        if i == 5 {
            continue
        }

        print(i)
    }

    loop {
        break
    }

    return Ok(count)
}
"#;
        let program = parse(&tokenize(src)).expect("failed to parse realistic SUMER source");
        assert_eq!(program.declarations.len(), 5);

        // 1. User<T> struct
        match &program.declarations[0] {
            Declaration::Struct(s) => {
                assert_eq!(s.name.name(), "User");
                assert_eq!(s.visibility, Visibility::Public);
                assert_eq!(s.attributes.len(), 1);
                assert_eq!(s.generics.params.len(), 1);
                assert_eq!(s.members.len(), 3);
            }
            _ => panic!("expected struct User"),
        }

        // 2. Status enum
        match &program.declarations[1] {
            Declaration::Enum(e) => {
                assert_eq!(e.name.name(), "Status");
                assert_eq!(e.variants.len(), 2);
            }
            _ => panic!("expected enum Status"),
        }

        // 3. Printable trait
        match &program.declarations[2] {
            Declaration::Trait(t) => {
                assert_eq!(t.name.name(), "Printable");
                assert_eq!(t.members.len(), 1);
            }
            _ => panic!("expected trait Printable"),
        }

        // 4. Impl Printable for User<Int>
        match &program.declarations[3] {
            Declaration::Impl(i) => {
                assert!(i.trait_type.is_some());
                assert_eq!(i.members.len(), 1);
            }
            _ => panic!("expected impl Printable for User<Int>"),
        }

        // 5. Main function
        match &program.declarations[4] {
            Declaration::Function(f) => {
                assert_eq!(f.name.name(), "main");
                assert_eq!(f.visibility, Visibility::Public);
                assert!(f.is_async);
                assert_eq!(f.generics.params.len(), 1);
                assert_eq!(f.generics.params[0].bounds[0].name.name(), "Printable");
                assert_eq!(f.parameters.len(), 1);
                assert!(f.return_type.is_some());
                assert_eq!(f.body.statements.len(), 6);
            }
            _ => panic!("expected main function"),
        }
    }
}
