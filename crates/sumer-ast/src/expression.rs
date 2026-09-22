//! Expression AST nodes.

use sumer_span::Span;

use crate::identifier::Identifier;
use crate::pattern::Pattern;
use crate::statement::Block;
use crate::types::TypePath;

/// Literal values in the AST.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Literal {
    /// Integer literal (preserved as string representation, e.g. `"123"`, `"0xff"`).
    Integer(String),
    /// Float literal (preserved as string representation, e.g. `"3.14"`).
    Float(String),
    /// String literal content.
    String(String),
    /// Character literal.
    Char(char),
    /// Boolean literal (`true` or `false`).
    Bool(bool),
    /// None literal.
    None,
}

/// Binary arithmetic, comparison, logical, and range operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    // Logical
    LogicalAnd,
    LogicalOr,

    // Range
    Range,
    RangeInclusive,
}

/// Unary prefix operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    /// Logical negation `!`.
    Not,
    /// Arithmetic negation `-`.
    Negate,
    /// Explicit positive `+`.
    Positive,
    /// Immutable borrow reference `&`.
    Reference,
    /// Mutable borrow reference `&mut`.
    MutableReference,
}

/// Compound and simple assignment operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssignmentOperator {
    /// `=`
    Assign,
    /// `+=`
    AddAssign,
    /// `-=`
    SubAssign,
    /// `*=`
    MulAssign,
    /// `/=`
    DivAssign,
    /// `%=`
    ModAssign,
}

/// A positional or named argument passed into a function call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Argument {
    /// Optional argument name for named arguments (e.g. `age: 30`).
    pub name: Option<Identifier>,
    /// The argument value expression.
    pub value: Expr,
    /// Span covering this argument.
    pub span: Span,
}

impl Argument {
    /// Creates a new `Argument`.
    pub fn new(name: Option<Identifier>, value: Expr, span: Span) -> Self {
        Self { name, value, span }
    }

    /// Creates a positional argument without a name.
    pub fn positional(value: Expr) -> Self {
        let span = value.span;
        Self {
            name: None,
            value,
            span,
        }
    }

    /// Creates a named argument.
    pub fn named(name: Identifier, value: Expr, span: Span) -> Self {
        Self {
            name: Some(name),
            value,
            span,
        }
    }
}

/// A field initializer in struct instantiation (e.g. `name: "Monir"`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldInit {
    /// Field identifier.
    pub name: Identifier,
    /// Initializer value expression.
    pub value: Expr,
    /// Span covering this field initializer.
    pub span: Span,
}

impl FieldInit {
    /// Creates a new `FieldInit`.
    pub fn new(name: Identifier, value: Expr, span: Span) -> Self {
        Self { name, value, span }
    }
}

/// A key-value pair in a map literal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MapEntry {
    /// Entry key expression.
    pub key: Expr,
    /// Entry value expression.
    pub value: Expr,
    /// Span covering the key-value entry.
    pub span: Span,
}

impl MapEntry {
    /// Creates a new `MapEntry`.
    pub fn new(key: Expr, value: Expr, span: Span) -> Self {
        Self { key, value, span }
    }
}

/// An arm in a match statement or expression (e.g. `Pattern => body`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MatchArm {
    /// Pattern to match against.
    pub pattern: Pattern,
    /// Execution body for this arm.
    pub body: Expr,
    /// Span covering this match arm.
    pub span: Span,
}

impl MatchArm {
    /// Creates a new `MatchArm`.
    pub fn new(pattern: Pattern, body: Expr, span: Span) -> Self {
        Self {
            pattern,
            body,
            span,
        }
    }
}

/// Else branch in an if-expression.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ElseExprBranch {
    /// Final `else { ... }` block.
    Block(Block),
    /// Chained `else if ...` expression.
    ElseIf(Box<IfExpr>),
}

/// An if-expression that evaluates to a value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IfExpr {
    /// Condition expression.
    pub condition: Box<Expr>,
    /// Body evaluated if condition is true.
    pub then_branch: Block,
    /// Optional else or else-if branch.
    pub else_branch: Option<ElseExprBranch>,
    /// Span covering the entire if-expression.
    pub span: Span,
}

impl IfExpr {
    /// Creates a new `IfExpr`.
    pub fn new(
        condition: Expr,
        then_branch: Block,
        else_branch: Option<ElseExprBranch>,
        span: Span,
    ) -> Self {
        Self {
            condition: Box::new(condition),
            then_branch,
            else_branch,
            span,
        }
    }
}

