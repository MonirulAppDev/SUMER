//! Two-pass declaration collection and lexical name resolver.

use std::collections::HashMap;

use sumer_ast::Visibility;
use sumer_ast::declaration::{
    Declaration, EnumDecl, FunctionDecl, ImplDecl, Parameter, StructDecl, TraitDecl,
};
use sumer_ast::expression::{ElseExprBranch, Expr, ExprKind, IfExpr, Literal, MatchArm};
use sumer_ast::generics::TypeBound;
use sumer_ast::pattern::{Pattern, PatternKind};
use sumer_ast::program::Program;
use sumer_ast::statement::{Block, ElseStmtBranch, IfStmt, Stmt, StmtKind};
use sumer_ast::types::{Type, TypeKind, TypePath};
use sumer_diagnostics::Diagnostics;
use sumer_span::{SourceId, Span};

use crate::error::SemanticError;
use crate::scope::{ScopeId, ScopeKind, ScopeTree};
use crate::symbol::{SymbolId, SymbolKind, SymbolTable};

/// Standard built-in primitive types recognized by SUMER v0.1.
pub const BUILTIN_TYPES: &[&str] = &[
    "Bool", "Int8", "Int16", "Int32", "Int64", "Int128", "UInt8", "UInt16", "UInt32", "UInt64",
    "UInt128", "Float32", "Float64", "Char", "String", "Byte", "Int", "UInt", "Float", "Result",
    "Option",
];

/// Standard prelude functions and constructors available globally.
pub const BUILTIN_FUNCTIONS: &[&str] = &["print", "println", "Ok", "Err", "Some", "None"];

/// AST visitor performing declaration collection, scope construction, and name resolution.
pub struct Resolver {
    /// Allocated symbol arena.
    pub symbols: SymbolTable,
    /// Lexical scope tree arena.
    pub scopes: ScopeTree,
    /// Currently active lexical scope index.
    pub current_scope: ScopeId,
    /// Mapping from source spans to resolved symbol identifiers.
    pub resolutions: HashMap<Span, SymbolId>,
    /// Accumulated diagnostics (errors, warnings).
    pub diagnostics: Diagnostics,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    /// Creates a new `Resolver` with the global scope and standard built-ins pre-populated.
    pub fn new() -> Self {
        let mut symbols = SymbolTable::new();
        let mut scopes = ScopeTree::new();
        let global_scope = scopes.alloc(ScopeKind::Global, None);
        let dummy_span = Span::new(SourceId::new(0), 0, 0);

        // Register built-in primitive and standard types
        for &ty_name in BUILTIN_TYPES {
            let sym = symbols.alloc(
                ty_name,
                SymbolKind::BuiltinType,
                dummy_span,
                Visibility::Public,
            );
            let _ = scopes.define_type(global_scope, ty_name, sym);
        }

        // Register built-in functions and constructors
        for &fn_name in BUILTIN_FUNCTIONS {
            let sym = symbols.alloc(
                fn_name,
                SymbolKind::Function,
                dummy_span,
                Visibility::Public,
            );
            let _ = scopes.define_value(global_scope, fn_name, sym);
        }

        Self {
            symbols,
            scopes,
            current_scope: global_scope,
            resolutions: HashMap::new(),
            diagnostics: Diagnostics::new(),
        }
    }

    /// Executes two-pass semantic analysis on the provided [`Program`].
    pub fn resolve_program(&mut self, program: &Program) {
        // Pass 1: Declaration collection at top-level
        self.collect_declarations(&program.declarations);

        // Pass 2: Resolution & validation
        for decl in &program.declarations {
            self.resolve_declaration(decl);
        }
    }

    // =========================================================================
    // PASS 1: Declaration Collection
    // =========================================================================

