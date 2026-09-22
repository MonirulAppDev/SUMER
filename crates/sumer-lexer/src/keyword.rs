//! Keyword recognition table for SUMER v0.1.

use crate::token::TokenKind;

/// Looks up an identifier string to determine if it is a reserved SUMER keyword.
///
/// Returns `Some(TokenKind)` for reserved keywords, or `None` if it is a user identifier.
pub fn lookup_keyword(ident: &str) -> Option<TokenKind> {
    match ident {
        // Variable & Constant declarations
        "let" => Some(TokenKind::Let),
        "var" => Some(TokenKind::Var),
        "const" => Some(TokenKind::Const),

        // Functions
        "fn" => Some(TokenKind::Fn),
        "return" => Some(TokenKind::Return),

        // Types & OOP/FP constructs
        "struct" => Some(TokenKind::Struct),
        "enum" => Some(TokenKind::Enum),
        "trait" => Some(TokenKind::Trait),
        "impl" => Some(TokenKind::Impl),

        // Control flow
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "for" => Some(TokenKind::For),
        "in" => Some(TokenKind::In),
        "while" => Some(TokenKind::While),
        "loop" => Some(TokenKind::Loop),
        "match" => Some(TokenKind::Match),
        "break" => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue),

        // Modules & Visibility
        "import" => Some(TokenKind::Import),
        "pub" => Some(TokenKind::Pub),

        // Concurrency & Async
        "async" => Some(TokenKind::Async),
        "await" => Some(TokenKind::Await),
        "spawn" => Some(TokenKind::Spawn),

        // Low-level & Safety
        "unsafe" => Some(TokenKind::Unsafe),

        // Boolean literals
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),

        // Option & Result built-in variants
        "Some" => Some(TokenKind::Some),
        "None" => Some(TokenKind::None),
        "Ok" => Some(TokenKind::Ok),
        "Err" => Some(TokenKind::Err),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_documented_keywords() {
        assert_eq!(lookup_keyword("let"), Some(TokenKind::Let));
        assert_eq!(lookup_keyword("var"), Some(TokenKind::Var));
        assert_eq!(lookup_keyword("const"), Some(TokenKind::Const));
        assert_eq!(lookup_keyword("fn"), Some(TokenKind::Fn));
        assert_eq!(lookup_keyword("return"), Some(TokenKind::Return));
        assert_eq!(lookup_keyword("struct"), Some(TokenKind::Struct));
        assert_eq!(lookup_keyword("enum"), Some(TokenKind::Enum));
        assert_eq!(lookup_keyword("trait"), Some(TokenKind::Trait));
        assert_eq!(lookup_keyword("impl"), Some(TokenKind::Impl));
        assert_eq!(lookup_keyword("if"), Some(TokenKind::If));
        assert_eq!(lookup_keyword("else"), Some(TokenKind::Else));
        assert_eq!(lookup_keyword("for"), Some(TokenKind::For));
        assert_eq!(lookup_keyword("in"), Some(TokenKind::In));
        assert_eq!(lookup_keyword("while"), Some(TokenKind::While));
        assert_eq!(lookup_keyword("loop"), Some(TokenKind::Loop));
        assert_eq!(lookup_keyword("match"), Some(TokenKind::Match));
        assert_eq!(lookup_keyword("break"), Some(TokenKind::Break));
        assert_eq!(lookup_keyword("continue"), Some(TokenKind::Continue));
        assert_eq!(lookup_keyword("import"), Some(TokenKind::Import));
        assert_eq!(lookup_keyword("pub"), Some(TokenKind::Pub));
        assert_eq!(lookup_keyword("async"), Some(TokenKind::Async));
        assert_eq!(lookup_keyword("await"), Some(TokenKind::Await));
        assert_eq!(lookup_keyword("spawn"), Some(TokenKind::Spawn));
        assert_eq!(lookup_keyword("unsafe"), Some(TokenKind::Unsafe));
        assert_eq!(lookup_keyword("true"), Some(TokenKind::True));
        assert_eq!(lookup_keyword("false"), Some(TokenKind::False));
        assert_eq!(lookup_keyword("Some"), Some(TokenKind::Some));
        assert_eq!(lookup_keyword("None"), Some(TokenKind::None));
        assert_eq!(lookup_keyword("Ok"), Some(TokenKind::Ok));
        assert_eq!(lookup_keyword("Err"), Some(TokenKind::Err));
    }

    #[test]
    fn test_non_keywords_return_none() {
        assert_eq!(lookup_keyword("user"), None);
        assert_eq!(lookup_keyword("main"), None);
        assert_eq!(lookup_keyword("calculate"), None);
        assert_eq!(lookup_keyword("User"), None);
        assert_eq!(lookup_keyword("Option"), None);
    }
}
