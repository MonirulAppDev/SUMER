//! Pattern parsing for match expressions, match statements, and destructuring bindings.

use sumer_ast::{FieldPattern, Literal, MatchArm, Pattern, TypePath};
use sumer_lexer::TokenKind;

use crate::cursor::TokenCursor;
use crate::error::{ParseError, ParseResult};
use crate::expression::parse_expression;

/// Parses a pattern.
pub fn parse_pattern(cursor: &mut TokenCursor<'_>) -> ParseResult<Pattern> {
    let tok = cursor.current();
    match tok.kind() {
        // --- Wildcard `_` or Identifier / Enum / Struct pattern ---
        TokenKind::Identifier(name) if name == "_" => {
            let span = tok.span();
            cursor.advance();
            Ok(Pattern::wildcard(span))
        }
        TokenKind::Identifier(_) | TokenKind::Some | TokenKind::Ok | TokenKind::Err => {
            let first_ident = cursor.parse_identifier()?;

            // Check for path segments: `Path.Variant` or `Path.SubPath.Variant`
            if cursor.check(&TokenKind::Dot) {
                let mut segments = vec![first_ident];
                while cursor.match_token(&TokenKind::Dot) {
                    segments.push(cursor.parse_identifier()?);
                }

                // The last segment is the enum variant, earlier segments form the type path
                let variant = segments.pop().unwrap();
                let first_span = segments[0].span();
                let last_path_span = segments.last().map(|s| s.span()).unwrap_or(first_span);
                let path_span = first_span.join(last_path_span).unwrap_or(first_span);
                let path = TypePath::new(segments, path_span);

                // Check for tuple payload: `Payment.Card(number)`
                if cursor.match_token(&TokenKind::LParen) {
                    let mut data = Vec::new();
                    if !cursor.check(&TokenKind::RParen) {
                        data.push(parse_pattern(cursor)?);
                        while cursor.match_token(&TokenKind::Comma) {
                            if cursor.check(&TokenKind::RParen) {
                                break;
                            }
                            data.push(parse_pattern(cursor)?);
                        }
                    }
                    let rparen = cursor.expect(&TokenKind::RParen)?;
                    let span = path.span.join(rparen.span()).unwrap_or(path.span);
                    return Ok(Pattern::enum_variant(path, variant, Some(data), span));
                }

                let span = path.span.join(variant.span()).unwrap_or(path.span);
                return Ok(Pattern::enum_variant(path, variant, None, span));
            }

            // Check for single-word enum payload: `Some(value)`
            if cursor.match_token(&TokenKind::LParen) {
                let mut data = Vec::new();
                if !cursor.check(&TokenKind::RParen) {
                    data.push(parse_pattern(cursor)?);
                    while cursor.match_token(&TokenKind::Comma) {
                        if cursor.check(&TokenKind::RParen) {
                            break;
                        }
                        data.push(parse_pattern(cursor)?);
                    }
                }
                let rparen = cursor.expect(&TokenKind::RParen)?;
                let span = first_ident
                    .span()
                    .join(rparen.span())
                    .unwrap_or(first_ident.span());
                let empty_path = TypePath::new(Vec::new(), first_ident.span());
                return Ok(Pattern::enum_variant(
                    empty_path,
                    first_ident,
                    Some(data),
                    span,
                ));
            }

            // Check for struct pattern: `User { name }` or `User { name: userName }`
            if cursor.match_token(&TokenKind::LBrace) {
                let path = TypePath::single(first_ident);
                let mut fields = Vec::new();
                while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
                    let f_name = cursor.parse_identifier()?;
                    let (sub_pat, f_span) = if cursor.match_token(&TokenKind::Colon) {
                        let sub = parse_pattern(cursor)?;
                        let span = f_name.span().join(sub.span).unwrap_or(f_name.span());
                        (Some(sub), span)
                    } else {
                        (None, f_name.span())
                    };
                    fields.push(FieldPattern::new(f_name, sub_pat, f_span));
                    cursor.match_token(&TokenKind::Comma);
                }
                let rbrace = cursor.expect(&TokenKind::RBrace)?;
                let span = path.span.join(rbrace.span()).unwrap_or(path.span);
                return Ok(Pattern::struct_pattern(path, fields, span));
            }

            // Standalone identifier pattern: `x`, `count`
            Ok(Pattern::identifier(first_ident))
        }

        // --- Tuple pattern `(a, b)` ---
        TokenKind::LParen => {
            let lparen = cursor.advance();
            let mut elements = Vec::new();
            if !cursor.check(&TokenKind::RParen) {
                elements.push(parse_pattern(cursor)?);
                while cursor.match_token(&TokenKind::Comma) {
                    if cursor.check(&TokenKind::RParen) {
                        break;
                    }
                    elements.push(parse_pattern(cursor)?);
                }
            }
            let rparen = cursor.expect(&TokenKind::RParen)?;
            let span = lparen.span().join(rparen.span()).unwrap_or(lparen.span());
            Ok(Pattern::tuple(elements, span))
        }

        // --- Literal patterns ---
        TokenKind::Integer(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Ok(Pattern::literal(Literal::Integer(val), span))
        }
        TokenKind::Float(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Ok(Pattern::literal(Literal::Float(val), span))
        }
        TokenKind::String(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Ok(Pattern::literal(Literal::String(val), span))
        }
        TokenKind::Char(c) => {
            let c = *c;
            let span = tok.span();
            cursor.advance();
            Ok(Pattern::literal(Literal::Char(c), span))
        }
        TokenKind::True => {
            let span = tok.span();
            cursor.advance();
            Ok(Pattern::literal(Literal::Bool(true), span))
        }
        TokenKind::False => {
            let span = tok.span();
            cursor.advance();
            Ok(Pattern::literal(Literal::Bool(false), span))
        }
        TokenKind::None => {
            let span = tok.span();
            cursor.advance();
            Ok(Pattern::literal(Literal::None, span))
        }

        // --- Negative literal pattern: `-10` ---
        TokenKind::Minus => {
            let start = cursor.advance().span();
            let next_tok = cursor.current();
            match next_tok.kind() {
                TokenKind::Integer(val) => {
                    let full_val = format!("-{val}");
                    let span = start.join(next_tok.span()).unwrap_or(start);
                    cursor.advance();
                    Ok(Pattern::literal(Literal::Integer(full_val), span))
                }
                TokenKind::Float(val) => {
                    let full_val = format!("-{val}");
                    let span = start.join(next_tok.span()).unwrap_or(start);
                    cursor.advance();
                    Ok(Pattern::literal(Literal::Float(full_val), span))
                }
                _ => Err(ParseError::unexpected_token(
                    "literal number after '-' in pattern",
                    next_tok,
                )),
            }
        }

        _ => Err(ParseError::unexpected_token("pattern", tok)),
    }
}

/// Parses an arm within a match statement or match expression: `pattern => expression`.
pub fn parse_match_arm(cursor: &mut TokenCursor<'_>) -> ParseResult<MatchArm> {
    let pattern = parse_pattern(cursor)?;
    cursor.expect(&TokenKind::FatArrow)?;
    let body = parse_expression(cursor)?;
    cursor.match_token(&TokenKind::Comma);
    cursor.consume_semicolon_if_present();

    let span = pattern.span.join(body.span).unwrap_or(pattern.span);
    Ok(MatchArm::new(pattern, body, span))
}
