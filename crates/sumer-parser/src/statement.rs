//! Statement and block parsing.

use sumer_ast::{
    Block, ElseStmtBranch, ForStmt, IfStmt, LoopStmt, MatchStmt, Stmt, StmtKind, Visibility,
    WhileStmt,
};
use sumer_lexer::TokenKind;

use crate::cursor::TokenCursor;
use crate::declaration::{parse_constant_decl, parse_variable_decl};
use crate::error::ParseResult;
use crate::expression::parse_expression;
use crate::pattern::parse_match_arm;

/// Parses a block enclosed in curly braces `{ ... }`.
pub fn parse_block(cursor: &mut TokenCursor<'_>) -> ParseResult<Block> {
    let lbrace = cursor.expect(&TokenKind::LBrace)?;
    let start = lbrace.span();

    let mut statements = Vec::new();
    while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
        if cursor.match_token(&TokenKind::Semicolon) {
            continue;
        }
        statements.push(parse_statement(cursor)?);
    }

    let rbrace = cursor.expect(&TokenKind::RBrace)?;
    let span = start.join(rbrace.span()).unwrap_or(start);

    Ok(Block::new(statements, span))
}

/// Parses a single statement.
pub fn parse_statement(cursor: &mut TokenCursor<'_>) -> ParseResult<Stmt> {
    match cursor.current().kind() {
        TokenKind::Let | TokenKind::Var => {
            let var_decl = parse_variable_decl(cursor, Visibility::Private, None)?;
            Ok(Stmt::variable(var_decl))
        }
        TokenKind::Const => {
            let const_decl = parse_constant_decl(cursor, Visibility::Private, Vec::new(), None)?;
            Ok(Stmt::constant(const_decl))
        }
        TokenKind::Return => {
            let ret_tok = cursor.advance();
            let start = ret_tok.span();
            if cursor.check(&TokenKind::Semicolon)
                || cursor.check(&TokenKind::RBrace)
                || cursor.is_at_end()
            {
                cursor.consume_semicolon_if_present();
                Ok(Stmt::return_stmt(None, start))
            } else {
                let expr = parse_expression(cursor)?;
                cursor.consume_semicolon_if_present();
                let span = start.join(expr.span).unwrap_or(start);
                Ok(Stmt::return_stmt(Some(expr), span))
            }
        }
        TokenKind::Break => {
            let tok = cursor.advance();
            let span = tok.span();
            cursor.consume_semicolon_if_present();
            Ok(Stmt::break_stmt(span))
        }
        TokenKind::Continue => {
            let tok = cursor.advance();
            let span = tok.span();
            cursor.consume_semicolon_if_present();
            Ok(Stmt::continue_stmt(span))
        }
        TokenKind::LBrace => {
            let block = parse_block(cursor)?;
            let span = block.span;
            Ok(Stmt::new(StmtKind::Block(block), span))
        }
        TokenKind::If => parse_if_statement(cursor),
        TokenKind::While => parse_while_statement(cursor),
        TokenKind::For => parse_for_statement(cursor),
        TokenKind::Loop => parse_loop_statement(cursor),
        TokenKind::Match => parse_match_statement(cursor),
        _ => {
            let expr = parse_expression(cursor)?;
            cursor.consume_semicolon_if_present();
            Ok(Stmt::expr(expr))
        }
    }
}

/// Parses an if-statement: `if condition { then } [else if cond2 { ... }] [else { ... }]`.
pub fn parse_if_statement(cursor: &mut TokenCursor<'_>) -> ParseResult<Stmt> {
    let if_tok = cursor.expect(&TokenKind::If)?;
    let start = if_tok.span();
    let condition = parse_expression(cursor)?;
    let then_branch = parse_block(cursor)?;

    let else_branch = if cursor.match_token(&TokenKind::Else) {
        if cursor.check(&TokenKind::If) {
            let nested_stmt = parse_if_statement(cursor)?;
            match nested_stmt.kind {
                StmtKind::If(nested_if) => Some(ElseStmtBranch::ElseIf(Box::new(nested_if))),
                _ => unreachable!(),
            }
        } else {
            let block = parse_block(cursor)?;
            Some(ElseStmtBranch::Block(block))
        }
    } else {
        None
    };

    let end = match &else_branch {
        Some(ElseStmtBranch::Block(b)) => b.span,
        Some(ElseStmtBranch::ElseIf(nested)) => nested.span,
        None => then_branch.span,
    };
    let span = start.join(end).unwrap_or(start);

    Ok(Stmt::new(
        StmtKind::If(IfStmt::new(condition, then_branch, else_branch, span)),
        span,
    ))
}

/// Parses a while-loop: `while condition { body }`.
pub fn parse_while_statement(cursor: &mut TokenCursor<'_>) -> ParseResult<Stmt> {
    let while_tok = cursor.expect(&TokenKind::While)?;
    let start = while_tok.span();
    let condition = parse_expression(cursor)?;
    let body = parse_block(cursor)?;
    let span = start.join(body.span).unwrap_or(start);

    Ok(Stmt::new(
        StmtKind::While(WhileStmt::new(condition, body, span)),
        span,
    ))
}

/// Parses a for-in loop: `for item in items { body }`.
pub fn parse_for_statement(cursor: &mut TokenCursor<'_>) -> ParseResult<Stmt> {
    let for_tok = cursor.expect(&TokenKind::For)?;
    let start = for_tok.span();
    let variable = cursor.parse_identifier()?;
    cursor.expect(&TokenKind::In)?;
    let iterable = parse_expression(cursor)?;
    let body = parse_block(cursor)?;
    let span = start.join(body.span).unwrap_or(start);

    Ok(Stmt::new(
        StmtKind::For(ForStmt::new(variable, iterable, body, span)),
        span,
    ))
}

/// Parses an unconditional loop: `loop { body }`.
pub fn parse_loop_statement(cursor: &mut TokenCursor<'_>) -> ParseResult<Stmt> {
    let loop_tok = cursor.expect(&TokenKind::Loop)?;
    let start = loop_tok.span();
    let body = parse_block(cursor)?;
    let span = start.join(body.span).unwrap_or(start);

    Ok(Stmt::new(StmtKind::Loop(LoopStmt::new(body, span)), span))
}

/// Parses a match-statement: `match value { arm1, arm2, ... }`.
pub fn parse_match_statement(cursor: &mut TokenCursor<'_>) -> ParseResult<Stmt> {
    let match_tok = cursor.expect(&TokenKind::Match)?;
    let start = match_tok.span();
    let value = parse_expression(cursor)?;
    cursor.expect(&TokenKind::LBrace)?;

    let mut arms = Vec::new();
    while !cursor.check(&TokenKind::RBrace) && !cursor.is_at_end() {
        arms.push(parse_match_arm(cursor)?);
    }

    let rbrace = cursor.expect(&TokenKind::RBrace)?;
    let span = start.join(rbrace.span()).unwrap_or(start);

    Ok(Stmt::new(
        StmtKind::Match(MatchStmt::new(value, arms, span)),
        span,
    ))
}
