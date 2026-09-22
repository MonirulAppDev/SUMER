//! Deterministic human-readable AST pretty-printer for SUMER.
//!
//! Produces structured, readable tree diagrams from [`Program`] and AST nodes
//! using standard box-drawing characters (`├── `, `└── `, `│   `, `    `).

use crate::attribute::{Attribute, AttributeArg};
use crate::declaration::{
    ConstantDecl, Declaration, EnumDecl, FunctionDecl, FunctionSignature, ImplDecl, ImportDecl,
    StructDecl, StructMember, TraitDecl, TraitMember, VariableDecl, VariantData, Visibility,
};
use crate::expression::{
    AssignmentOperator, BinaryOperator, Expr, ExprKind, Literal, MatchArm, UnaryOperator,
};
use crate::generics::GenericParams;
use crate::pattern::{Pattern, PatternKind};
use crate::program::Program;
use crate::statement::{Block, ElseStmtBranch, Stmt, StmtKind};
use crate::types::{Type, TypeKind};
use sumer_span::Span;

/// A node in the hierarchical tree representation of an AST structure.
#[derive(Clone, Debug)]
struct TreeNode {
    text: String,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    fn render(&self, output: &mut String, prefix: &str, is_last: bool, is_root: bool) {
        if is_root {
            output.push_str(&self.text);
            output.push('\n');
        } else {
            output.push_str(prefix);
            if is_last {
                output.push_str("└── ");
            } else {
                output.push_str("├── ");
            }
            output.push_str(&self.text);
            output.push('\n');
        }

        let child_prefix = if is_root {
            String::new()
        } else if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };

        for (i, child) in self.children.iter().enumerate() {
            let child_is_last = i == self.children.len() - 1;
            child.render(output, &child_prefix, child_is_last, false);
        }
    }
}

/// AST Pretty-Printer that produces human-readable tree outputs.
#[derive(Clone, Debug, Default)]
pub struct AstPrinter {
    show_spans: bool,
}

impl AstPrinter {
    /// Creates a new `AstPrinter` with default configuration (clean output, no spans).
    pub fn new() -> Self {
        Self { show_spans: false }
    }

    /// Sets whether to include source spans on nodes.
    pub fn with_spans(mut self, show_spans: bool) -> Self {
        self.show_spans = show_spans;
        self
    }

    /// Formats a node label, optionally appending compact span information.
    fn label(&self, text: &str, span: Span) -> String {
        if self.show_spans {
            format!("{text} [{}..{}]", span.start, span.end)
        } else {
            text.to_string()
        }
    }

    fn node(&self, text: &str, span: Span) -> TreeNode {
        TreeNode::new(self.label(text, span))
    }

    /// Pretty-prints a [`Program`] AST into a tree-formatted string.
    pub fn print(&self, program: &Program) -> String {
        let mut root = self.node("Program", program.span);

        for attr in &program.attributes {
            root.add_child(self.print_attribute(attr));
        }

        for decl in &program.declarations {
            root.add_child(self.print_declaration(decl));
        }

        let mut output = String::new();
        root.render(&mut output, "", true, true);
        output
    }

    // -------------------------------------------------------------------------
    // Declarations
    // -------------------------------------------------------------------------

    /// Pretty-prints a top-level declaration.
    fn print_declaration(&self, decl: &Declaration) -> TreeNode {
        match decl {
            Declaration::Function(f) => self.print_function(f),
            Declaration::Struct(s) => self.print_struct(s),
            Declaration::Enum(e) => self.print_enum(e),
            Declaration::Trait(t) => self.print_trait(t),
            Declaration::Impl(i) => self.print_impl(i),
            Declaration::Variable(v) => self.print_variable(v),
            Declaration::Constant(c) => self.print_constant(c),
            Declaration::Import(im) => self.print_import(im),
        }
    }

