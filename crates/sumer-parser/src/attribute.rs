//! Attribute parsing (e.g. `@inline`, `@derive(Debug, Json)`).

use sumer_ast::{Attribute, AttributeArg};
use sumer_lexer::TokenKind;

use crate::cursor::TokenCursor;
use crate::error::ParseResult;

/// Parses an attribute starting with `@`.
pub fn parse_attribute(cursor: &mut TokenCursor<'_>) -> ParseResult<Attribute> {
    let at_tok = cursor.expect(&TokenKind::At)?;
    let start = at_tok.span();

    let name = cursor.parse_identifier()?;
    let mut args = Vec::new();
    let mut end = name.span();

    if cursor.match_token(&TokenKind::LParen) {
        if !cursor.check(&TokenKind::RParen) {
            args.push(parse_attribute_arg(cursor)?);
            while cursor.match_token(&TokenKind::Comma) {
                if cursor.check(&TokenKind::RParen) {
                    break;
                }
                args.push(parse_attribute_arg(cursor)?);
            }
        }
        let rparen = cursor.expect(&TokenKind::RParen)?;
        end = rparen.span();
    }

    let span = start.join(end).unwrap_or(start);
    Ok(Attribute::new(name, args, span))
}

fn parse_attribute_arg(cursor: &mut TokenCursor<'_>) -> ParseResult<AttributeArg> {
    let tok = cursor.current();
    match tok.kind() {
        TokenKind::String(val) => {
            let s = val.clone();
            cursor.advance();
            Ok(AttributeArg::String(s))
        }
        _ => {
            let ident = cursor.parse_identifier()?;
            Ok(AttributeArg::Identifier(ident))
        }
    }
}

/// Parses zero or more leading attributes.
pub fn parse_attributes(cursor: &mut TokenCursor<'_>) -> ParseResult<Vec<Attribute>> {
    let mut attributes = Vec::new();
    while cursor.check(&TokenKind::At) {
        attributes.push(parse_attribute(cursor)?);
    }
    Ok(attributes)
}
