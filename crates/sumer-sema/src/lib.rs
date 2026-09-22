//! Semantic analysis pipeline for the SUMER compiler.
//!
//! Provides declaration collection, symbol tables, lexical scope trees,
//! name resolution, duplicate detection, and diagnostic reporting.

pub mod analyzer;
pub mod error;
pub mod resolver;
pub mod scope;
pub mod symbol;
pub mod typeck;

pub use analyzer::{SemanticAnalyzer, SemanticResult, analyze};
pub use error::SemanticError;
pub use resolver::{BUILTIN_FUNCTIONS, BUILTIN_TYPES, Resolver};
pub use scope::{Scope, ScopeId, ScopeKind, ScopeTree};
pub use symbol::{Symbol, SymbolId, SymbolKind, SymbolTable};
pub use typeck::{TypeChecker, TypeckResult};

#[cfg(test)]
mod tests {
    use super::*;
    use sumer_lexer::Lexer;
    use sumer_parser::parse;
    use sumer_span::SourceMap;

    fn parse_source(source: &str) -> sumer_ast::Program {
        let mut source_map = SourceMap::new();
        let source_id = source_map.add("test.sm", source);
        let tokens = Lexer::new(source_id, source)
            .lex()
            .expect("lexing failed in test");
        parse(&tokens).expect("parsing failed in test")
    }

    #[test]
    fn test_symbol_kinds_and_table() {
        let mut table = SymbolTable::new();
        let span = sumer_span::Span::new(sumer_span::SourceId::new(1), 0, 5);

        let s1 = table.alloc(
            "add",
            SymbolKind::Function,
            span,
            sumer_ast::Visibility::Public,
        );
        let s2 = table.alloc(
            "User",
            SymbolKind::Struct,
            span,
            sumer_ast::Visibility::Public,
        );
        let s3 = table.alloc(
            "x",
            SymbolKind::Variable,
            span,
            sumer_ast::Visibility::Private,
        );
        let s4 = table.alloc(
            "MAX",
            SymbolKind::Constant,
            span,
            sumer_ast::Visibility::Public,
        );
        let s5 = table.alloc(
            "a",
            SymbolKind::Parameter,
            span,
            sumer_ast::Visibility::Private,
        );

        assert_eq!(table.len(), 5);
        assert_eq!(table.get(s1).unwrap().kind, SymbolKind::Function);
        assert_eq!(table.get(s2).unwrap().kind, SymbolKind::Struct);
        assert_eq!(table.get(s3).unwrap().kind, SymbolKind::Variable);
        assert_eq!(table.get(s4).unwrap().kind, SymbolKind::Constant);
        assert_eq!(table.get(s5).unwrap().kind, SymbolKind::Parameter);
        assert!(table.get(s1).unwrap().kind.is_function());
        assert!(table.get(s2).unwrap().kind.is_type());
    }

    #[test]
    fn test_scope_hierarchy_and_parent_lookup() {
        let mut scopes = ScopeTree::new();
        let global = scopes.alloc(ScopeKind::Global, None);
        let func = scopes.alloc(ScopeKind::Function, Some(global));
        let block = scopes.alloc(ScopeKind::Block, Some(func));

        let sym1 = SymbolId(1);
        let sym2 = SymbolId(2);

        assert!(scopes.define_value(global, "global_var", sym1).is_ok());
        assert!(scopes.define_value(func, "func_var", sym2).is_ok());

        // Recursive lookup from block scope
        assert_eq!(scopes.lookup_value(block, "global_var"), Some(sym1));
        assert_eq!(scopes.lookup_value(block, "func_var"), Some(sym2));
        assert_eq!(scopes.lookup_value(block, "non_existent"), None);

        // Current scope lookup
        assert_eq!(scopes.lookup_value_current(block, "global_var"), None);
        assert_eq!(scopes.lookup_value_current(func, "func_var"), Some(sym2));
    }

    #[test]
    fn test_duplicate_declarations_in_same_scope() {
        let source = r#"
fn main() {
    let name = "A"
    let name = "B"
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(result.has_errors());
        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("duplicate declaration `name`"));
    }

