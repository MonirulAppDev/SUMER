//! Pratt expression parser for the SUMER programming language.
//!
//! Handles operator precedence, associativity, prefix unary operators,
//! postfix chaining, function calls, member access, array/map literals,
//! and lambda expressions.

use sumer_ast::{
    Argument, AssignmentOperator, BinaryOperator, ElseExprBranch, Expr, ExprKind, IfExpr, Literal,
    MapEntry, UnaryOperator,
};
use sumer_lexer::TokenKind;

use crate::cursor::TokenCursor;
use crate::error::{ParseError, ParseResult};

/// Canonical entry point for parsing an expression.
pub fn parse_expression(cursor: &mut TokenCursor<'_>) -> ParseResult<Expr> {
    parse_expression_with_precedence(cursor, 0)
}

/// Parses an expression with a minimum binding power using Pratt parsing.
pub fn parse_expression_with_precedence(
    cursor: &mut TokenCursor<'_>,
    min_bp: u8,
) -> ParseResult<Expr> {
    let mut left = parse_prefix(cursor)?;
    left = parse_postfix(cursor, left)?;

    while let Some((op, left_bp, right_bp)) = infix_binding_power(cursor.current().kind()) {
        if left_bp < min_bp {
            break;
        }

        // Consume operator token
        cursor.advance();

        // Parse right operand with right binding power
        let right = parse_expression_with_precedence(cursor, right_bp)?;
        let span = left.span.join(right.span).unwrap_or(left.span);

        left = match op {
            InfixOp::Binary(binary_op) => Expr::binary(left, binary_op, right, span),
            InfixOp::Assignment(assign_op) => Expr::new(
                ExprKind::Assignment {
                    target: Box::new(left),
                    operator: assign_op,
                    value: Box::new(right),
                },
                span,
            ),
        };
    }

    Ok(left)
}