    fn print_function(&self, f: &FunctionDecl) -> TreeNode {
        let mut node = self.node(&format!("Function: {}", f.name.name()), f.span);

        for attr in &f.attributes {
            node.add_child(self.print_attribute(attr));
        }

        let vis = match f.visibility {
            Visibility::Public => "Public",
            Visibility::Private => "Private",
        };
        node.add_child(TreeNode::new(format!("Visibility: {vis}")));
        node.add_child(TreeNode::new(format!("Async: {}", f.is_async)));

        if !f.generics.params.is_empty() {
            node.add_child(self.print_generics(&f.generics));
        }

        if f.parameters.is_empty() {
            node.add_child(TreeNode::new("Parameters: 0"));
        } else {
            let mut params_node = TreeNode::new("Parameters");
            for p in &f.parameters {
                let ty_str = self.type_to_string(&p.param_type);
                let label = if let Some(ref def) = p.default_value {
                    let mut p_node = TreeNode::new(format!("{}: {}", p.name.name(), ty_str));
                    let mut def_node = TreeNode::new("Default");
                    def_node.add_child(self.print_expr(def));
                    p_node.add_child(def_node);
                    p_node
                } else {
                    TreeNode::new(format!("{}: {}", p.name.name(), ty_str))
                };
                params_node.add_child(label);
            }
            node.add_child(params_node);
        }

        if let Some(ref ret) = f.return_type {
            node.add_child(TreeNode::new(format!(
                "ReturnType: {}",
                self.type_to_string(ret)
            )));
        }

        let mut body_node = TreeNode::new("Body");
        for stmt in &f.body.statements {
            body_node.add_child(self.print_statement(stmt));
        }
        node.add_child(body_node);

        node
    }

    fn print_struct(&self, s: &StructDecl) -> TreeNode {
        let mut node = self.node(&format!("Struct: {}", s.name.name()), s.span);

        for attr in &s.attributes {
            node.add_child(self.print_attribute(attr));
        }

        let vis = match s.visibility {
            Visibility::Public => "Public",
            Visibility::Private => "Private",
        };
        node.add_child(TreeNode::new(format!("Visibility: {vis}")));

        if !s.generics.params.is_empty() {
            node.add_child(self.print_generics(&s.generics));
        }

        for member in &s.members {
            match member {
                StructMember::Field(f) => {
                    node.add_child(TreeNode::new(format!(
                        "Field: {} : {}",
                        f.name.name(),
                        self.type_to_string(&f.field_type)
                    )));
                }
                StructMember::Method(m) => {
                    let mut method_node = TreeNode::new(format!("Method: {}", m.name.name()));
                    let mut body_node = TreeNode::new("Body");
                    for stmt in &m.body.statements {
                        body_node.add_child(self.print_statement(stmt));
                    }
                    method_node.add_child(body_node);
                    node.add_child(method_node);
                }
            }
        }

        node
    }

    fn print_enum(&self, e: &EnumDecl) -> TreeNode {
        let mut node = self.node(&format!("Enum: {}", e.name.name()), e.span);

        for attr in &e.attributes {
            node.add_child(self.print_attribute(attr));
        }

        let vis = match e.visibility {
            Visibility::Public => "Public",
            Visibility::Private => "Private",
        };
        node.add_child(TreeNode::new(format!("Visibility: {vis}")));

        if !e.generics.params.is_empty() {
            node.add_child(self.print_generics(&e.generics));
        }

        let mut variants_node = TreeNode::new("Variants");
        for v in &e.variants {
            let v_node = match &v.data {
                VariantData::Unit => TreeNode::new(format!("Variant: {}", v.name.name())),
                VariantData::Tuple(types) => {
                    let types_str = types
                        .iter()
                        .map(|t| self.type_to_string(t))
                        .collect::<Vec<_>>()
                        .join(", ");
                    TreeNode::new(format!("Variant: {}({})", v.name.name(), types_str))
                }
                VariantData::Struct(fields) => {
                    let mut s_node = TreeNode::new(format!("Variant: {}", v.name.name()));
                    for f in fields {
                        s_node.add_child(TreeNode::new(format!(
                            "Field: {} : {}",
                            f.name.name(),
                            self.type_to_string(&f.field_type)
                        )));
                    }
                    s_node
                }
            };
            variants_node.add_child(v_node);
        }
        node.add_child(variants_node);

        node
    }

