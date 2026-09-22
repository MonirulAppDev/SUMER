//! Basic expression parsing boundary.
//!
//! Provides a clean foundation for primary expressions, literals, identifiers,
//! simple binary operations, and function calls. Full Pratt expression parsing
//! will be implemented in the next dedicated task.

use sumer_ast::{Argument, BinaryOperator, Expr, ExprKind, Literal};
use sumer_lexer::TokenKind;

use crate::cursor::TokenCursor;
use crate::error::{ParseError, ParseResult};

/// Parses a basic expression.
pub fn parse_expression(cursor: &mut TokenCursor<'_>) -> ParseResult<Expr> {
    let mut left = parse_primary(cursor)?;

    // Support basic binary operators if immediately following (e.g. `a + b`, `1 + 2`)
    while let Some(op) = match_binary_operator(cursor) {
        let right = parse_primary(cursor)?;
        let span = left.span.join(right.span).unwrap_or(left.span);
        left = Expr::binary(left, op, right, span);
    }

    Ok(left)
}

fn match_binary_operator(cursor: &mut TokenCursor<'_>) -> Option<BinaryOperator> {
    let op = match cursor.current().kind() {
        TokenKind::Plus => BinaryOperator::Add,
        TokenKind::Minus => BinaryOperator::Subtract,
        TokenKind::Star => BinaryOperator::Multiply,
        TokenKind::Slash => BinaryOperator::Divide,
        TokenKind::Percent => BinaryOperator::Modulo,
        TokenKind::EqualEqual => BinaryOperator::Equal,
        TokenKind::BangEqual => BinaryOperator::NotEqual,
        TokenKind::Greater => BinaryOperator::Greater,
        TokenKind::Less => BinaryOperator::Less,
        TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
        TokenKind::LessEqual => BinaryOperator::LessEqual,
        TokenKind::AmpAmp => BinaryOperator::LogicalAnd,
        TokenKind::PipePipe => BinaryOperator::LogicalOr,
        _ => return None,
    };
    cursor.advance();
    Some(op)
}

/// Parses a primary expression (literals, identifiers, calls, grouped expressions).
pub fn parse_primary(cursor: &mut TokenCursor<'_>) -> ParseResult<Expr> {
    let tok = cursor.current();
    let mut expr = match tok.kind() {
        TokenKind::Integer(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Expr::literal(Literal::Integer(val), span)
        }
        TokenKind::Float(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Expr::literal(Literal::Float(val), span)
        }
        TokenKind::String(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Expr::literal(Literal::String(val), span)
        }
        TokenKind::Char(c) => {
            let c = *c;
            let span = tok.span();
            cursor.advance();
            Expr::literal(Literal::Char(c), span)
        }
        TokenKind::True => {
            let span = tok.span();
            cursor.advance();
            Expr::literal(Literal::Bool(true), span)
        }
        TokenKind::False => {
            let span = tok.span();
            cursor.advance();
            Expr::literal(Literal::Bool(false), span)
        }
        TokenKind::None => {
            let span = tok.span();
            cursor.advance();
            Expr::literal(Literal::None, span)
        }
        TokenKind::Identifier(_) => {
            let ident = cursor.parse_identifier()?;
            Expr::identifier(ident)
        }
        TokenKind::LParen => {
            cursor.advance(); // consume '('
            let inner = parse_expression(cursor)?;
            cursor.expect(&TokenKind::RParen)?;
            inner
        }
        _ => {
            return Err(ParseError::unexpected_token("expression", tok));
        }
    };

    // Check for call suffix: `expr(...)`
    while cursor.check(&TokenKind::LParen) {
        cursor.advance(); // consume '('
        let mut args = Vec::new();
        if !cursor.check(&TokenKind::RParen) {
            args.push(parse_argument(cursor)?);
            while cursor.match_token(&TokenKind::Comma) {
                if cursor.check(&TokenKind::RParen) {
                    break;
                }
                args.push(parse_argument(cursor)?);
            }
        }
        let rparen = cursor.expect(&TokenKind::RParen)?;
        let span = expr.span.join(rparen.span()).unwrap_or(expr.span);
        expr = Expr::call(expr, args, span);
    }

    Ok(expr)
}

fn parse_argument(cursor: &mut TokenCursor<'_>) -> ParseResult<Argument> {
    // Check for named argument: `ident: expr`
    if let TokenKind::Identifier(_) = cursor.current().kind() {
        if cursor.peek_n(1).kind() == &TokenKind::Colon {
            let name = cursor.parse_identifier()?;
            cursor.expect(&TokenKind::Colon)?;
            let value = parse_expression(cursor)?;
            let span = name.span().join(value.span).unwrap_or(name.span());
            return Ok(Argument::named(name, value, span));
        }
    }

    let value = parse_expression(cursor)?;
    Ok(Argument::positional(value))
}