/// Parses prefix operators, literals, parenthesized expressions, and primary atoms.
fn parse_prefix(cursor: &mut TokenCursor<'_>) -> ParseResult<Expr> {
    // 1. Try parsing lambda expressions like `(a, b) => expr`
    if let Some(lambda_res) = try_parse_lambda(cursor) {
        return lambda_res;
    }

    let tok = cursor.current();
    match tok.kind() {
        // --- Literals ---
        TokenKind::Integer(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Ok(Expr::literal(Literal::Integer(val), span))
        }
        TokenKind::Float(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Ok(Expr::literal(Literal::Float(val), span))
        }
        TokenKind::String(val) => {
            let val = val.clone();
            let span = tok.span();
            cursor.advance();
            Ok(Expr::literal(Literal::String(val), span))
        }
        TokenKind::Char(c) => {
            let c = *c;
            let span = tok.span();
            cursor.advance();
            Ok(Expr::literal(Literal::Char(c), span))
        }
        TokenKind::True => {
            let span = tok.span();
            cursor.advance();
            Ok(Expr::literal(Literal::Bool(true), span))
        }
        TokenKind::False => {
            let span = tok.span();
            cursor.advance();
            Ok(Expr::literal(Literal::Bool(false), span))
        }
        TokenKind::None => {
            let span = tok.span();
            cursor.advance();
            Ok(Expr::literal(Literal::None, span))
        }

        // --- Identifiers / Single-Param Lambda / Built-in Variant Constructors ---
        TokenKind::Identifier(_) | TokenKind::Some | TokenKind::Ok | TokenKind::Err => {
            // Check for single parameter lambda: `x => expr`
            if cursor.peek_n(1).kind() == &TokenKind::FatArrow {
                let param = cursor.parse_identifier()?;
                cursor.advance(); // consume `=>`
                let body = parse_expression(cursor)?;
                let span = param.span.join(body.span).unwrap_or(param.span);
                Ok(Expr::new(
                    ExprKind::Lambda {
                        parameters: vec![param],
                        body: Box::new(body),
                    },
                    span,
                ))
            } else {
                let ident = cursor.parse_identifier()?;
                Ok(Expr::identifier(ident))
            }
        }

        // --- Grouped / Parenthesized Expression ---
        TokenKind::LParen => {
            let lparen = cursor.advance();
            let inner = parse_expression(cursor)?;
            let rparen = cursor.expect(&TokenKind::RParen)?;
            let span = lparen.span().join(rparen.span()).unwrap_or(inner.span);
            Ok(Expr::new(inner.kind, span))
        }

        // --- Array Literal: `[1, 2, 3]` ---
        TokenKind::LBracket => parse_array_literal(cursor),

        // --- Map Literal or Block Expression: `{ ... }` ---
        TokenKind::LBrace => parse_map_or_block(cursor),

        // --- Unary Prefix Operators (Precedence level 9, binding power 17) ---
        TokenKind::Bang => {
            let start = cursor.advance().span();
            let operand = parse_expression_with_precedence(cursor, 17)?;
            let span = start.join(operand.span).unwrap_or(start);
            Ok(Expr::new(
                ExprKind::Unary {
                    operator: UnaryOperator::Not,
                    operand: Box::new(operand),
                },
                span,
            ))
        }
        TokenKind::Minus => {
            let start = cursor.advance().span();
            let operand = parse_expression_with_precedence(cursor, 17)?;
            let span = start.join(operand.span).unwrap_or(start);
            Ok(Expr::new(
                ExprKind::Unary {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(operand),
                },
                span,
            ))
        }
        TokenKind::Plus => {
            let start = cursor.advance().span();
            let operand = parse_expression_with_precedence(cursor, 17)?;
            let span = start.join(operand.span).unwrap_or(start);
            Ok(Expr::new(
                ExprKind::Unary {
                    operator: UnaryOperator::Positive,
                    operand: Box::new(operand),
                },
                span,
            ))
        }
        TokenKind::AmpMut => {
            let start = cursor.advance().span();
            let operand = parse_expression_with_precedence(cursor, 17)?;
            let span = start.join(operand.span).unwrap_or(start);
            Ok(Expr::new(
                ExprKind::Reference {
                    mutable: true,
                    operand: Box::new(operand),
                },
                span,
            ))
        }
        TokenKind::Ampersand => {
            let start = cursor.advance().span();
            let operand = parse_expression_with_precedence(cursor, 17)?;
            let span = start.join(operand.span).unwrap_or(start);
            Ok(Expr::new(
                ExprKind::Reference {
                    mutable: false,
                    operand: Box::new(operand),
                },
                span,
            ))
        }
        TokenKind::Await => {
            let start = cursor.advance().span();
            let operand = parse_expression_with_precedence(cursor, 17)?;
            let span = start.join(operand.span).unwrap_or(start);
            Ok(Expr::await_expr(operand, span))
        }
        TokenKind::Spawn => {
            let start = cursor.advance().span();
            let operand = parse_expression_with_precedence(cursor, 17)?;
            let span = start.join(operand.span).unwrap_or(start);
            Ok(Expr::spawn_expr(operand, span))
        }

        // --- If and Match expressions ---
        TokenKind::If => parse_if_expression(cursor),
        TokenKind::Match => parse_match_expression(cursor),

        _ => {
            if cursor.is_at_end() {
                Err(ParseError::unexpected_eof(
                    "expression",
                    cursor.current().span(),
                ))
            } else {
                Err(ParseError::unexpected_token("expression", cursor.current()))
            }
        }
    }
}

/// Parses postfix operators (calls, members, optional members, index subscriptions, force unwrap).
fn parse_postfix(cursor: &mut TokenCursor<'_>, mut expr: Expr) -> ParseResult<Expr> {
    loop {
        // Optional member access: `?.`
        if cursor.check(&TokenKind::Question) && cursor.peek_n(1).kind() == &TokenKind::Dot {
            cursor.advance(); // consume `?`
            cursor.advance(); // consume `.`
            let member = cursor.parse_identifier()?;
            let span = expr.span.join(member.span).unwrap_or(expr.span);
            expr = Expr::optional_member(expr, member, span);
        }
        // Member access: `.`
        else if cursor.check(&TokenKind::Dot) {
            cursor.advance(); // consume `.`
            let member = cursor.parse_identifier()?;
            let span = expr.span.join(member.span).unwrap_or(expr.span);
            expr = Expr::member(expr, member, span);
        }
        // Function call: `(`
        else if cursor.check(&TokenKind::LParen) {
            expr = parse_call_arguments(cursor, expr)?;
        }
        // Index subscription: `[`
        else if cursor.check(&TokenKind::LBracket) {
            cursor.advance(); // consume `[`
            let index = parse_expression(cursor)?;
            let rbracket = cursor.expect(&TokenKind::RBracket)?;
            let span = expr.span.join(rbracket.span()).unwrap_or(expr.span);
            expr = Expr::index(expr, index, span);
        }
        // Force unwrap: `!`
        else if cursor.check(&TokenKind::Bang) {
            let bang = cursor.advance();
            let span = expr.span.join(bang.span()).unwrap_or(expr.span);
            expr = Expr::force_unwrap(expr, span);
        } else {
            break;
        }
    }

    Ok(expr)
}

