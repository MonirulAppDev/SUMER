//! Type expression parsing.

use sumer_ast::{Type, TypePath};
use sumer_lexer::TokenKind;

use crate::cursor::TokenCursor;
use crate::error::{ParseError, ParseErrorKind, ParseResult};

/// Parses a type expression from the token cursor.
///
/// Supports:
/// - Named types (e.g. `Int`, `String`, `User`)
/// - Qualified paths (e.g. `user.User`)
/// - Generic types (e.g. `List<Int>`, `Result<User, Error>`)
/// - Reference types (e.g. `&T`, `&mut T`)
/// - Array types (e.g. `[T]`)
/// - Optional types (e.g. `T?`)
/// - Function types (e.g. `fn(Int, String) -> Bool`)
/// - Tuple types (e.g. `(Int, String)`)
pub fn parse_type(cursor: &mut TokenCursor<'_>) -> ParseResult<Type> {
    let mut base = match cursor.current().kind() {
        TokenKind::AmpMut => {
            let start = cursor.advance().span();
            let inner = parse_type(cursor)?;
            let span = start.join(inner.span).unwrap_or(start);
            Type::mut_reference(inner, span)
        }
        TokenKind::Ampersand => {
            let start = cursor.advance().span();
            let inner = parse_type(cursor)?;
            let span = start.join(inner.span).unwrap_or(start);
            Type::reference(inner, span)
        }
        TokenKind::LBracket => {
            let start = cursor.advance().span();
            let element = parse_type(cursor)?;
            let size = if cursor.match_token(&TokenKind::Semicolon) {
                Some(crate::expression::parse_expression(cursor)?)
            } else {
                None
            };
            let rbracket = cursor.expect(&TokenKind::RBracket)?;
            let span = start.join(rbracket.span()).unwrap_or(start);
            Type::array(element, size, span)
        }
        TokenKind::LParen => {
            let start = cursor.advance().span();
            if cursor.match_token(&TokenKind::RParen) {
                let span = start.join(cursor.previous().span()).unwrap_or(start);
                Type::tuple(vec![], span)
            } else {
                let mut elements = vec![parse_type(cursor)?];
                while cursor.match_token(&TokenKind::Comma) {
                    if cursor.check(&TokenKind::RParen) {
                        break;
                    }
                    elements.push(parse_type(cursor)?);
                }
                let rparen = cursor.expect(&TokenKind::RParen)?;
                let span = start.join(rparen.span()).unwrap_or(start);
                if elements.len() == 1 {
                    elements.remove(0)
                } else {
                    Type::tuple(elements, span)
                }
            }
        }
        TokenKind::Fn => {
            let start = cursor.advance().span();
            cursor.expect(&TokenKind::LParen)?;
            let mut params = Vec::new();
            if !cursor.check(&TokenKind::RParen) {
                params.push(parse_type(cursor)?);
                while cursor.match_token(&TokenKind::Comma) {
                    if cursor.check(&TokenKind::RParen) {
                        break;
                    }
                    params.push(parse_type(cursor)?);
                }
            }
            cursor.expect(&TokenKind::RParen)?;
            cursor.expect(&TokenKind::Arrow)?;
            let ret = parse_type(cursor)?;
            let span = start.join(ret.span).unwrap_or(start);
            Type::function(params, ret, span)
        }
        TokenKind::Identifier(_)
        | TokenKind::Some
        | TokenKind::None
        | TokenKind::Ok
        | TokenKind::Err => {
            let first = cursor.parse_identifier()?;
            let mut segments = vec![first.clone()];
            let mut span = first.span();

            while cursor.match_token(&TokenKind::Dot) {
                let next = cursor.parse_identifier()?;
                span = span.join(next.span()).unwrap_or(span);
                segments.push(next);
            }

            let path = TypePath::new(segments, span);

            // Generic arguments: `<T, U>`
            if cursor.match_token(&TokenKind::Less) {
                let mut args = Vec::new();
                if !cursor.check(&TokenKind::Greater) {
                    args.push(parse_type(cursor)?);
                    while cursor.match_token(&TokenKind::Comma) {
                        if cursor.check(&TokenKind::Greater) {
                            break;
                        }
                        args.push(parse_type(cursor)?);
                    }
                }
                let end_tok = cursor.expect(&TokenKind::Greater)?;
                let full_span = span.join(end_tok.span()).unwrap_or(span);
                Type::generic(path, args, full_span)
            } else {
                Type::named(path, span)
            }
        }
        _ => {
            let tok = cursor.current();
            return Err(ParseError::new(
                ParseErrorKind::InvalidType(format!("unexpected token {}", tok.kind())),
                format!("expected type, found {}", tok.kind()),
                tok.span(),
            ));
        }
    };

    // Suffix: `?` optional
    if cursor.match_token(&TokenKind::Question) {
        let span = base
            .span
            .join(cursor.previous().span())
            .unwrap_or(base.span);
        base = Type::optional(base, span);
    }

    Ok(base)
}
