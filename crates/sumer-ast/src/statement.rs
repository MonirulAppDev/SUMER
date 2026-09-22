//! Statement AST nodes.

use sumer_span::Span;

use crate::declaration::{ConstantDecl, VariableDecl};
use crate::expression::{Expr, MatchArm};
use crate::identifier::Identifier;

/// A code block containing a sequence of statements enclosed in `{ ... }`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Block {
    /// Statements executed sequentially within this block.
    pub statements: Vec<Stmt>,
    /// Span covering `{ ... }`.
    pub span: Span,
}

impl Block {
    /// Creates a new `Block`.
    pub fn new(statements: Vec<Stmt>, span: Span) -> Self {
        Self { statements, span }
    }

    /// Creates an empty `Block`.
    pub fn empty(span: Span) -> Self {
        Self {
            statements: Vec::new(),
            span,
        }
    }
}

/// An expression statement (e.g. `print("Hello")`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExpressionStmt {
    /// The inner expression.
    pub expr: Expr,
    /// Span of the statement.
    pub span: Span,
}

impl ExpressionStmt {
    /// Creates a new `ExpressionStmt`.
    pub fn new(expr: Expr, span: Span) -> Self {
        Self { expr, span }
    }
}

/// A return statement with optional return value (e.g. `return`, `return a + b`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReturnStmt {
    /// Optional return value expression.
    pub value: Option<Expr>,
    /// Span covering `return` and optional value.
    pub span: Span,
}

impl ReturnStmt {
    /// Creates a new `ReturnStmt`.
    pub fn new(value: Option<Expr>, span: Span) -> Self {
        Self { value, span }
    }
}

/// Else branch in an if-statement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ElseStmtBranch {
    /// Final `else { ... }` block.
    Block(Block),
    /// Chained `else if ...` statement.
    ElseIf(Box<IfStmt>),
}

/// An if-statement controlling execution flow.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IfStmt {
    /// Boolean condition expression.
    pub condition: Expr,
    /// Then block executed if condition is true.
    pub then_branch: Block,
    /// Optional else or else-if branch.
    pub else_branch: Option<ElseStmtBranch>,
    /// Span covering the entire if statement.
    pub span: Span,
}

impl IfStmt {
    /// Creates a new `IfStmt`.
    pub fn new(
        condition: Expr,
        then_branch: Block,
        else_branch: Option<ElseStmtBranch>,
        span: Span,
    ) -> Self {
        Self {
            condition,
            then_branch,
            else_branch,
            span,
        }
    }
}

/// A while loop statement (e.g. `while x > 0 { ... }`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WhileStmt {
    /// Loop condition expression.
    pub condition: Expr,
    /// Body block executed on each iteration.
    pub body: Block,
    /// Span covering `while ... { ... }`.
    pub span: Span,
}

impl WhileStmt {
    /// Creates a new `WhileStmt`.
    pub fn new(condition: Expr, body: Block, span: Span) -> Self {
        Self {
            condition,
            body,
            span,
        }
    }
}

/// A for-in loop statement (e.g. `for item in items { ... }`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ForStmt {
    /// Loop iteration variable identifier.
    pub variable: Identifier,
    /// Expression producing the iterable collection.
    pub iterable: Expr,
    /// Loop body block.
    pub body: Block,
    /// Span covering `for ... in ... { ... }`.
    pub span: Span,
}

impl ForStmt {
    /// Creates a new `ForStmt`.
    pub fn new(variable: Identifier, iterable: Expr, body: Block, span: Span) -> Self {
        Self {
            variable,
            iterable,
            body,
            span,
        }
    }
}

/// An unconditional loop statement (e.g. `loop { ... }`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LoopStmt {
    /// Body block repeated indefinitely until break/return.
    pub body: Block,
    /// Span covering `loop { ... }`.
    pub span: Span,
}

impl LoopStmt {
    /// Creates a new `LoopStmt`.
    pub fn new(body: Block, span: Span) -> Self {
        Self { body, span }
    }
}

/// A match statement executing arms without returning a value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MatchStmt {
    /// The value being matched.
    pub value: Expr,
    /// Match arms.
    pub arms: Vec<MatchArm>,
    /// Span covering `match ... { ... }`.
    pub span: Span,
}

impl MatchStmt {
    /// Creates a new `MatchStmt`.
    pub fn new(value: Expr, arms: Vec<MatchArm>, span: Span) -> Self {
        Self { value, arms, span }
    }
}

/// A break statement to exit a loop.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BreakStmt {
    /// Span of `break`.
    pub span: Span,
}

impl BreakStmt {
    /// Creates a new `BreakStmt`.
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

/// A continue statement to proceed to the next iteration of a loop.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContinueStmt {
    /// Span of `continue`.
    pub span: Span,
}

impl ContinueStmt {
    /// Creates a new `ContinueStmt`.
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

/// The specific statement variant kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StmtKind {
    /// Local variable binding (`let` or `var`).
    Variable(VariableDecl),
    /// Local constant binding (`const`).
    Constant(ConstantDecl),
    /// Standalone expression statement.
    Expression(ExpressionStmt),
    /// Return statement.
    Return(ReturnStmt),
    /// If statement.
    If(IfStmt),
    /// While loop.
    While(WhileStmt),
    /// For-in loop.
    For(ForStmt),
    /// Unconditional loop.
    Loop(LoopStmt),
    /// Match statement.
    Match(MatchStmt),
    /// Break loop statement.
    Break(BreakStmt),
    /// Continue loop statement.
    Continue(ContinueStmt),
    /// Nested block statement.
    Block(Block),
}

/// A statement node with its source span.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Stmt {
    /// The statement variant.
    pub kind: StmtKind,
    /// Span covering this statement.
    pub span: Span,
}

impl Stmt {
    /// Creates a new `Stmt`.
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Helper to construct an expression statement.
    pub fn expr(expr: Expr) -> Self {
        let span = expr.span;
        Self::new(StmtKind::Expression(ExpressionStmt::new(expr, span)), span)
    }

    /// Helper to construct a return statement.
    pub fn return_stmt(value: Option<Expr>, span: Span) -> Self {
        Self::new(StmtKind::Return(ReturnStmt::new(value, span)), span)
    }

    /// Helper to construct a variable statement.
    pub fn variable(decl: VariableDecl) -> Self {
        let span = decl.span;
        Self::new(StmtKind::Variable(decl), span)
    }

    /// Helper to construct a constant statement.
    pub fn constant(decl: ConstantDecl) -> Self {
        let span = decl.span;
        Self::new(StmtKind::Constant(decl), span)
    }

    /// Helper to construct a break statement.
    pub fn break_stmt(span: Span) -> Self {
        Self::new(StmtKind::Break(BreakStmt::new(span)), span)
    }

    /// Helper to construct a continue statement.
    pub fn continue_stmt(span: Span) -> Self {
        Self::new(StmtKind::Continue(ContinueStmt::new(span)), span)
    }
}