    fn collect_declarations(&mut self, declarations: &[Declaration]) {
        for decl in declarations {
            match decl {
                Declaration::Function(f) => {
                    let sym = self.symbols.alloc(
                        &f.name.name,
                        SymbolKind::Function,
                        f.name.span,
                        f.visibility,
                    );
                    if let Err(prev) =
                        self.scopes
                            .define_value(self.current_scope, &f.name.name, sym)
                    {
                        self.emit_duplicate(&f.name.name, f.name.span, prev);
                    }
                }
                Declaration::Struct(s) => {
                    let sym = self.symbols.alloc(
                        &s.name.name,
                        SymbolKind::Struct,
                        s.name.span,
                        s.visibility,
                    );
                    if let Err(prev) =
                        self.scopes
                            .define_type(self.current_scope, &s.name.name, sym)
                    {
                        self.emit_duplicate(&s.name.name, s.name.span, prev);
                    } else {
                        // Also make accessible in value namespace for constructors / namespaces
                        let _ = self
                            .scopes
                            .define_value(self.current_scope, &s.name.name, sym);
                    }
                }
                Declaration::Enum(e) => {
                    let sym = self.symbols.alloc(
                        &e.name.name,
                        SymbolKind::Enum,
                        e.name.span,
                        e.visibility,
                    );
                    if let Err(prev) =
                        self.scopes
                            .define_type(self.current_scope, &e.name.name, sym)
                    {
                        self.emit_duplicate(&e.name.name, e.name.span, prev);
                    } else {
                        // Also make accessible in value namespace for variant access e.g. Status.Active
                        let _ = self
                            .scopes
                            .define_value(self.current_scope, &e.name.name, sym);
                    }
                }
                Declaration::Trait(t) => {
                    let sym = self.symbols.alloc(
                        &t.name.name,
                        SymbolKind::Trait,
                        t.name.span,
                        t.visibility,
                    );
                    if let Err(prev) =
                        self.scopes
                            .define_type(self.current_scope, &t.name.name, sym)
                    {
                        self.emit_duplicate(&t.name.name, t.name.span, prev);
                    }
                }
                Declaration::Variable(v) => {
                    let sym = self.symbols.alloc(
                        &v.name.name,
                        SymbolKind::Variable,
                        v.name.span,
                        v.visibility,
                    );
                    if let Err(prev) =
                        self.scopes
                            .define_value(self.current_scope, &v.name.name, sym)
                    {
                        self.emit_duplicate(&v.name.name, v.name.span, prev);
                    }
                }
                Declaration::Constant(c) => {
                    let sym = self.symbols.alloc(
                        &c.name.name,
                        SymbolKind::Constant,
                        c.name.span,
                        c.visibility,
                    );
                    if let Err(prev) =
                        self.scopes
                            .define_value(self.current_scope, &c.name.name, sym)
                    {
                        self.emit_duplicate(&c.name.name, c.name.span, prev);
                    }
                }
                Declaration::Import(im) => {
                    if let Some(last_seg) = im.path.segments.last() {
                        let sym = self.symbols.alloc(
                            &last_seg.name,
                            SymbolKind::Variable,
                            last_seg.span,
                            im.visibility,
                        );
                        let _ = self
                            .scopes
                            .define_value(self.current_scope, &last_seg.name, sym);
                        let _ = self
                            .scopes
                            .define_type(self.current_scope, &last_seg.name, sym);
                    }
                }
                Declaration::Impl(_) => {
                    // Impl blocks do not bind new top-level type names in the global scope
                }
            }
        }
    }

    // =========================================================================
    // PASS 2: Declaration Resolution
    // =========================================================================