    #[test]
    fn test_shadowing_in_nested_scopes_allowed() {
        let source = r#"
let name = "global"

fn main() {
    let name = "local"
    {
        let name = "inner"
        print(name)
    }
    print(name)
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(!result.has_errors());
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_forward_function_reference() {
        let source = r#"
fn main() {
    hello()
}

fn hello() {
    print("Hello")
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(!result.has_errors(), "forward function call must succeed");
    }

    #[test]
    fn test_unresolved_identifier() {
        let source = r#"
fn main() {
    print(userName)
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(result.has_errors());
        let diag = result.diagnostics.iter().next().unwrap();
        assert!(
            diag.message
                .contains("cannot find `userName` in this scope")
        );
    }

    #[test]
    fn test_unresolved_type() {
        let source = r#"
fn main() {
    let user: UnknownType = 10
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(result.has_errors());
        let diag = result.diagnostics.iter().next().unwrap();
        assert!(
            diag.message
                .contains("cannot find type `UnknownType` in this scope")
        );
    }

    #[test]
    fn test_builtin_primitive_types() {
        let source = r#"
fn check_types(
    a: Bool,
    b: Int8,
    c: Int16,
    d: Int32,
    e: Int64,
    f: Int128,
    g: UInt8,
    h: UInt16,
    i: UInt32,
    j: UInt64,
    k: UInt128,
    l: Float32,
    m: Float64,
    n: Char,
    o: String,
    p: Byte,
    q: Int,
    r: UInt,
    s: Float
) -> Bool {
    return true
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(!result.has_errors());
    }

    #[test]
    fn test_function_parameters() {
        let source = r#"
fn add(a: Int, b: Int) -> Int {
    return a + b
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(!result.has_errors());
    }

    #[test]
    fn test_nested_block_scope_isolation() {
        let source = r#"
fn main() {
    let a = 1
    {
        let b = 2
        print(a)
        print(b)
    }
    print(a)
    print(b)
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(result.has_errors());
        assert_eq!(result.diagnostics.len(), 1);
        let diag = result.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("cannot find `b` in this scope"));
    }

    #[test]
    fn test_struct_enum_trait_impl_registration() {
        let source = r#"
struct User {
    id: Int
    name: String
}

enum Status {
    Active
    Inactive
}

trait Printable {
    fn print()
}

impl Printable for User {
    fn print() {
        print("User")
    }
}

fn getUser(user: User) -> Status {
    let s = Status.Active
    return s
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(!result.has_errors());
    }

    #[test]
    fn test_generic_type_and_param_resolution() {
        let source = r#"
struct Box<T> {
    value: T
}

fn wrap<T>(item: Box<T>) -> Box<T> {
    return item
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(!result.has_errors());
    }

    #[test]
    fn test_local_variable_initializer_scope() {
        // let x = 10; { let x = x; }
        // The inner RHS x must resolve to the outer x (10), not produce an unresolved error.
        let source = r#"
fn main() {
    let x = 10
    {
        let x = x
        print(x)
    }
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(!result.has_errors());

        // And if x was undeclared in outer scope:
        let invalid_source = r#"
fn main() {
    let y = y
}
"#;
        let invalid_program = parse_source(invalid_source);
        let invalid_result = analyze(&invalid_program);
        assert!(invalid_result.has_errors());
        let diag = invalid_result.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("cannot find `y` in this scope"));
    }

    #[test]
    fn test_complex_expression_traversal() {
        let source = r#"
fn calculate(val: Int) -> Int {
    let arr = [1, 2, val]
    let map = { "key": val }
    let lam = (a) => a + val
    let cond = if val > 0 { val } else { 0 }
    let matched = match val {
        1 => 10
        _ => 0
    }
    return cond + matched
}
"#;
        let program = parse_source(source);
        let result = analyze(&program);

        assert!(!result.has_errors());
    }

    #[test]
    fn test_examples_hello_and_features() {
        let hello_src = include_str!("../../../examples/hello.sm");
        let hello_prog = parse_source(hello_src);
        let hello_res = analyze(&hello_prog);
        assert!(!hello_res.has_errors(), "hello.sm must pass sema cleanly");

        let features_src = include_str!("../../../examples/features.sm");
        let features_prog = parse_source(features_src);
        let features_res = analyze(&features_prog);
        assert!(
            !features_res.has_errors(),
            "features.sm must pass sema cleanly: {:?}",
            features_res
                .diagnostics
                .iter()
                .map(|d| &d.message)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_valid_variable_type_inference() {
        let source = r#"
fn main() {
    let x = 10
    let name: String = "Monir"
    let y: Int = 10
    let result = x + y
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(!res.has_errors());
    }

    #[test]
    fn test_valid_function_calls_and_implicit_return() {
        let source = r#"
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    let result = add(10, 20)
    let age = 20
    if age > 18 {
        print("adult")
    }
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(!res.has_errors());
    }

    #[test]
    fn test_type_mismatch_variable_assignment() {
        let source = r#"
fn main() {
    let x: Int = "hello"
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(res.has_errors());
        let diag = res.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("type mismatch"));
    }

    #[test]
    fn test_cannot_apply_binary_incompatible_types() {
        let source = r#"
fn main() {
    let x = 10 + "hello"
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(res.has_errors());
        let diag = res.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("cannot apply binary operator `+`"));
    }

    #[test]
    fn test_cannot_apply_binary_booleans() {
        let source = r#"
fn main() {
    let x = true + false
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(res.has_errors());
        let diag = res.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("cannot apply binary operator `+`"));
    }

    #[test]
    fn test_argument_type_mismatch() {
        let source = r#"
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    add("10", 20)
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(res.has_errors());
        let diag = res.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("argument type mismatch"));
    }

    #[test]
    fn test_return_type_mismatch() {
        let source = r#"
fn getAge() -> Int {
    return "30"
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(res.has_errors());
        let diag = res.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("return type mismatch"));
    }

    #[test]
    fn test_condition_type_mismatch_if_and_while() {
        let source = r#"
fn main() {
    if 10 {
    }
    while "hello" {
    }
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(res.has_errors());
        assert!(
            res.diagnostics
                .iter()
                .any(|d| d.message.contains("condition"))
        );
    }

    #[test]
    fn test_array_element_type_mismatch() {
        let source = r#"
fn main() {
    let arr = [1, "two"]
}
"#;
        let prog = parse_source(source);
        let res = analyze(&prog);
        assert!(res.has_errors());
        let diag = res.diagnostics.iter().next().unwrap();
        assert!(diag.message.contains("type mismatch"));
    }
}