/// The specific syntactic kind of an expression.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExprKind {
    /// Literal constant (e.g. `123`, `"text"`, `true`).
    Literal(Literal),

    /// Named identifier (e.g. `user`, `total`).
    Identifier(Identifier),

    /// Binary expression (e.g. `a + b`, `x == y`).
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },

    /// Unary prefix expression (e.g. `!flag`, `-count`).
    Unary {
        operator: UnaryOperator,
        operand: Box<Expr>,
    },

    /// Assignment expression (e.g. `x = 10`, `count += 1`).
    Assignment {
        target: Box<Expr>,
        operator: AssignmentOperator,
        value: Box<Expr>,
    },

    /// Function call expression (e.g. `print("hi")`, `add(1, 2)`).
    Call {
        callee: Box<Expr>,
        arguments: Vec<Argument>,
    },

    /// Field or method access (e.g. `user.name`).
    Member {
        object: Box<Expr>,
        member: Identifier,
    },

    /// Optional navigation member access (e.g. `user?.name`).
    OptionalMember {
        object: Box<Expr>,
        member: Identifier,
    },

    /// Index subscription expression (e.g. `items[0]`).
    Index { object: Box<Expr>, index: Box<Expr> },

    /// Struct initialization expression (e.g. `User { id: 1, name: "Monir" }`).
    StructInit {
        name: TypePath,
        fields: Vec<FieldInit>,
    },

    /// Array literal expression (e.g. `[1, 2, 3]`).
    Array(Vec<Expr>),

    /// Map literal expression (e.g. `{"name": "Monir", "age": 30}`).
    Map(Vec<MapEntry>),

    /// Anonymous lambda function (e.g. `(a, b) => a + b`).
    Lambda {
        parameters: Vec<Identifier>,
        body: Box<Expr>,
    },

    /// If-expression evaluating to a value.
    If(IfExpr),

    /// Match expression (e.g. `match val { 1 => "one", _ => "other" }`).
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// Await expression on an async task (e.g. `await task`).
    Await(Box<Expr>),

    /// Task spawn expression (e.g. `spawn download(url)`).
    Spawn(Box<Expr>),

    /// Borrow reference expression (e.g. `&value`, `&mut value`).
    Reference { mutable: bool, operand: Box<Expr> },

    /// Block expression evaluating to its last statement expression (e.g. `{ let x = 1; x + 2 }`).
    Block(Block),

    /// Force unwrap expression on an optional value (e.g. `user!`).
    ForceUnwrap(Box<Expr>),
}

/// An expression node with its source span.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Expr {
    /// The expression variant kind.
    pub kind: ExprKind,
    /// Span covering this expression.
    pub span: Span,
}

impl Expr {
    /// Creates a new `Expr`.
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Helper to construct a literal expression.
    pub fn literal(lit: Literal, span: Span) -> Self {
        Self::new(ExprKind::Literal(lit), span)
    }

    /// Helper to construct an identifier expression.
    pub fn identifier(ident: Identifier) -> Self {
        let span = ident.span;
        Self::new(ExprKind::Identifier(ident), span)
    }

    /// Helper to construct a binary expression.
    pub fn binary(left: Expr, operator: BinaryOperator, right: Expr, span: Span) -> Self {
        Self::new(
            ExprKind::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            },
            span,
        )
    }

    /// Helper to construct a call expression.
    pub fn call(callee: Expr, arguments: Vec<Argument>, span: Span) -> Self {
        Self::new(
            ExprKind::Call {
                callee: Box::new(callee),
                arguments,
            },
            span,
        )
    }

    /// Helper to construct a member expression.
    pub fn member(object: Expr, member: Identifier, span: Span) -> Self {
        Self::new(
            ExprKind::Member {
                object: Box::new(object),
                member,
            },
            span,
        )
    }

    /// Helper to construct an optional member expression (`?.`).
    pub fn optional_member(object: Expr, member: Identifier, span: Span) -> Self {
        Self::new(
            ExprKind::OptionalMember {
                object: Box::new(object),
                member,
            },
            span,
        )
    }

    /// Helper to construct an index expression (`[index]`).
    pub fn index(object: Expr, index: Expr, span: Span) -> Self {
        Self::new(
            ExprKind::Index {
                object: Box::new(object),
                index: Box::new(index),
            },
            span,
        )
    }

    /// Helper to construct an await expression.
    pub fn await_expr(expr: Expr, span: Span) -> Self {
        Self::new(ExprKind::Await(Box::new(expr)), span)
    }

    /// Helper to construct a spawn expression.
    pub fn spawn_expr(expr: Expr, span: Span) -> Self {
        Self::new(ExprKind::Spawn(Box::new(expr)), span)
    }

    /// Helper to construct a force unwrap expression (`!`).
    pub fn force_unwrap(expr: Expr, span: Span) -> Self {
        Self::new(ExprKind::ForceUnwrap(Box::new(expr)), span)
    }
}