/// Helper to parse function call arguments: `(arg1, name: arg2)`.
fn parse_call_arguments(cursor: &mut TokenCursor<'_>, callee: Expr) -> ParseResult<Expr> {
    cursor.expect(&TokenKind::LParen)?;
    let mut arguments = Vec::new();

    if !cursor.check(&TokenKind::RParen) {
        arguments.push(parse_argument(cursor)?);
        while cursor.match_token(&TokenKind::Comma) {
            if cursor.check(&TokenKind::RParen) {
                break;
            }
            arguments.push(parse_argument(cursor)?);
        }
    }

    let rparen = cursor.expect(&TokenKind::RParen)?;
    let span = callee.span.join(rparen.span()).unwrap_or(callee.span);
    Ok(Expr::call(callee, arguments, span))
}

/// Parses an individual argument (positional or named `name: value`).
fn parse_argument(cursor: &mut TokenCursor<'_>) -> ParseResult<Argument> {
    if matches!(cursor.current().kind(), TokenKind::Identifier(_))
        && cursor.peek_n(1).kind() == &TokenKind::Colon
    {
        let name = cursor.parse_identifier()?;
        cursor.expect(&TokenKind::Colon)?;
        let value = parse_expression(cursor)?;
        let span = name.span.join(value.span).unwrap_or(name.span);
        Ok(Argument::new(Some(name), value, span))
    } else {
        let value = parse_expression(cursor)?;
        let span = value.span;
        Ok(Argument::new(None, value, span))
    }
}

/// Parses array literals: `[elem1, elem2]`.
fn parse_array_literal(cursor: &mut TokenCursor<'_>) -> ParseResult<Expr> {
    let lbracket = cursor.expect(&TokenKind::LBracket)?;
    let start = lbracket.span();
    let mut elements = Vec::new();

    if !cursor.check(&TokenKind::RBracket) {
        elements.push(parse_expression(cursor)?);
        while cursor.match_token(&TokenKind::Comma) {
            if cursor.check(&TokenKind::RBracket) {
                break;
            }
            elements.push(parse_expression(cursor)?);
        }
    }

    let rbracket = cursor.expect(&TokenKind::RBracket)?;
    let span = start.join(rbracket.span()).unwrap_or(start);
    Ok(Expr::new(ExprKind::Array(elements), span))
}

/// Parses map literals: `{"key": value}` or block expressions `{ ... }`.
fn parse_map_or_block(cursor: &mut TokenCursor<'_>) -> ParseResult<Expr> {
    if let Some(map_res) = try_parse_map(cursor) {
        return map_res;
    }

    let block = crate::statement::parse_block(cursor)?;
    let span = block.span;
    Ok(Expr::new(ExprKind::Block(block), span))
}

/// Tries to parse a map literal `{ key: value, ... }`.
fn try_parse_map(cursor: &mut TokenCursor<'_>) -> Option<ParseResult<Expr>> {
    if !cursor.check(&TokenKind::LBrace) {
        return None;
    }

    let saved_pos = cursor.position();
    let lbrace_span = cursor.current().span();
    cursor.advance(); // consume `{`

    if cursor.check(&TokenKind::RBrace) {
        // Ambiguous empty braces `{}` - treated as empty block expression
        cursor.seek(saved_pos);
        return None;
    }

    // Try parsing first key expression
    let key = match parse_expression(cursor) {
        Ok(k) => k,
        Err(_) => {
            cursor.seek(saved_pos);
            return None;
        }
    };

    if !cursor.match_token(&TokenKind::Colon) {
        cursor.seek(saved_pos);
        return None;
    }

    // At this point it is definitely a map literal!
    let val = match parse_expression(cursor) {
        Ok(v) => v,
        Err(e) => return Some(Err(e)),
    };
    let entry_span = key.span.join(val.span).unwrap_or(key.span);
    let mut entries = vec![MapEntry::new(key, val, entry_span)];

    while cursor.match_token(&TokenKind::Comma) {
        if cursor.check(&TokenKind::RBrace) {
            break;
        }
        let k = match parse_expression(cursor) {
            Ok(k) => k,
            Err(e) => return Some(Err(e)),
        };
        if let Err(e) = cursor.expect(&TokenKind::Colon) {
            return Some(Err(e));
        }
        let v = match parse_expression(cursor) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let span = k.span.join(v.span).unwrap_or(k.span);
        entries.push(MapEntry::new(k, v, span));
    }

    let rbrace = match cursor.expect(&TokenKind::RBrace) {
        Ok(r) => r,
        Err(e) => return Some(Err(e)),
    };

    let span = lbrace_span.join(rbrace.span()).unwrap_or(lbrace_span);
    Some(Ok(Expr::new(ExprKind::Map(entries), span)))
}