    fn resolve_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Function(f) => self.resolve_function(f),
            Declaration::Struct(s) => self.resolve_struct(s),
            Declaration::Enum(e) => self.resolve_enum(e),
            Declaration::Trait(t) => self.resolve_trait(t),
            Declaration::Impl(i) => self.resolve_impl(i),
            Declaration::Variable(v) => {
                if let Some(explicit_ty) = &v.explicit_type {
                    self.resolve_type(explicit_ty);
                }
                self.resolve_expr(&v.initializer);
            }
            Declaration::Constant(c) => {
                if let Some(explicit_ty) = &c.explicit_type {
                    self.resolve_type(explicit_ty);
                }
                self.resolve_expr(&c.value);
            }
            Declaration::Import(_) => {
                // Structural import already handled in pass 1
            }
        }
    }

    fn resolve_function(&mut self, f: &FunctionDecl) {
        let fn_scope = self
            .scopes
            .alloc(ScopeKind::Function, Some(self.current_scope));
        let prev_scope = self.current_scope;
        self.current_scope = fn_scope;

        // Register generic type parameters
        for param in &f.generics.params {
            for bound in &param.bounds {
                self.resolve_trait_bound(bound);
            }
            let sym = self.symbols.alloc(
                &param.name.name,
                SymbolKind::TypeParam,
                param.name.span,
                Visibility::Private,
            );
            if let Err(prev) = self
                .scopes
                .define_type(self.current_scope, &param.name.name, sym)
            {
                self.emit_duplicate(&param.name.name, param.name.span, prev);
            }
        }

        // Register formal parameters
        for param in &f.parameters {
            self.resolve_parameter(param);
        }

        // Resolve return type annotation
        if let Some(ret_ty) = &f.return_type {
            self.resolve_type(ret_ty);
        }

        // Resolve function body statements
        self.resolve_block_statements(&f.body);

        self.current_scope = prev_scope;
    }

    fn resolve_parameter(&mut self, param: &Parameter) {
        self.resolve_type(&param.param_type);
        if let Some(default_val) = &param.default_value {
            self.resolve_expr(default_val);
        }

        let sym = self.symbols.alloc(
            &param.name.name,
            SymbolKind::Parameter,
            param.name.span,
            Visibility::Private,
        );
        if let Err(prev) = self
            .scopes
            .define_value(self.current_scope, &param.name.name, sym)
        {
            self.emit_duplicate(&param.name.name, param.name.span, prev);
        }
    }

    fn resolve_struct(&mut self, s: &StructDecl) {
        let has_generics = !s.generics.is_empty();
        let prev_scope = self.current_scope;

        if has_generics {
            let struct_scope = self
                .scopes
                .alloc(ScopeKind::Block, Some(self.current_scope));
            self.current_scope = struct_scope;
            for param in &s.generics.params {
                for bound in &param.bounds {
                    self.resolve_trait_bound(bound);
                }
                let sym = self.symbols.alloc(
                    &param.name.name,
                    SymbolKind::TypeParam,
                    param.name.span,
                    Visibility::Private,
                );
                let _ = self
                    .scopes
                    .define_type(self.current_scope, &param.name.name, sym);
            }
        }

        for member in &s.members {
            match member {
                sumer_ast::declaration::StructMember::Field(f) => {
                    self.resolve_type(&f.field_type);
                    if let Some(default_val) = &f.default_value {
                        self.resolve_expr(default_val);
                    }
                }
                sumer_ast::declaration::StructMember::Method(m) => {
                    self.resolve_function(m);
                }
            }
        }

        self.current_scope = prev_scope;
    }

    fn resolve_enum(&mut self, e: &EnumDecl) {
        let has_generics = !e.generics.is_empty();
        let prev_scope = self.current_scope;

        if has_generics {
            let enum_scope = self
                .scopes
                .alloc(ScopeKind::Block, Some(self.current_scope));
            self.current_scope = enum_scope;
            for param in &e.generics.params {
                for bound in &param.bounds {
                    self.resolve_trait_bound(bound);
                }
                let sym = self.symbols.alloc(
                    &param.name.name,
                    SymbolKind::TypeParam,
                    param.name.span,
                    Visibility::Private,
                );
                let _ = self
                    .scopes
                    .define_type(self.current_scope, &param.name.name, sym);
            }
        }

        for variant in &e.variants {
            match &variant.data {
                sumer_ast::declaration::VariantData::Unit => {}
                sumer_ast::declaration::VariantData::Tuple(types) => {
                    for ty in types {
                        self.resolve_type(ty);
                    }
                }
                sumer_ast::declaration::VariantData::Struct(fields) => {
                    for f in fields {
                        self.resolve_type(&f.field_type);
                    }
                }
            }
        }

        self.current_scope = prev_scope;
    }

    fn resolve_trait(&mut self, t: &TraitDecl) {
        let has_generics = !t.generics.is_empty();
        let prev_scope = self.current_scope;

        if has_generics {
            let trait_scope = self
                .scopes
                .alloc(ScopeKind::Block, Some(self.current_scope));
            self.current_scope = trait_scope;
            for param in &t.generics.params {
                for bound in &param.bounds {
                    self.resolve_trait_bound(bound);
                }
                let sym = self.symbols.alloc(
                    &param.name.name,
                    SymbolKind::TypeParam,
                    param.name.span,
                    Visibility::Private,
                );
                let _ = self
                    .scopes
                    .define_type(self.current_scope, &param.name.name, sym);
            }
        }

        for bound in &t.bounds {
            self.resolve_trait_bound(bound);
        }

        for member in &t.members {
            match member {
                sumer_ast::declaration::TraitMember::FunctionSignature(sig) => {
                    for param in &sig.parameters {
                        self.resolve_type(&param.param_type);
                    }
                    if let Some(ret_ty) = &sig.return_type {
                        self.resolve_type(ret_ty);
                    }
                }
                sumer_ast::declaration::TraitMember::Function(f) => {
                    self.resolve_function(f);
                }
            }
        }

        self.current_scope = prev_scope;
    }

    fn resolve_impl(&mut self, i: &ImplDecl) {
        let has_generics = !i.generics.is_empty();
        let prev_scope = self.current_scope;

        if has_generics {
            let impl_scope = self
                .scopes
                .alloc(ScopeKind::Block, Some(self.current_scope));
            self.current_scope = impl_scope;
            for param in &i.generics.params {
                for bound in &param.bounds {
                    self.resolve_trait_bound(bound);
                }
                let sym = self.symbols.alloc(
                    &param.name.name,
                    SymbolKind::TypeParam,
                    param.name.span,
                    Visibility::Private,
                );
                let _ = self
                    .scopes
                    .define_type(self.current_scope, &param.name.name, sym);
            }
        }

        if let Some(tr) = &i.trait_type {
            self.resolve_impl_trait(tr);
        }
        self.resolve_type(&i.target_type);

        for method in &i.members {
            self.resolve_function(method);
        }

        self.current_scope = prev_scope;
    }

    // =========================================================================
    // Statement Resolution
    // =========================================================================

    fn resolve_block_statements(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Variable(v) => {
                if let Some(explicit_ty) = &v.explicit_type {
                    self.resolve_type(explicit_ty);
                }
                // Section 12: Analyze initializer in the current scope BEFORE defining the new local
                self.resolve_expr(&v.initializer);

                let sym = self.symbols.alloc(
                    &v.name.name,
                    SymbolKind::Variable,
                    v.name.span,
                    v.visibility,
                );
                if let Err(prev) = self
                    .scopes
                    .define_value(self.current_scope, &v.name.name, sym)
                {
                    self.emit_duplicate(&v.name.name, v.name.span, prev);
                }
            }
            StmtKind::Constant(c) => {
                if let Some(explicit_ty) = &c.explicit_type {
                    self.resolve_type(explicit_ty);
                }
                // Analyze constant value before binding
                self.resolve_expr(&c.value);

                let sym = self.symbols.alloc(
                    &c.name.name,
                    SymbolKind::Constant,
                    c.name.span,
                    c.visibility,
                );
                if let Err(prev) = self
                    .scopes
                    .define_value(self.current_scope, &c.name.name, sym)
                {
                    self.emit_duplicate(&c.name.name, c.name.span, prev);
                }
            }
            StmtKind::Expression(expr_stmt) => {
                self.resolve_expr(&expr_stmt.expr);
            }
            StmtKind::Return(ret_stmt) => {
                if let Some(val) = &ret_stmt.value {
                    self.resolve_expr(val);
                }
            }
            StmtKind::If(if_stmt) => {
                self.resolve_if_stmt(if_stmt);
            }
            StmtKind::While(while_stmt) => {
                self.resolve_expr(&while_stmt.condition);
                let loop_scope = self
                    .scopes
                    .alloc(ScopeKind::Block, Some(self.current_scope));
                let prev_scope = self.current_scope;
                self.current_scope = loop_scope;
                self.resolve_block_statements(&while_stmt.body);
                self.current_scope = prev_scope;
            }
            StmtKind::For(for_stmt) => {
                self.resolve_expr(&for_stmt.iterable);
                let loop_scope = self
                    .scopes
                    .alloc(ScopeKind::Block, Some(self.current_scope));
                let prev_scope = self.current_scope;
                self.current_scope = loop_scope;

                // Bind iteration variable in loop block scope
                let sym = self.symbols.alloc(
                    &for_stmt.variable.name,
                    SymbolKind::Variable,
                    for_stmt.variable.span,
                    Visibility::Private,
                );
                let _ = self
                    .scopes
                    .define_value(self.current_scope, &for_stmt.variable.name, sym);

                self.resolve_block_statements(&for_stmt.body);
                self.current_scope = prev_scope;
            }
            StmtKind::Loop(loop_stmt) => {
                let loop_scope = self
                    .scopes
                    .alloc(ScopeKind::Block, Some(self.current_scope));
                let prev_scope = self.current_scope;
                self.current_scope = loop_scope;
                self.resolve_block_statements(&loop_stmt.body);
                self.current_scope = prev_scope;
            }
            StmtKind::Match(match_stmt) => {
                self.resolve_expr(&match_stmt.value);
                for arm in &match_stmt.arms {
                    self.resolve_match_arm(arm);
                }
            }
            StmtKind::Break(_) | StmtKind::Continue(_) => {}
            StmtKind::Block(b) => {
                let block_scope = self
                    .scopes
                    .alloc(ScopeKind::Block, Some(self.current_scope));
                let prev_scope = self.current_scope;
                self.current_scope = block_scope;
                self.resolve_block_statements(b);
                self.current_scope = prev_scope;
            }
        }
    }

    fn resolve_if_stmt(&mut self, if_stmt: &IfStmt) {
        self.resolve_expr(&if_stmt.condition);

        let then_scope = self
            .scopes
            .alloc(ScopeKind::Block, Some(self.current_scope));
        let prev_scope = self.current_scope;
        self.current_scope = then_scope;
        self.resolve_block_statements(&if_stmt.then_branch);
        self.current_scope = prev_scope;

        if let Some(else_branch) = &if_stmt.else_branch {
            match else_branch {
                ElseStmtBranch::Block(b) => {
                    let else_scope = self
                        .scopes
                        .alloc(ScopeKind::Block, Some(self.current_scope));
                    let prev = self.current_scope;
                    self.current_scope = else_scope;
                    self.resolve_block_statements(b);
                    self.current_scope = prev;
                }
                ElseStmtBranch::ElseIf(nested_if) => {
                    self.resolve_if_stmt(nested_if);
                }
            }
        }
    }

    fn resolve_if_expr(&mut self, if_expr: &IfExpr) {
        self.resolve_expr(&if_expr.condition);

        let then_scope = self
            .scopes
            .alloc(ScopeKind::Block, Some(self.current_scope));
        let prev_scope = self.current_scope;
        self.current_scope = then_scope;
        self.resolve_block_statements(&if_expr.then_branch);
        self.current_scope = prev_scope;

        if let Some(else_branch) = &if_expr.else_branch {
            match else_branch {
                ElseExprBranch::Block(b) => {
                    let else_scope = self
                        .scopes
                        .alloc(ScopeKind::Block, Some(self.current_scope));
                    let prev = self.current_scope;
                    self.current_scope = else_scope;
                    self.resolve_block_statements(b);
                    self.current_scope = prev;
                }
                ElseExprBranch::ElseIf(nested_if) => {
                    self.resolve_if_expr(nested_if);
                }
            }
        }
    }

    fn resolve_match_arm(&mut self, arm: &MatchArm) {
        let arm_scope = self
            .scopes
            .alloc(ScopeKind::MatchArm, Some(self.current_scope));
        let prev_scope = self.current_scope;
        self.current_scope = arm_scope;

        self.bind_pattern(&arm.pattern);
        self.resolve_expr(&arm.body);

        self.current_scope = prev_scope;
    }

    fn bind_pattern(&mut self, pattern: &Pattern) {
        match &pattern.kind {
            PatternKind::Wildcard | PatternKind::Literal(_) => {}
            PatternKind::Identifier(ident) => {
                let sym = self.symbols.alloc(
                    &ident.name,
                    SymbolKind::Variable,
                    ident.span,
                    Visibility::Private,
                );
                if let Err(prev) = self
                    .scopes
                    .define_value(self.current_scope, &ident.name, sym)
                {
                    self.emit_duplicate(&ident.name, ident.span, prev);
                }
            }
            PatternKind::Enum { path, data, .. } => {
                self.resolve_type_path(path);
                if let Some(sub_patterns) = data {
                    for p in sub_patterns {
                        self.bind_pattern(p);
                    }
                }
            }
            PatternKind::Struct { path, fields } => {
                self.resolve_type_path(path);
                for field in fields {
                    if let Some(sub_pat) = &field.pattern {
                        self.bind_pattern(sub_pat);
                    } else {
                        let sym = self.symbols.alloc(
                            &field.name.name,
                            SymbolKind::Variable,
                            field.name.span,
                            Visibility::Private,
                        );
                        if let Err(prev) =
                            self.scopes
                                .define_value(self.current_scope, &field.name.name, sym)
                        {
                            self.emit_duplicate(&field.name.name, field.name.span, prev);
                        }
                    }
                }
            }
            PatternKind::Tuple(elements) => {
                for elem in elements {
                    self.bind_pattern(elem);
                }
            }
        }
    }

    // =========================================================================
    // Expression Resolution
    // =========================================================================

    fn resolve_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(Literal::None) | ExprKind::Literal(_) => {}
            ExprKind::Identifier(ident) => {
                if let Some(sym_id) = self.scopes.lookup_value(self.current_scope, &ident.name) {
                    self.resolutions.insert(ident.span, sym_id);
                } else {
                    self.diagnostics.push(
                        SemanticError::UnresolvedIdentifier {
                            name: ident.name.clone(),
                            span: ident.span,
                        }
                        .to_diagnostic(),
                    );
                }
            }
            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            ExprKind::Unary { operand, .. } => {
                self.resolve_expr(operand);
            }
            ExprKind::Assignment { target, value, .. } => {
                self.resolve_expr(target);
                self.resolve_expr(value);
            }
            ExprKind::Call { callee, arguments } => {
                self.resolve_expr(callee);
                for arg in arguments {
                    self.resolve_expr(&arg.value);
                }
            }
            ExprKind::Member { object, .. } | ExprKind::OptionalMember { object, .. } => {
                // Section 10: resolve object only, do NOT resolve member against lexical scope
                self.resolve_expr(object);
            }
            ExprKind::Index { object, index } => {
                self.resolve_expr(object);
                self.resolve_expr(index);
            }
            ExprKind::StructInit { name, fields } => {
                self.resolve_type_path(name);
                for field in fields {
                    self.resolve_expr(&field.value);
                }
            }
            ExprKind::Array(elements) => {
                for el in elements {
                    self.resolve_expr(el);
                }
            }
            ExprKind::Map(entries) => {
                for entry in entries {
                    self.resolve_expr(&entry.key);
                    self.resolve_expr(&entry.value);
                }
            }
            ExprKind::Lambda { parameters, body } => {
                let lambda_scope = self
                    .scopes
                    .alloc(ScopeKind::Lambda, Some(self.current_scope));
                let prev_scope = self.current_scope;
                self.current_scope = lambda_scope;

                for param in parameters {
                    let sym = self.symbols.alloc(
                        &param.name,
                        SymbolKind::Parameter,
                        param.span,
                        Visibility::Private,
                    );
                    if let Err(prev) =
                        self.scopes
                            .define_value(self.current_scope, &param.name, sym)
                    {
                        self.emit_duplicate(&param.name, param.span, prev);
                    }
                }

                self.resolve_expr(body);
                self.current_scope = prev_scope;
            }
            ExprKind::If(if_expr) => {
                self.resolve_if_expr(if_expr);
            }
            ExprKind::Match { value, arms } => {
                self.resolve_expr(value);
                for arm in arms {
                    self.resolve_match_arm(arm);
                }
            }
            ExprKind::Await(e)
            | ExprKind::Spawn(e)
            | ExprKind::Reference { operand: e, .. }
            | ExprKind::ForceUnwrap(e) => {
                self.resolve_expr(e);
            }
            ExprKind::Block(b) => {
                let block_scope = self
                    .scopes
                    .alloc(ScopeKind::Block, Some(self.current_scope));
                let prev_scope = self.current_scope;
                self.current_scope = block_scope;
                self.resolve_block_statements(b);
                self.current_scope = prev_scope;
            }
        }
    }

    // =========================================================================
    // Type Resolution
    // =========================================================================

    fn resolve_type(&mut self, ty: &Type) {
        match &ty.kind {
            TypeKind::Named(path) => {
                self.resolve_type_path(path);
            }
            TypeKind::Generic { name, arguments } => {
                self.resolve_type_path(name);
                for arg in arguments {
                    self.resolve_type(arg);
                }
            }
            TypeKind::Reference { inner }
            | TypeKind::MutableReference { inner }
            | TypeKind::Optional { inner } => {
                self.resolve_type(inner);
            }
            TypeKind::Function {
                parameters,
                return_type,
            } => {
                for p in parameters {
                    self.resolve_type(p);
                }
                self.resolve_type(return_type);
            }
            TypeKind::Tuple { elements } => {
                for el in elements {
                    self.resolve_type(el);
                }
            }
            TypeKind::Array { element, size } => {
                self.resolve_type(element);
                if let Some(sz) = size {
                    self.resolve_expr(sz);
                }
            }
        }
    }

    fn resolve_type_path(&mut self, path: &TypePath) {
        if let Some(first_seg) = path.segments.first() {
            if let Some(sym_id) = self.scopes.lookup_type(self.current_scope, &first_seg.name) {
                self.resolutions.insert(path.span, sym_id);
            } else {
                self.diagnostics.push(
                    SemanticError::UnresolvedType {
                        name: first_seg.name.clone(),
                        span: path.span,
                    }
                    .to_diagnostic(),
                );
            }
        }
    }

    fn resolve_trait_bound(&mut self, bound: &TypeBound) {
        if let Some(sym_id) = self.scopes.lookup_type(self.current_scope, &bound.name.name) {
            let is_trait = self
                .symbols
                .get(sym_id)
                .map(|s| s.kind == SymbolKind::Trait)
                .unwrap_or(false);
            if is_trait {
                self.resolutions.insert(bound.span, sym_id);
                self.resolutions.insert(bound.name.span, sym_id);
            } else {
                self.diagnostics.push(
                    SemanticError::UnknownTrait {
                        name: bound.name.name.clone(),
                        span: bound.span,
                    }
                    .to_diagnostic(),
                );
            }
        } else {
            self.diagnostics.push(
                SemanticError::UnknownTrait {
                    name: bound.name.name.clone(),
                    span: bound.span,
                }
                .to_diagnostic(),
            );
        }
    }

    fn resolve_impl_trait(&mut self, ty: &Type) {
        let (name_str, span) = match &ty.kind {
            TypeKind::Named(path) => {
                if let Some(seg) = path.segments.first() {
                    (seg.name.clone(), path.span)
                } else {
                    return;
                }
            }
            TypeKind::Generic { name, arguments } => {
                for arg in arguments {
                    self.resolve_type(arg);
                }
                if let Some(seg) = name.segments.first() {
                    (seg.name.clone(), name.span)
                } else {
                    return;
                }
            }
            _ => {
                self.resolve_type(ty);
                return;
            }
        };

        if let Some(sym_id) = self.scopes.lookup_type(self.current_scope, &name_str) {
            let is_trait = self
                .symbols
                .get(sym_id)
                .map(|s| s.kind == SymbolKind::Trait)
                .unwrap_or(false);
            if is_trait {
                self.resolutions.insert(span, sym_id);
            } else {
                self.diagnostics.push(
                    SemanticError::UnknownTrait {
                        name: name_str,
                        span,
                    }
                    .to_diagnostic(),
                );
            }
        } else {
            self.diagnostics.push(
                SemanticError::UnknownTrait {
                    name: name_str,
                    span,
                }
                .to_diagnostic(),
            );
        }
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    fn emit_duplicate(&mut self, name: &str, span: Span, prev_sym_id: SymbolId) {
        let previous_span = self
            .symbols
            .get(prev_sym_id)
            .map(|s| s.span)
            .unwrap_or(span);

        self.diagnostics.push(
            SemanticError::DuplicateDeclaration {
                name: name.to_string(),
                span,
                previous_span,
            }
            .to_diagnostic(),
        );
    }
}