    fn print_trait(&self, t: &TraitDecl) -> TreeNode {
        let mut node = self.node(&format!("Trait: {}", t.name.name()), t.span);

        for attr in &t.attributes {
            node.add_child(self.print_attribute(attr));
        }

        let vis = match t.visibility {
            Visibility::Public => "Public",
            Visibility::Private => "Private",
        };
        node.add_child(TreeNode::new(format!("Visibility: {vis}")));

        if !t.generics.params.is_empty() {
            node.add_child(self.print_generics(&t.generics));
        }

        if !t.bounds.is_empty() {
            let bounds_str = t
                .bounds
                .iter()
                .map(|b| b.name.name().to_string())
                .collect::<Vec<_>>()
                .join(" + ");
            node.add_child(TreeNode::new(format!("Supertraits: {bounds_str}")));
        }

        let mut members_node = TreeNode::new("Members");
        for m in &t.members {
            match m {
                TraitMember::FunctionSignature(sig) => {
                    members_node.add_child(self.print_function_sig(sig));
                }
                TraitMember::Function(f) => {
                    members_node.add_child(self.print_function(f));
                }
            }
        }
        node.add_child(members_node);

        node
    }

    fn print_function_sig(&self, sig: &FunctionSignature) -> TreeNode {
        let mut node = self.node(&format!("Signature: {}", sig.name.name()), sig.span);
        node.add_child(TreeNode::new(format!("Async: {}", sig.is_async)));

        if !sig.generics.params.is_empty() {
            node.add_child(self.print_generics(&sig.generics));
        }

        let mut params_node = TreeNode::new("Parameters");
        for p in &sig.parameters {
            params_node.add_child(TreeNode::new(format!(
                "{}: {}",
                p.name.name(),
                self.type_to_string(&p.param_type)
            )));
        }
        node.add_child(params_node);

        if let Some(ref ret) = sig.return_type {
            node.add_child(TreeNode::new(format!(
                "ReturnType: {}",
                self.type_to_string(ret)
            )));
        }

        node
    }

    fn print_impl(&self, i: &ImplDecl) -> TreeNode {
        let title = if let Some(ref tr) = i.trait_type {
            format!(
                "Impl: {} for {}",
                self.type_to_string(tr),
                self.type_to_string(&i.target_type)
            )
        } else {
            format!("Impl: {}", self.type_to_string(&i.target_type))
        };
        let mut node = self.node(&title, i.span);

        for attr in &i.attributes {
            node.add_child(self.print_attribute(attr));
        }

        if !i.generics.params.is_empty() {
            node.add_child(self.print_generics(&i.generics));
        }

        let mut members_node = TreeNode::new("Members");
        for m in &i.members {
            members_node.add_child(self.print_function(m));
        }
        node.add_child(members_node);

        node
    }

    fn print_variable(&self, v: &VariableDecl) -> TreeNode {
        let keyword = if v.is_mutable { "var" } else { "let" };
        let mut title = format!("VariableDecl: {} {}", keyword, v.name.name());
        if let Some(ref ty) = v.explicit_type {
            title.push_str(&format!(": {}", self.type_to_string(ty)));
        }
        let mut node = self.node(&title, v.span);

        let mut init_node = TreeNode::new("Initializer");
        init_node.add_child(self.print_expr(&v.initializer));
        node.add_child(init_node);

        node
    }

    fn print_constant(&self, c: &ConstantDecl) -> TreeNode {
        let mut title = format!("ConstantDecl: {}", c.name.name());
        if let Some(ref ty) = c.explicit_type {
            title.push_str(&format!(": {}", self.type_to_string(ty)));
        }
        let mut node = self.node(&title, c.span);

        let mut val_node = TreeNode::new("Value");
        val_node.add_child(self.print_expr(&c.value));
        node.add_child(val_node);

        node
    }

    fn print_import(&self, im: &ImportDecl) -> TreeNode {
        let mut node = self.node("Import", im.span);
        let mut path_node = TreeNode::new("Path");
        for seg in &im.path.segments {
            path_node.add_child(TreeNode::new(seg.name().to_string()));
        }
        node.add_child(path_node);
        node
    }