/// Tries to parse a lambda expression `(param1, param2) => body`.
fn try_parse_lambda(cursor: &mut TokenCursor<'_>) -> Option<ParseResult<Expr>> {
    if !cursor.check(&TokenKind::LParen) {
        return None;
    }

    let saved_pos = cursor.position();
    let lparen_span = cursor.current().span();
    cursor.advance(); // consume `(`

    let mut parameters = Vec::new();

    if !cursor.check(&TokenKind::RParen) {
        let ident = match cursor.parse_identifier() {
            Ok(id) => id,
            Err(_) => {
                cursor.seek(saved_pos);
                return None;
            }
        };
        parameters.push(ident);

        while cursor.match_token(&TokenKind::Comma) {
            if cursor.check(&TokenKind::RParen) {
                break;
            }
            let ident = match cursor.parse_identifier() {
                Ok(id) => id,
                Err(_) => {
                    cursor.seek(saved_pos);
                    return None;
                }
            };
            parameters.push(ident);
        }
    }

    if !cursor.match_token(&TokenKind::RParen) {
        cursor.seek(saved_pos);
        return None;
    }

    if !cursor.match_token(&TokenKind::FatArrow) {
        cursor.seek(saved_pos);
        return None;
    }

    // Unambiguously a lambda: `(params) => body`
    let body = match parse_expression(cursor) {
        Ok(b) => b,
        Err(e) => return Some(Err(e)),
    };

    let span = lparen_span.join(body.span).unwrap_or(lparen_span);
    Some(Ok(Expr::new(
        ExprKind::Lambda {
            parameters,
            body: Box::new(body),
        },
        span,
    )))
}

/// Infix operator category (binary operator or assignment).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InfixOp {
    Binary(BinaryOperator),
    Assignment(AssignmentOperator),
}

