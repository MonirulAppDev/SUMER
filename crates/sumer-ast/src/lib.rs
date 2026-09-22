//! Abstract Syntax Tree (AST) definitions for the SUMER programming language.
//!
//! Represents purely syntactic source constructs (functions, structs, enums,
//! statements, expressions, types, patterns) preserving source spans.

pub mod attribute;
pub mod declaration;
pub mod expression;
pub mod generics;
pub mod identifier;
pub mod pattern;
pub mod pretty;
pub mod program;
pub mod statement;
pub mod types;

pub use attribute::*;
pub use declaration::*;
pub use expression::*;
pub use generics::*;
pub use identifier::*;
pub use pattern::*;
pub use pretty::{AstPrinter, pretty_print, pretty_print_with_spans};
pub use program::*;
pub use statement::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use sumer_span::{SourceId, Span};

    fn dummy_span() -> Span {
        Span::new(SourceId(0), 0, 0)
    }

    fn ident(name: &str) -> Identifier {
        Identifier::new(name, dummy_span())
    }

    fn named_type(name: &str) -> Type {
        Type::named(TypePath::single(ident(name)), dummy_span())
    }

    // ----------------------------------------------------
    // 1. PROGRAM
    // ----------------------------------------------------
    #[test]
    fn test_program_ast() {
        let prog = Program::new(
            vec![Attribute::simple(ident("inline"), vec![], dummy_span())],
            vec![Declaration::Constant(ConstantDecl::new(
                ident("VERSION"),
                Some(named_type("String")),
                Expr::literal(Literal::String("0.1".to_string()), dummy_span()),
                Visibility::Public,
                vec![],
                dummy_span(),
            ))],
            dummy_span(),
        );
        assert_eq!(prog.attributes.len(), 1);
        assert_eq!(prog.declarations.len(), 1);
    }

    // ----------------------------------------------------
    // 2. FUNCTION & 3. FUNCTION PARAMETERS
    // ----------------------------------------------------
    #[test]
    fn test_function_declaration() {
        // fn add(a: Int, b: Int) -> Int { return a + b }
        let func = FunctionDecl::new(
            ident("add"),
            Visibility::Private,
            false,
            GenericParams::default(),
            vec![
                Parameter::new(ident("a"), named_type("Int"), None, dummy_span()),
                Parameter::new(ident("b"), named_type("Int"), None, dummy_span()),
            ],
            Some(named_type("Int")),
            Block::new(
                vec![Stmt::return_stmt(
                    Some(Expr::binary(
                        Expr::identifier(ident("a")),
                        BinaryOperator::Add,
                        Expr::identifier(ident("b")),
                        dummy_span(),
                    )),
                    dummy_span(),
                )],
                dummy_span(),
            ),
            vec![],
            dummy_span(),
        );

        assert_eq!(func.name.name, "add");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.body.statements.len(), 1);
    }

    // ----------------------------------------------------
    // 4. ASYNC FUNCTION
    // ----------------------------------------------------
    #[test]
    fn test_async_function() {
        // pub async fn fetch(url: String) -> Result<Data, Error> {}
        let ret_type = Type::generic(
            TypePath::single(ident("Result")),
            vec![named_type("Data"), named_type("Error")],
            dummy_span(),
        );

        let func = FunctionDecl::new(
            ident("fetch"),
            Visibility::Public,
            true,
            GenericParams::default(),
            vec![Parameter::new(
                ident("url"),
                named_type("String"),
                None,
                dummy_span(),
            )],
            Some(ret_type),
            Block::empty(dummy_span()),
            vec![],
            dummy_span(),
        );

        assert!(func.is_async);
        assert!(func.visibility.is_public());
    }

    // ----------------------------------------------------
    // 5. VARIABLES & 6. CONSTANTS
    // ----------------------------------------------------
    #[test]
    fn test_variable_and_constant() {
        // let name = "Monir"
        let let_var = VariableDecl::new(
            ident("name"),
            false,
            None,
            Expr::literal(Literal::String("Monir".to_string()), dummy_span()),
            Visibility::Private,
            dummy_span(),
        );
        assert!(!let_var.is_mutable);

        // var age: Int = 30
        let mut_var = VariableDecl::new(
            ident("age"),
            true,
            Some(named_type("Int")),
            Expr::literal(Literal::Integer("30".to_string()), dummy_span()),
            Visibility::Private,
            dummy_span(),
        );
        assert!(mut_var.is_mutable);

        // const MAX_USERS = 1000
        let const_decl = ConstantDecl::new(
            ident("MAX_USERS"),
            None,
            Expr::literal(Literal::Integer("1000".to_string()), dummy_span()),
            Visibility::Public,
            vec![],
            dummy_span(),
        );
        assert_eq!(const_decl.name.name, "MAX_USERS");
    }

    // ----------------------------------------------------
    // 7. STRUCT & 8. STRUCT METHOD
    // ----------------------------------------------------
    #[test]
    fn test_struct_with_methods() {
        // struct User { name: String, fn greet() { print(self.name) } }
        let struct_decl = StructDecl::new(
            ident("User"),
            Visibility::Public,
            GenericParams::default(),
            vec![
                StructMember::Field(FieldDecl::new(
                    ident("name"),
                    Visibility::Private,
                    named_type("String"),
                    None,
                    vec![],
                    dummy_span(),
                )),
                StructMember::Method(FunctionDecl::new(
                    ident("greet"),
                    Visibility::Public,
                    false,
                    GenericParams::default(),
                    vec![],
                    None,
                    Block::new(
                        vec![Stmt::expr(Expr::call(
                            Expr::identifier(ident("print")),
                            vec![Argument::positional(Expr::member(
                                Expr::identifier(ident("self")),
                                ident("name"),
                                dummy_span(),
                            ))],
                            dummy_span(),
                        ))],
                        dummy_span(),
                    ),
                    vec![],
                    dummy_span(),
                )),
            ],
            vec![],
            dummy_span(),
        );

        assert_eq!(struct_decl.name.name, "User");
        assert_eq!(struct_decl.members.len(), 2);
    }

    // ----------------------------------------------------
    // 9. ENUM & 10. ENUM DATA
    // ----------------------------------------------------
    #[test]
    fn test_enum_data() {
        // enum Payment { Cash, Card(number: String), Mobile(provider: String) }
        let enum_decl = EnumDecl::new(
            ident("Payment"),
            Visibility::Public,
            GenericParams::default(),
            vec![
                EnumVariant::new(ident("Cash"), VariantData::Unit, dummy_span()),
                EnumVariant::new(
                    ident("Card"),
                    VariantData::Struct(vec![FieldDecl::new(
                        ident("number"),
                        Visibility::Private,
                        named_type("String"),
                        None,
                        vec![],
                        dummy_span(),
                    )]),
                    dummy_span(),
                ),
            ],
            vec![],
            dummy_span(),
        );

        assert_eq!(enum_decl.variants.len(), 2);
    }

    // ----------------------------------------------------
    // 11. TRAIT & 12. IMPL
    // ----------------------------------------------------
    #[test]
    fn test_trait_and_impl() {
        // trait Printable { fn print() }
        let trait_decl = TraitDecl::new(
            ident("Printable"),
            Visibility::Public,
            GenericParams::default(),
            vec![],
            vec![TraitMember::FunctionSignature(FunctionSignature::new(
                ident("print"),
                false,
                GenericParams::default(),
                vec![],
                None,
                dummy_span(),
            ))],
            vec![],
            dummy_span(),
        );

        // impl Printable for User { fn print() {} }
        let impl_decl = ImplDecl::new(
            GenericParams::default(),
            Some(named_type("Printable")),
            named_type("User"),
            vec![FunctionDecl::new(
                ident("print"),
                Visibility::Public,
                false,
                GenericParams::default(),
                vec![],
                None,
                Block::empty(dummy_span()),
                vec![],
                dummy_span(),
            )],
            vec![],
            dummy_span(),
        );

        assert_eq!(trait_decl.name.name, "Printable");
        assert!(impl_decl.trait_type.is_some());
    }

    // ----------------------------------------------------
    // 13. IMPORT
    // ----------------------------------------------------
    #[test]
    fn test_import() {
        // import user.User
        let imp = ImportDecl::new(
            ModulePath::new(vec![ident("user"), ident("User")], dummy_span()),
            Visibility::Private,
            dummy_span(),
        );
        assert_eq!(imp.path.segments.len(), 2);
    }

    // ----------------------------------------------------
    // 14. IF & 15. IF EXPRESSION
    // ----------------------------------------------------
    #[test]
    fn test_if_statement_and_expression() {
        // if cond { 1 } else { 2 }
        let if_stmt = Stmt::new(
            StmtKind::If(IfStmt::new(
                Expr::identifier(ident("cond")),
                Block::empty(dummy_span()),
                Some(ElseStmtBranch::Block(Block::empty(dummy_span()))),
                dummy_span(),
            )),
            dummy_span(),
        );

        let if_expr = Expr::new(
            ExprKind::If(IfExpr::new(
                Expr::identifier(ident("cond")),
                Block::empty(dummy_span()),
                Some(ElseExprBranch::Block(Block::empty(dummy_span()))),
                dummy_span(),
            )),
            dummy_span(),
        );

        assert!(matches!(if_stmt.kind, StmtKind::If(_)));
        assert!(matches!(if_expr.kind, ExprKind::If(_)));
    }

    // ----------------------------------------------------
    // 16. WHILE, 17. FOR, 18. LOOP
    // ----------------------------------------------------
    #[test]
    fn test_loops() {
        let while_stmt = Stmt::new(
            StmtKind::While(WhileStmt::new(
                Expr::identifier(ident("cond")),
                Block::empty(dummy_span()),
                dummy_span(),
            )),
            dummy_span(),
        );

        let for_stmt = Stmt::new(
            StmtKind::For(ForStmt::new(
                ident("item"),
                Expr::identifier(ident("items")),
                Block::empty(dummy_span()),
                dummy_span(),
            )),
            dummy_span(),
        );

        let loop_stmt = Stmt::new(
            StmtKind::Loop(LoopStmt::new(Block::empty(dummy_span()), dummy_span())),
            dummy_span(),
        );

        assert!(matches!(while_stmt.kind, StmtKind::While(_)));
        assert!(matches!(for_stmt.kind, StmtKind::For(_)));
        assert!(matches!(loop_stmt.kind, StmtKind::Loop(_)));
    }

    // ----------------------------------------------------
    // 19. MATCH, 20. BREAK / CONTINUE
    // ----------------------------------------------------
    #[test]
    fn test_match_break_continue() {
        let match_stmt = Stmt::new(
            StmtKind::Match(MatchStmt::new(
                Expr::identifier(ident("val")),
                vec![MatchArm::new(
                    Pattern::wildcard(dummy_span()),
                    Expr::literal(Literal::Integer("0".to_string()), dummy_span()),
                    dummy_span(),
                )],
                dummy_span(),
            )),
            dummy_span(),
        );

        let break_stmt = Stmt::break_stmt(dummy_span());
        let cont_stmt = Stmt::continue_stmt(dummy_span());

        assert!(matches!(match_stmt.kind, StmtKind::Match(_)));
        assert!(matches!(break_stmt.kind, StmtKind::Break(_)));
        assert!(matches!(cont_stmt.kind, StmtKind::Continue(_)));
    }

    // ----------------------------------------------------
    // 21. CALL, 22. MEMBER, 23. OPTIONAL MEMBER, 24. INDEX
    // ----------------------------------------------------
    #[test]
    fn test_call_member_index() {
        // createUser(name: "Monir", age: 30)
        let call = Expr::call(
            Expr::identifier(ident("createUser")),
            vec![
                Argument::named(
                    ident("name"),
                    Expr::literal(Literal::String("Monir".to_string()), dummy_span()),
                    dummy_span(),
                ),
                Argument::named(
                    ident("age"),
                    Expr::literal(Literal::Integer("30".to_string()), dummy_span()),
                    dummy_span(),
                ),
            ],
            dummy_span(),
        );

        // user.name
        let member = Expr::member(Expr::identifier(ident("user")), ident("name"), dummy_span());

        // user?.name
        let opt_member =
            Expr::optional_member(Expr::identifier(ident("user")), ident("name"), dummy_span());

        // items[0]
        let idx = Expr::index(
            Expr::identifier(ident("items")),
            Expr::literal(Literal::Integer("0".to_string()), dummy_span()),
            dummy_span(),
        );

        assert!(matches!(call.kind, ExprKind::Call { .. }));
        assert!(matches!(member.kind, ExprKind::Member { .. }));
        assert!(matches!(opt_member.kind, ExprKind::OptionalMember { .. }));
        assert!(matches!(idx.kind, ExprKind::Index { .. }));
    }

    // ----------------------------------------------------
    // 25. STRUCT INIT, 26. ARRAY, 27. MAP
    // ----------------------------------------------------
    #[test]
    fn test_struct_init_array_map() {
        // User { id: 1, name: "Monir" }
        let struct_init = Expr::new(
            ExprKind::StructInit {
                name: TypePath::single(ident("User")),
                fields: vec![
                    FieldInit::new(
                        ident("id"),
                        Expr::literal(Literal::Integer("1".to_string()), dummy_span()),
                        dummy_span(),
                    ),
                    FieldInit::new(
                        ident("name"),
                        Expr::literal(Literal::String("Monir".to_string()), dummy_span()),
                        dummy_span(),
                    ),
                ],
            },
            dummy_span(),
        );

        // [1, 2, 3]
        let arr = Expr::new(
            ExprKind::Array(vec![
                Expr::literal(Literal::Integer("1".to_string()), dummy_span()),
                Expr::literal(Literal::Integer("2".to_string()), dummy_span()),
            ]),
            dummy_span(),
        );

        // {"name": "Monir"}
        let map = Expr::new(
            ExprKind::Map(vec![MapEntry::new(
                Expr::literal(Literal::String("name".to_string()), dummy_span()),
                Expr::literal(Literal::String("Monir".to_string()), dummy_span()),
                dummy_span(),
            )]),
            dummy_span(),
        );

        assert!(matches!(struct_init.kind, ExprKind::StructInit { .. }));
        assert!(matches!(arr.kind, ExprKind::Array(_)));
        assert!(matches!(map.kind, ExprKind::Map(_)));
    }

    // ----------------------------------------------------
    // 28. LAMBDA, 29. AWAIT, 30. SPAWN, 31. REFERENCE, 32. FORCE UNWRAP
    // ----------------------------------------------------
    #[test]
    fn test_lambda_async_ref_force_unwrap() {
        // (a, b) => a + b
        let lambda = Expr::new(
            ExprKind::Lambda {
                parameters: vec![ident("a"), ident("b")],
                body: Box::new(Expr::binary(
                    Expr::identifier(ident("a")),
                    BinaryOperator::Add,
                    Expr::identifier(ident("b")),
                    dummy_span(),
                )),
            },
            dummy_span(),
        );

        // await task
        let await_e = Expr::await_expr(Expr::identifier(ident("task")), dummy_span());

        // spawn download(url)
        let spawn_e = Expr::spawn_expr(
            Expr::call(
                Expr::identifier(ident("download")),
                vec![Argument::positional(Expr::identifier(ident("url")))],
                dummy_span(),
            ),
            dummy_span(),
        );

        // &value, &mut value
        let ref_e = Expr::new(
            ExprKind::Reference {
                mutable: false,
                operand: Box::new(Expr::identifier(ident("value"))),
            },
            dummy_span(),
        );
        let mut_ref_e = Expr::new(
            ExprKind::Reference {
                mutable: true,
                operand: Box::new(Expr::identifier(ident("value"))),
            },
            dummy_span(),
        );

        // user!
        let unwrap = Expr::force_unwrap(Expr::identifier(ident("user")), dummy_span());

        assert!(matches!(lambda.kind, ExprKind::Lambda { .. }));
        assert!(matches!(await_e.kind, ExprKind::Await(_)));
        assert!(matches!(spawn_e.kind, ExprKind::Spawn(_)));
        assert!(matches!(
            ref_e.kind,
            ExprKind::Reference { mutable: false, .. }
        ));
        assert!(matches!(
            mut_ref_e.kind,
            ExprKind::Reference { mutable: true, .. }
        ));
        assert!(matches!(unwrap.kind, ExprKind::ForceUnwrap(_)));
    }

    // ----------------------------------------------------
    // 33. GENERICS & 34. PATTERNS
    // ----------------------------------------------------
    #[test]
    fn test_generics_and_patterns() {
        // fn max<T: Comparable>(a: T, b: T) -> T
        let generics = GenericParams::new(
            vec![GenericParam::new(
                ident("T"),
                vec![TypeBound::new(ident("Comparable"), dummy_span())],
                dummy_span(),
            )],
            dummy_span(),
        );
        assert_eq!(generics.params.len(), 1);

        // Patterns: _, name, 10, Payment.Cash, Payment.Card(number), User { name }
        let p_wildcard = Pattern::wildcard(dummy_span());
        let p_ident = Pattern::identifier(ident("x"));
        let p_lit = Pattern::literal(Literal::Integer("10".to_string()), dummy_span());
        let p_enum = Pattern::enum_variant(
            TypePath::single(ident("Payment")),
            ident("Card"),
            Some(vec![Pattern::identifier(ident("number"))]),
            dummy_span(),
        );
        let p_struct = Pattern::struct_pattern(
            TypePath::single(ident("User")),
            vec![FieldPattern::new(ident("name"), None, dummy_span())],
            dummy_span(),
        );
        let p_tuple = Pattern::tuple(
            vec![
                Pattern::identifier(ident("a")),
                Pattern::identifier(ident("b")),
            ],
            dummy_span(),
        );

        assert!(matches!(p_wildcard.kind, PatternKind::Wildcard));
        assert!(matches!(p_ident.kind, PatternKind::Identifier(_)));
        assert!(matches!(p_lit.kind, PatternKind::Literal(_)));
        assert!(matches!(p_enum.kind, PatternKind::Enum { .. }));
        assert!(matches!(p_struct.kind, PatternKind::Struct { .. }));
        assert!(matches!(p_tuple.kind, PatternKind::Tuple(_)));
    }

    // ----------------------------------------------------
    // MANUAL VERIFICATION: hello.sm
    // ----------------------------------------------------
    #[test]
    fn test_hello_sm_manual_ast_construction() {
        // fn main() {
        //     print("Hello, SUMER!")
        // }
        let call_expr = Expr::call(
            Expr::identifier(ident("print")),
            vec![Argument::positional(Expr::literal(
                Literal::String("Hello, SUMER!".to_string()),
                dummy_span(),
            ))],
            dummy_span(),
        );

        let body_block = Block::new(vec![Stmt::expr(call_expr)], dummy_span());

        let func_main = FunctionDecl::new(
            ident("main"),
            Visibility::Private,
            false,
            GenericParams::default(),
            vec![],
            None,
            body_block,
            vec![],
            dummy_span(),
        );

        let program = Program::new(vec![], vec![Declaration::Function(func_main)], dummy_span());

        assert_eq!(program.declarations.len(), 1);
        if let Declaration::Function(f) = &program.declarations[0] {
            assert_eq!(f.name.name, "main");
            assert_eq!(f.body.statements.len(), 1);
        } else {
            panic!("expected Function declaration");
        }
    }

    // ----------------------------------------------------
    // 8. AST PRETTY PRINTER TESTS
    // ----------------------------------------------------
    #[test]
    fn test_pretty_print_hello_world() {
        let call_expr = Expr::call(
            Expr::identifier(ident("print")),
            vec![Argument::positional(Expr::literal(
                Literal::String("Hello, SUMER!".to_string()),
                dummy_span(),
            ))],
            dummy_span(),
        );

        let body_block = Block::new(vec![Stmt::expr(call_expr)], dummy_span());

        let func_main = FunctionDecl::new(
            ident("main"),
            Visibility::Private,
            false,
            GenericParams::default(),
            vec![],
            None,
            body_block,
            vec![],
            dummy_span(),
        );

        let program = Program::new(vec![], vec![Declaration::Function(func_main)], dummy_span());
        let output = pretty_print(&program);

        let expected = "\
Program
└── Function: main
    ├── Visibility: Private
    ├── Async: false
    ├── Parameters: 0
    └── Body
        └── ExpressionStatement
            └── Call
                ├── Function
                │   └── Identifier: print
                └── Arguments
                    └── String: \"Hello, SUMER!\"
";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_pretty_print_binary_precedence() {
        // a + b * c
        let mul = Expr::binary(
            Expr::identifier(ident("b")),
            BinaryOperator::Multiply,
            Expr::identifier(ident("c")),
            dummy_span(),
        );
        let add = Expr::binary(
            Expr::identifier(ident("a")),
            BinaryOperator::Add,
            mul,
            dummy_span(),
        );
        let func = FunctionDecl::new(
            ident("calc"),
            Visibility::Public,
            false,
            GenericParams::default(),
            vec![],
            None,
            Block::new(vec![Stmt::expr(add)], dummy_span()),
            vec![],
            dummy_span(),
        );
        let program = Program::new(vec![], vec![Declaration::Function(func)], dummy_span());
        let output = pretty_print(&program);

        assert!(output.contains("Binary: +"));
        assert!(output.contains("├── Identifier: a"));
        assert!(output.contains("└── Binary: *"));
        assert!(output.contains("├── Identifier: b"));
        assert!(output.contains("└── Identifier: c"));
    }

    #[test]
    fn test_pretty_print_spans() {
        let span1 = Span::new(SourceId(0), 0, 37);
        let func = FunctionDecl::new(
            ident("main"),
            Visibility::Private,
            false,
            GenericParams::default(),
            vec![],
            None,
            Block::new(vec![], dummy_span()),
            vec![],
            span1,
        );
        let program = Program::new(vec![], vec![Declaration::Function(func)], span1);
        let output = pretty_print_with_spans(&program);

        assert!(output.contains("Program [0..37]"));
        assert!(output.contains("Function: main [0..37]"));
    }

    #[test]
    fn test_pretty_print_declarations_and_patterns() {
        let field = FieldDecl::new(
            ident("id"),
            Visibility::Public,
            named_type("Int"),
            None,
            vec![],
            dummy_span(),
        );
        let struct_decl = StructDecl::new(
            ident("User"),
            Visibility::Public,
            GenericParams::default(),
            vec![StructMember::Field(field)],
            vec![Attribute::simple(
                ident("derive"),
                vec![ident("Debug")],
                dummy_span(),
            )],
            dummy_span(),
        );

        let variant_active = EnumVariant::new(ident("Active"), VariantData::Unit, dummy_span());
        let variant_card = EnumVariant::new(
            ident("Card"),
            VariantData::Tuple(vec![named_type("String")]),
            dummy_span(),
        );
        let enum_decl = EnumDecl::new(
            ident("Status"),
            Visibility::Private,
            GenericParams::default(),
            vec![variant_active, variant_card],
            vec![],
            dummy_span(),
        );

        let var_decl = VariableDecl::new(
            ident("age"),
            false,
            Some(named_type("Int")),
            Expr::literal(Literal::Integer("30".to_string()), dummy_span()),
            Visibility::Private,
            dummy_span(),
        );

        let program = Program::new(
            vec![],
            vec![
                Declaration::Struct(struct_decl),
                Declaration::Enum(enum_decl),
                Declaration::Variable(var_decl),
            ],
            dummy_span(),
        );

        let output = pretty_print(&program);
        assert!(output.contains("Struct: User"));
        assert!(output.contains("Field: id : Int"));
        assert!(output.contains("Enum: Status"));
        assert!(output.contains("Variant: Active"));
        assert!(output.contains("Variant: Card(String)"));
        assert!(output.contains("VariableDecl: let age: Int"));
        assert!(output.contains("Integer: 30"));
    }
}