    fn print_generics(&self, g: &GenericParams) -> TreeNode {
        let mut node = TreeNode::new("Generics");
        for p in &g.params {
            let mut p_node = TreeNode::new(p.name.name().to_string());
            for b in &p.bounds {
                p_node.add_child(TreeNode::new(format!("Bound: {}", b.name.name())));
            }
            node.add_child(p_node);
        }
        node
    }

    fn print_attribute(&self, attr: &Attribute) -> TreeNode {
        let mut node = self.node(&format!("Attribute: {}", attr.name.name()), attr.span);
        for arg in &attr.arguments {
            let arg_node = match arg {
                AttributeArg::Identifier(id) => TreeNode::new(format!("Argument: {}", id.name())),
                AttributeArg::String(s) => TreeNode::new(format!("Argument: \"{s}\"")),
            };
            node.add_child(arg_node);
        }
        node
    }

    // -------------------------------------------------------------------------
    // Statements
    // -------------------------------------------------------------------------

    /// Pretty-prints a statement.
    fn print_statement(&self, stmt: &Stmt) -> TreeNode {
        match &stmt.kind {
            StmtKind::Variable(v) => self.print_variable(v),
            StmtKind::Constant(c) => self.print_constant(c),
            StmtKind::Expression(e) => {
                let mut node = self.node("ExpressionStatement", stmt.span);
                node.add_child(self.print_expr(&e.expr));
                node
            }
            StmtKind::Return(r) => {
                let mut node = self.node("Return", stmt.span);
                if let Some(ref val) = r.value {
                    node.add_child(self.print_expr(val));
                }
                node
            }
            StmtKind::If(i) => {
                let mut node = self.node("If", stmt.span);
                let mut cond_node = TreeNode::new("Condition");
                cond_node.add_child(self.print_expr(&i.condition));
                node.add_child(cond_node);

                let mut then_node = TreeNode::new("Then");
                then_node.add_child(self.print_block(&i.then_branch));
                node.add_child(then_node);

                if let Some(ref else_branch) = i.else_branch {
                    match else_branch {
                        ElseStmtBranch::Block(b) => {
                            let mut else_node = TreeNode::new("Else");
                            else_node.add_child(self.print_block(b));
                            node.add_child(else_node);
                        }
                        ElseStmtBranch::ElseIf(else_if) => {
                            let mut else_node = TreeNode::new("Else");
                            let fake_stmt = Stmt::new(StmtKind::If(*else_if.clone()), i.span);
                            else_node.add_child(self.print_statement(&fake_stmt));
                            node.add_child(else_node);
                        }
                    }
                }
                node
            }
            StmtKind::While(w) => {
                let mut node = self.node("While", stmt.span);
                let mut cond_node = TreeNode::new("Condition");
                cond_node.add_child(self.print_expr(&w.condition));
                node.add_child(cond_node);

                let mut body_node = TreeNode::new("Body");
                body_node.add_child(self.print_block(&w.body));
                node.add_child(body_node);
                node
            }
            StmtKind::For(f) => {
                let mut node = self.node("For", stmt.span);
                node.add_child(TreeNode::new(format!("Variable: {}", f.variable.name())));
                let mut iter_node = TreeNode::new("Iterable");
                iter_node.add_child(self.print_expr(&f.iterable));
                node.add_child(iter_node);

                let mut body_node = TreeNode::new("Body");
                body_node.add_child(self.print_block(&f.body));
                node.add_child(body_node);
                node
            }
            StmtKind::Loop(l) => {
                let mut node = self.node("Loop", stmt.span);
                let mut body_node = TreeNode::new("Body");
                body_node.add_child(self.print_block(&l.body));
                node.add_child(body_node);
                node
            }
            StmtKind::Match(m) => {
                let mut node = self.node("Match", stmt.span);
                let mut val_node = TreeNode::new("Value");
                val_node.add_child(self.print_expr(&m.value));
                node.add_child(val_node);

                let mut arms_node = TreeNode::new("Arms");
                for arm in &m.arms {
                    arms_node.add_child(self.print_match_arm(arm));
                }
                node.add_child(arms_node);
                node
            }
            StmtKind::Break(_) => self.node("Break", stmt.span),
            StmtKind::Continue(_) => self.node("Continue", stmt.span),
            StmtKind::Block(b) => self.print_block(b),
        }
    }