/// Returns the operator and its Pratt `(left_binding_power, right_binding_power)` for a given token.
///
/// Precedence levels (highest to lowest):
/// - 8: Multiplicative (`*`, `/`, `%`) - left-associative: (15, 16)
/// - 7: Additive (`+`, `-`) - left-associative: (13, 14)
/// - 6: Range (`..`, `..=`) - left-associative: (11, 12)
/// - 5: Comparison (`<`, `>`, `<=`, `>=`) - left-associative: (9, 10)
/// - 4: Equality (`==`, `!=`) - left-associative: (7, 8)
/// - 3: Logical AND (`&&`) - left-associative: (5, 6)
/// - 2: Logical OR (`||`) - left-associative: (3, 4)
/// - 1: Assignment (`=`, `+=`, `-=`, `*=`, `/=`, `%=`) - right-associative: (2, 1)
fn infix_binding_power(kind: &TokenKind) -> Option<(InfixOp, u8, u8)> {
    match kind {
        // Level 1: Assignment (right-associative: left_bp > right_bp)
        TokenKind::Equal => Some((InfixOp::Assignment(AssignmentOperator::Assign), 2, 1)),
        TokenKind::PlusEqual => Some((InfixOp::Assignment(AssignmentOperator::AddAssign), 2, 1)),
        TokenKind::MinusEqual => Some((InfixOp::Assignment(AssignmentOperator::SubAssign), 2, 1)),
        TokenKind::StarEqual => Some((InfixOp::Assignment(AssignmentOperator::MulAssign), 2, 1)),
        TokenKind::SlashEqual => Some((InfixOp::Assignment(AssignmentOperator::DivAssign), 2, 1)),
        TokenKind::PercentEqual => Some((InfixOp::Assignment(AssignmentOperator::ModAssign), 2, 1)),

        // Level 2: Logical OR (left-associative: left_bp < right_bp)
        TokenKind::PipePipe => Some((InfixOp::Binary(BinaryOperator::LogicalOr), 3, 4)),

        // Level 3: Logical AND (left-associative)
        TokenKind::AmpAmp => Some((InfixOp::Binary(BinaryOperator::LogicalAnd), 5, 6)),

        // Level 4: Equality (left-associative)
        TokenKind::EqualEqual => Some((InfixOp::Binary(BinaryOperator::Equal), 7, 8)),
        TokenKind::BangEqual => Some((InfixOp::Binary(BinaryOperator::NotEqual), 7, 8)),

        // Level 5: Comparison (left-associative)
        TokenKind::Greater => Some((InfixOp::Binary(BinaryOperator::Greater), 9, 10)),
        TokenKind::Less => Some((InfixOp::Binary(BinaryOperator::Less), 9, 10)),
        TokenKind::GreaterEqual => Some((InfixOp::Binary(BinaryOperator::GreaterEqual), 9, 10)),
        TokenKind::LessEqual => Some((InfixOp::Binary(BinaryOperator::LessEqual), 9, 10)),

        // Level 6: Range (left-associative)
        TokenKind::DotDot => Some((InfixOp::Binary(BinaryOperator::Range), 11, 12)),
        TokenKind::DotDotEqual => Some((InfixOp::Binary(BinaryOperator::RangeInclusive), 11, 12)),

        // Level 7: Additive (left-associative)
        TokenKind::Plus => Some((InfixOp::Binary(BinaryOperator::Add), 13, 14)),
        TokenKind::Minus => Some((InfixOp::Binary(BinaryOperator::Subtract), 13, 14)),

        // Level 8: Multiplicative (left-associative)
        TokenKind::Star => Some((InfixOp::Binary(BinaryOperator::Multiply), 15, 16)),
        TokenKind::Slash => Some((InfixOp::Binary(BinaryOperator::Divide), 15, 16)),
        TokenKind::Percent => Some((InfixOp::Binary(BinaryOperator::Modulo), 15, 16)),

        _ => None,
    }
}

/// Parses an if-expression: `if cond { then } [else if cond2 { ... }] [else { ... }]`.
fn parse_if_expression(cursor: &mut TokenCursor<'_>) -> ParseResult<Expr> {
    let if_tok = cursor.expect(&TokenKind::If)?;
    let start = if_tok.span();
    let condition = parse_expression(cursor)?;
    let then_branch = crate::statement::parse_block(cursor)?;

    let else_branch = if cursor.match_token(&TokenKind::Else) {
        if cursor.check(&TokenKind::If) {
            let nested = parse_if_expression(cursor)?;
            match nested.kind {
                ExprKind::If(nested_if) => Some(ElseExprBranch::ElseIf(Box::new(nested_if))),
                _ => unreachable!(),
            }
        } else {
            let block = crate::statement::parse_block(cursor)?;
            Some(ElseExprBranch::Block(block))
        }
    } else {
        None
    };

    let end = match &else_branch {
        Some(ElseExprBranch::Block(b)) => b.span,
        Some(ElseExprBranch::ElseIf(nested)) => nested.span,
        None => then_branch.span,
    };
    let span = start.join(end).unwrap_or(start);

    Ok(Expr::new(
        ExprKind::If(IfExpr::new(condition, then_branch, else_branch, span)),
        span,
    ))
}

/// Parses a match-expression: `match value { arm1, arm2, ... }`.
fn parse_match_expression(cursor: &mut TokenCursor<'_>) -> ParseResult<Expr> {
    let match_tok = cursor.expect(&TokenKind::Match)?;
    let start = match_tok.span();
    let value = parse_expression(cursor)?;
    cursor.expect(&TokenKind::LBrace)?;

    let mut arms = Vec::new();
    while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
        arms.push(crate::pattern::parse_match_arm(cursor)?);
    }

    let rbrace = cursor.expect(&TokenKind::RBrace)?;
    let span = start.join(rbrace.span()).unwrap_or(start);

    Ok(Expr::new(
        ExprKind::Match {
            value: Box::new(value),
            arms,
        },
        span,
    ))
}