    fn print_block(&self, b: &Block) -> TreeNode {
        let mut node = self.node("Block", b.span);
        for stmt in &b.statements {
            node.add_child(self.print_statement(stmt));
        }
        node
    }

    fn print_match_arm(&self, arm: &MatchArm) -> TreeNode {
        let mut node = self.node("Arm", arm.span);
        let mut pat_node = TreeNode::new("Pattern");
        pat_node.add_child(self.print_pattern(&arm.pattern));
        node.add_child(pat_node);

        let mut body_node = TreeNode::new("Body");
        body_node.add_child(self.print_expr(&arm.body));
        node.add_child(body_node);
        node
    }

    // -------------------------------------------------------------------------
    // Expressions
    // -------------------------------------------------------------------------

    /// Pretty-prints an expression.
    fn print_expr(&self, expr: &Expr) -> TreeNode {
        match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                Literal::Integer(i) => self.node(&format!("Integer: {i}"), expr.span),
                Literal::Float(f) => self.node(&format!("Float: {f}"), expr.span),
                Literal::String(s) => self.node(&format!("String: \"{s}\""), expr.span),
                Literal::Char(c) => self.node(&format!("Char: '{c}'"), expr.span),
                Literal::Bool(b) => self.node(&format!("Bool: {b}"), expr.span),
                Literal::None => self.node("Literal: None", expr.span),
            },
            ExprKind::Identifier(id) => self.node(&format!("Identifier: {}", id.name()), expr.span),
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                let op_str = match operator {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Modulo => "%",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::Less => "<",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::LogicalAnd => "&&",
                    BinaryOperator::LogicalOr => "||",
                    BinaryOperator::Range => "..",
                    BinaryOperator::RangeInclusive => "..=",
                };
                let mut node = self.node(&format!("Binary: {op_str}"), expr.span);
                node.add_child(self.print_expr(left));
                node.add_child(self.print_expr(right));
                node
            }
            ExprKind::Unary { operator, operand } => {
                let op_str = match operator {
                    UnaryOperator::Not => "!",
                    UnaryOperator::Negate => "-",
                    UnaryOperator::Positive => "+",
                    UnaryOperator::Reference => "&",
                    UnaryOperator::MutableReference => "&mut",
                };
                let mut node = self.node(&format!("Unary: {op_str}"), expr.span);
                node.add_child(self.print_expr(operand));
                node
            }
            ExprKind::Assignment {
                target,
                operator,
                value,
            } => {
                let op_str = match operator {
                    AssignmentOperator::Assign => "=",
                    AssignmentOperator::AddAssign => "+=",
                    AssignmentOperator::SubAssign => "-=",
                    AssignmentOperator::MulAssign => "*=",
                    AssignmentOperator::DivAssign => "/=",
                    AssignmentOperator::ModAssign => "%=",
                };
                let mut node = self.node(&format!("Assignment: {op_str}"), expr.span);
                node.add_child(self.print_expr(target));
                node.add_child(self.print_expr(value));
                node
            }
            ExprKind::Call { callee, arguments } => {
                let mut node = self.node("Call", expr.span);
                let mut fn_node = TreeNode::new("Function");
                fn_node.add_child(self.print_expr(callee));
                node.add_child(fn_node);

                let mut args_node = TreeNode::new("Arguments");
                for arg in arguments {
                    if let Some(ref name) = arg.name {
                        let mut named_node = TreeNode::new(format!("Named: {}", name.name()));
                        named_node.add_child(self.print_expr(&arg.value));
                        args_node.add_child(named_node);
                    } else {
                        args_node.add_child(self.print_expr(&arg.value));
                    }
                }
                node.add_child(args_node);
                node
            }
            ExprKind::Member { object, member } => {
                let mut node = self.node(&format!("Member: {}", member.name()), expr.span);
                node.add_child(self.print_expr(object));
                node
            }
            ExprKind::OptionalMember { object, member } => {
                let mut node = self.node(&format!("OptionalMember: {}", member.name()), expr.span);
                node.add_child(self.print_expr(object));
                node
            }
            ExprKind::Index { object, index } => {
                let mut node = self.node("Index", expr.span);
                let mut obj_node = TreeNode::new("Object");
                obj_node.add_child(self.print_expr(object));
                node.add_child(obj_node);

                let mut idx_node = TreeNode::new("Index");
                idx_node.add_child(self.print_expr(index));
                node.add_child(idx_node);
                node
            }
            ExprKind::StructInit { name, fields } => {
                let path_str = name
                    .segments
                    .iter()
                    .map(|s| s.name())
                    .collect::<Vec<_>>()
                    .join(".");
                let mut node = self.node(&format!("StructInit: {path_str}"), expr.span);
                for f in fields {
                    let mut f_node = TreeNode::new(format!("Field: {}", f.name.name()));
                    f_node.add_child(self.print_expr(&f.value));
                    node.add_child(f_node);
                }
                node
            }
            ExprKind::Array(elements) => {
                let mut node = self.node("Array", expr.span);
                for elem in elements {
                    node.add_child(self.print_expr(elem));
                }
                node
            }
            ExprKind::Map(entries) => {
                let mut node = self.node("Map", expr.span);
                for entry in entries {
                    let mut entry_node = TreeNode::new("Entry");
                    let mut key_node = TreeNode::new("Key");
                    key_node.add_child(self.print_expr(&entry.key));
                    entry_node.add_child(key_node);

                    let mut val_node = TreeNode::new("Value");
                    val_node.add_child(self.print_expr(&entry.value));
                    entry_node.add_child(val_node);
                    node.add_child(entry_node);
                }
                node
            }
            ExprKind::Lambda { parameters, body } => {
                let params_str = parameters
                    .iter()
                    .map(|p| p.name())
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut node = self.node(&format!("Lambda: ({params_str})"), expr.span);
                node.add_child(self.print_expr(body));
                node
            }
            ExprKind::If(i) => {
                let mut node = self.node("IfExpr", expr.span);
                let mut cond_node = TreeNode::new("Condition");
                cond_node.add_child(self.print_expr(&i.condition));
                node.add_child(cond_node);

                let mut then_node = TreeNode::new("Then");
                then_node.add_child(self.print_block(&i.then_branch));
                node.add_child(then_node);

                if let Some(ref else_branch) = i.else_branch {
                    match else_branch {
                        crate::expression::ElseExprBranch::Block(b) => {
                            let mut else_node = TreeNode::new("Else");
                            else_node.add_child(self.print_block(b));
                            node.add_child(else_node);
                        }
                        crate::expression::ElseExprBranch::ElseIf(else_if) => {
                            let mut else_node = TreeNode::new("Else");
                            let fake_expr = Expr::new(ExprKind::If(*else_if.clone()), expr.span);
                            else_node.add_child(self.print_expr(&fake_expr));
                            node.add_child(else_node);
                        }
                    }
                }
                node
            }
            ExprKind::Match { value, arms } => {
                let mut node = self.node("MatchExpr", expr.span);
                let mut val_node = TreeNode::new("Value");
                val_node.add_child(self.print_expr(value));
                node.add_child(val_node);

                let mut arms_node = TreeNode::new("Arms");
                for arm in arms {
                    arms_node.add_child(self.print_match_arm(arm));
                }
                node.add_child(arms_node);
                node
            }
            ExprKind::Await(inner) => {
                let mut node = self.node("Await", expr.span);
                node.add_child(self.print_expr(inner));
                node
            }
            ExprKind::Spawn(inner) => {
                let mut node = self.node("Spawn", expr.span);
                node.add_child(self.print_expr(inner));
                node
            }
            ExprKind::Reference { mutable, operand } => {
                let title = if *mutable {
                    "Reference: &mut"
                } else {
                    "Reference: &"
                };
                let mut node = self.node(title, expr.span);
                node.add_child(self.print_expr(operand));
                node
            }
            ExprKind::Block(b) => self.print_block(b),
            ExprKind::ForceUnwrap(inner) => {
                let mut node = self.node("ForceUnwrap", expr.span);
                node.add_child(self.print_expr(inner));
                node
            }
        }
    }

    // -------------------------------------------------------------------------
    // Types
    // -------------------------------------------------------------------------

    fn type_to_string(&self, ty: &Type) -> String {
        match &ty.kind {
            TypeKind::Named(p) => p
                .segments
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>()
                .join("."),
            TypeKind::Generic { name, arguments } => {
                let base_str = name
                    .segments
                    .iter()
                    .map(|s| s.name())
                    .collect::<Vec<_>>()
                    .join(".");
                let args_str = arguments
                    .iter()
                    .map(|a| self.type_to_string(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{base_str}<{args_str}>")
            }
            TypeKind::Reference { inner } => format!("&{}", self.type_to_string(inner)),
            TypeKind::MutableReference { inner } => format!("&mut {}", self.type_to_string(inner)),
            TypeKind::Optional { inner } => format!("{}?", self.type_to_string(inner)),
            TypeKind::Function {
                parameters,
                return_type,
            } => {
                let params_str = parameters
                    .iter()
                    .map(|p| self.type_to_string(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({params_str}) -> {}", self.type_to_string(return_type))
            }
            TypeKind::Tuple { elements } => {
                let types_str = elements
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({types_str})")
            }
            TypeKind::Array { element, size } => {
                if let Some(sz) = size {
                    match &sz.kind {
                        ExprKind::Literal(Literal::Integer(i)) => {
                            format!("[{}; {i}]", self.type_to_string(element))
                        }
                        _ => format!("[{}; ...]", self.type_to_string(element)),
                    }
                } else {
                    format!("[{}]", self.type_to_string(element))
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Patterns
    // -------------------------------------------------------------------------

    /// Pretty-prints a pattern.
    fn print_pattern(&self, pat: &Pattern) -> TreeNode {
        match &pat.kind {
            PatternKind::Wildcard => self.node("WildcardPattern: _", pat.span),
            PatternKind::Identifier(id) => {
                self.node(&format!("IdentifierPattern: {}", id.name()), pat.span)
            }
            PatternKind::Literal(lit) => {
                let lit_str = match lit {
                    Literal::Integer(i) => i.clone(),
                    Literal::Float(f) => f.clone(),
                    Literal::String(s) => format!("\"{s}\""),
                    Literal::Char(c) => format!("'{c}'"),
                    Literal::Bool(b) => b.to_string(),
                    Literal::None => "None".to_string(),
                };
                self.node(&format!("LiteralPattern: {lit_str}"), pat.span)
            }
            PatternKind::Enum {
                path,
                variant,
                data,
            } => {
                let name = if path.segments.is_empty() {
                    variant.name().to_string()
                } else {
                    let path_str = path
                        .segments
                        .iter()
                        .map(|s| s.name())
                        .collect::<Vec<_>>()
                        .join(".");
                    format!("{path_str}.{}", variant.name())
                };
                let mut node = self.node(&format!("EnumPattern: {name}"), pat.span);
                if let Some(sub_patterns) = data {
                    for sub in sub_patterns {
                        node.add_child(self.print_pattern(sub));
                    }
                }
                node
            }
            PatternKind::Struct { path, fields } => {
                let path_str = path
                    .segments
                    .iter()
                    .map(|s| s.name())
                    .collect::<Vec<_>>()
                    .join(".");
                let mut node = self.node(&format!("StructPattern: {path_str}"), pat.span);
                for f in fields {
                    let mut f_node = TreeNode::new(format!("Field: {}", f.name.name()));
                    if let Some(ref sub) = f.pattern {
                        f_node.add_child(self.print_pattern(sub));
                    }
                    node.add_child(f_node);
                }
                node
            }
            PatternKind::Tuple(sub_patterns) => {
                let mut node = self.node("TuplePattern", pat.span);
                for sub in sub_patterns {
                    node.add_child(self.print_pattern(sub));
                }
                node
            }
        }
    }
}

/// Convenience function to pretty-print a [`Program`] into a clean string.
pub fn pretty_print(program: &Program) -> String {
    AstPrinter::new().print(program)
}

/// Convenience function to pretty-print a [`Program`] including source spans.
pub fn pretty_print_with_spans(program: &Program) -> String {
    AstPrinter::new().with_spans(true).print(program)
}
