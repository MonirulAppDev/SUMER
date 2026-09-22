//! Semantic type checker and type environment.

use std::collections::HashMap;

use sumer_ast::declaration::{
    Declaration, EnumDecl, FieldDecl, FunctionDecl, ImplDecl, Parameter, StructDecl,
    StructMember, TraitDecl, TraitMember,
};
use sumer_ast::expression::{
    Argument, BinaryOperator, ElseExprBranch, Expr, ExprKind, IfExpr, Literal, UnaryOperator,
};
use sumer_ast::program::Program;
use sumer_ast::statement::{Block, ElseStmtBranch, IfStmt, Stmt, StmtKind};
use sumer_ast::types::{Type as AstType, TypeKind as AstTypeKind};
use sumer_diagnostics::Diagnostics;
use sumer_span::Span;
use sumer_types::{GenericParamId, Type, TypeId, TypeStore, TypeSubstitution};

use crate::error::SemanticError;
use crate::resolver::Resolver;
use crate::symbol::{SymbolId, SymbolKind};
use crate::trait_system::{
    GenericParamDef, InherentImplDef, TraitBoundDef, TraitDef, TraitId, TraitImplDef,
    TraitMethodDef, TraitRegistry,
};

/// Stores the results of semantic type checking.
#[derive(Clone, Debug)]
pub struct TypeckResult {
    /// Inferred or validated semantic type for each expression span.
    pub expression_types: HashMap<Span, TypeId>,
    /// Semantic type associated with each declared symbol.
    pub symbol_types: HashMap<SymbolId, TypeId>,
    /// Interned semantic type arena.
    pub type_store: TypeStore,
    /// Diagnostics accumulated during type checking.
    pub diagnostics: Diagnostics,
    /// Trait metadata and implementation registry.
    pub traits: TraitRegistry,
}

impl TypeckResult {
    /// Returns `true` if any diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

/// Parameter metadata: `(name, type, has_default_value)`.
pub type FnParamInfo = (Option<String>, TypeId, bool);

/// Function signature metadata: `(parameter_details, return_type)`.
pub type FnSignature = (Vec<FnParamInfo>, TypeId);

/// Semantic definition of a struct type including generic parameters and fields.
#[derive(Clone, Debug)]
pub struct StructDef {
    /// Name of the struct.
    pub name: String,
    /// Generic type parameters declared on the struct.
    pub generic_params: Vec<GenericParamId>,
    /// Fields and their semantic types (in terms of generic parameters).
    pub fields: HashMap<String, TypeId>,
}

/// Extended function metadata preserving generic parameters and bounds.
#[derive(Clone, Debug)]
pub struct FnDef {
    /// Generic type parameters.
    pub generic_params: Vec<GenericParamId>,
    /// Trait bounds on generic parameters: `(param_id, bound_names)`.
    pub generic_bounds: Vec<(GenericParamId, Vec<String>)>,
    /// Formal parameter details.
    pub param_details: Vec<FnParamInfo>,
    /// Function return type.
    pub return_type: TypeId,
}

/// Type checking engine executing semantic type resolution and expression validation.
pub struct TypeChecker<'a> {
    resolver: &'a Resolver,
    pub type_store: TypeStore,
    pub symbol_types: HashMap<SymbolId, TypeId>,
    /// Details for each function symbol: (parameters: (name, type, has_default), return_type)
    pub fn_signatures: HashMap<SymbolId, FnSignature>,
    /// Full function definitions including generic parameters and bounds.
    pub fn_defs: HashMap<SymbolId, FnDef>,
    /// Declared struct definitions with field types.
    pub struct_defs: HashMap<String, StructDef>,
    /// Generic parameter counts (arity) for declared types and built-in types.
    pub generic_arities: HashMap<String, usize>,
    /// Mapping from generic parameter SymbolId to its stable GenericParamId.
    pub generic_param_symbols: HashMap<SymbolId, GenericParamId>,
    /// Central trait and implementation registry.
    pub trait_registry: TraitRegistry,
    pub current_fn_return: Option<TypeId>,
    pub expression_types: HashMap<Span, TypeId>,
    pub diagnostics: Diagnostics,
}

impl<'a> TypeChecker<'a> {
    /// Creates a new `TypeChecker` bound to the resolved AST symbols and scopes.
    pub fn new(resolver: &'a Resolver) -> Self {
        let mut generic_arities = HashMap::new();
        generic_arities.insert("Option".to_string(), 1);
        generic_arities.insert("List".to_string(), 1);
        generic_arities.insert("Map".to_string(), 2);
        generic_arities.insert("Set".to_string(), 1);
        generic_arities.insert("Result".to_string(), 2);

        Self {
            resolver,
            type_store: TypeStore::new(),
            symbol_types: HashMap::new(),
            fn_signatures: HashMap::new(),
            fn_defs: HashMap::new(),
            struct_defs: HashMap::new(),
            generic_arities,
            generic_param_symbols: HashMap::new(),
            trait_registry: TraitRegistry::new(),
            current_fn_return: None,
            expression_types: HashMap::new(),
            diagnostics: Diagnostics::new(),
        }
    }

    /// Performs complete type checking over the AST [`Program`].
    pub fn check_program(mut self, program: &Program) -> TypeckResult {
        // Pass 1: Collect top-level item types, generic parameters & signatures
        self.collect_item_types(&program.declarations);

        // Pass 2: Type check declaration bodies
        for decl in &program.declarations {
            self.check_declaration(decl);
        }

        TypeckResult {
            expression_types: self.expression_types,
            symbol_types: self.symbol_types,
            type_store: self.type_store,
            diagnostics: self.diagnostics,
            traits: self.trait_registry,
        }
    }

    // =========================================================================
    // Pass 1: Item Type Collection
    // =========================================================================

    fn collect_item_types(&mut self, declarations: &[Declaration]) {
        // Step 1: Pre-collect generic parameters & arities across all declarations
        for decl in declarations {
            match decl {
                Declaration::Function(f) => {
                    self.generic_arities
                        .insert(f.name.name.clone(), f.generics.params.len());
                    for p in &f.generics.params {
                        let pid = self.type_store.alloc_generic_param(&p.name.name);
                        let ty = self.type_store.intern(Type::GenericParam(pid));
                        if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                            self.symbol_types.insert(sym, ty);
                            self.generic_param_symbols.insert(sym, pid);
                        }
                    }
                }
                Declaration::Struct(s) => {
                    self.generic_arities
                        .insert(s.name.name.clone(), s.generics.params.len());
                    for p in &s.generics.params {
                        let pid = self.type_store.alloc_generic_param(&p.name.name);
                        let ty = self.type_store.intern(Type::GenericParam(pid));
                        if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                            self.symbol_types.insert(sym, ty);
                            self.generic_param_symbols.insert(sym, pid);
                        }
                    }
                    for member in &s.members {
                        if let StructMember::Method(m) = member {
                            self.generic_arities
                                .insert(m.name.name.clone(), m.generics.params.len());
                            for p in &m.generics.params {
                                let pid = self.type_store.alloc_generic_param(&p.name.name);
                                let ty = self.type_store.intern(Type::GenericParam(pid));
                                if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                                    self.symbol_types.insert(sym, ty);
                                    self.generic_param_symbols.insert(sym, pid);
                                }
                            }
                        }
                    }
                }
                Declaration::Enum(e) => {
                    self.generic_arities
                        .insert(e.name.name.clone(), e.generics.params.len());
                    for p in &e.generics.params {
                        let pid = self.type_store.alloc_generic_param(&p.name.name);
                        let ty = self.type_store.intern(Type::GenericParam(pid));
                        if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                            self.symbol_types.insert(sym, ty);
                            self.generic_param_symbols.insert(sym, pid);
                        }
                    }
                }
                Declaration::Trait(t) => {
                    self.generic_arities
                        .insert(t.name.name.clone(), t.generics.params.len());
                    for p in &t.generics.params {
                        let pid = self.type_store.alloc_generic_param(&p.name.name);
                        let ty = self.type_store.intern(Type::GenericParam(pid));
                        if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                            self.symbol_types.insert(sym, ty);
                            self.generic_param_symbols.insert(sym, pid);
                        }
                    }
                }
                Declaration::Impl(i) => {
                    for p in &i.generics.params {
                        let pid = self.type_store.alloc_generic_param(&p.name.name);
                        let ty = self.type_store.intern(Type::GenericParam(pid));
                        if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                            self.symbol_types.insert(sym, ty);
                            self.generic_param_symbols.insert(sym, pid);
                        }
                    }
                    for member in &i.members {
                        self.generic_arities
                            .insert(member.name.name.clone(), member.generics.params.len());
                        for p in &member.generics.params {
                            let pid = self.type_store.alloc_generic_param(&p.name.name);
                            let ty = self.type_store.intern(Type::GenericParam(pid));
                            if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                                self.symbol_types.insert(sym, ty);
                                self.generic_param_symbols.insert(sym, pid);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Step 2: Register traits into TraitRegistry
        for decl in declarations {
            if let Declaration::Trait(t) = decl {
                let trait_id = self.trait_registry.next_id();
                if let Some(sym_id) = self.find_symbol_by_span(t.name.span) {
                    let named_ty = self.type_store.intern(Type::Named(t.name.name.clone()));
                    self.symbol_types.insert(sym_id, named_ty);
                }

                let mut generic_params = Vec::new();
                for p in &t.generics.params {
                    let pid = self
                        .find_symbol_by_span(p.name.span)
                        .and_then(|sym| self.generic_param_symbols.get(&sym).copied())
                        .unwrap_or(GenericParamId(0));
                    let bounds = p
                        .bounds
                        .iter()
                        .map(|b| TraitBoundDef::new(&b.name.name, None, b.span))
                        .collect();
                    generic_params.push(GenericParamDef::new(pid, &p.name.name, bounds, p.span));
                }

                let mut methods = Vec::new();
                for member in &t.members {
                    match member {
                        TraitMember::FunctionSignature(sig) => {
                            let mut m_params = Vec::new();
                            for p in &sig.parameters {
                                let p_ty = self.resolve_ast_type(&p.param_type);
                                m_params.push((p.name.name.clone(), p_ty));
                            }
                            let ret_ty = sig
                                .return_type
                                .as_ref()
                                .map(|r| self.resolve_ast_type(r))
                                .unwrap_or(TypeId::UNIT);
                            methods.push(TraitMethodDef::new(
                                &sig.name.name,
                                Vec::new(),
                                m_params,
                                ret_ty,
                                false,
                                sig.span,
                            ));
                        }
                        TraitMember::Function(f) => {
                            self.register_function_signature(f);
                            let mut m_params = Vec::new();
                            for p in &f.parameters {
                                let p_ty = self.resolve_ast_type(&p.param_type);
                                m_params.push((p.name.name.clone(), p_ty));
                            }
                            let ret_ty = f
                                .return_type
                                .as_ref()
                                .map(|r| self.resolve_ast_type(r))
                                .unwrap_or(TypeId::UNIT);
                            methods.push(TraitMethodDef::new(
                                &f.name.name,
                                Vec::new(),
                                m_params,
                                ret_ty,
                                true,
                                f.span,
                            ));
                        }
                    }
                }

                let bounds = t
                    .bounds
                    .iter()
                    .map(|b| TraitBoundDef::new(&b.name.name, None, b.span))
                    .collect();

                let trait_def = TraitDef::new(
                    trait_id,
                    t.name.name.clone(),
                    generic_params,
                    methods,
                    bounds,
                    t.span,
                );
                let _ = self.trait_registry.register_trait(trait_def);
            }
        }

        // Step 3: Register structs, functions, enums, impl blocks, variables, constants
        for decl in declarations {
            match decl {
                Declaration::Function(f) => {
                    self.register_function_signature(f);
                }
                Declaration::Struct(s) => {
                    if let Some(sym_id) = self.find_symbol_by_span(s.name.span) {
                        let named_ty = self.type_store.intern(Type::Named(s.name.name.clone()));
                        self.symbol_types.insert(sym_id, named_ty);
                    }
                    let mut fields = HashMap::new();
                    for member in &s.members {
                        match member {
                            StructMember::Field(f) => {
                                let f_ty = self.resolve_ast_type(&f.field_type);
                                fields.insert(f.name.name.clone(), f_ty);
                            }
                            StructMember::Method(m) => {
                                self.register_function_signature(m);
                            }
                        }
                    }
                    let mut generic_params = Vec::new();
                    for p in &s.generics.params {
                        if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                            if let Some(&pid) = self.generic_param_symbols.get(&sym) {
                                generic_params.push(pid);
                            }
                        }
                    }
                    self.struct_defs.insert(
                        s.name.name.clone(),
                        StructDef {
                            name: s.name.name.clone(),
                            generic_params,
                            fields,
                        },
                    );
                }
                Declaration::Enum(e) => {
                    if let Some(sym_id) = self.find_symbol_by_span(e.name.span) {
                        let named_ty = self.type_store.intern(Type::Named(e.name.name.clone()));
                        self.symbol_types.insert(sym_id, named_ty);
                    }
                }
                Declaration::Trait(_) => {
                    // Handled in Step 2
                }
                Declaration::Impl(i) => {
                    for member in &i.members {
                        self.register_function_signature(member);
                    }

                    let target_ty = self.resolve_ast_type(&i.target_type);
                    let mut generic_params = Vec::new();
                    for p in &i.generics.params {
                        if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                            if let Some(&pid) = self.generic_param_symbols.get(&sym) {
                                generic_params.push(GenericParamDef::new(
                                    pid,
                                    &p.name.name,
                                    Vec::new(),
                                    p.span,
                                ));
                            }
                        }
                    }
                    let method_names: Vec<String> =
                        i.members.iter().map(|m| m.name.name.clone()).collect();

                    if let Some(trait_type) = &i.trait_type {
                        let trait_name_opt = match &trait_type.kind {
                            AstTypeKind::Named(path) => {
                                path.segments.first().map(|s| s.name.clone())
                            }
                            AstTypeKind::Generic { name, .. } => {
                                name.segments.first().map(|s| s.name.clone())
                            }
                            _ => None,
                        };

                        if let Some(trait_name) = trait_name_opt {
                            if let Some(tdef) = self.trait_registry.lookup_trait(&trait_name) {
                                let trait_id = tdef.id;
                                let impl_def = TraitImplDef::new(
                                    trait_id,
                                    trait_name.clone(),
                                    target_ty,
                                    generic_params,
                                    method_names,
                                    i.span,
                                );
                                if let Err(existing) =
                                    self.trait_registry.register_impl(impl_def)
                                {
                                    self.diagnostics.push(
                                        SemanticError::DuplicateTraitImpl {
                                            trait_name,
                                            target_type: self.type_store.format_type(target_ty),
                                            span: i.span,
                                            previous_span: existing.span,
                                        }
                                        .to_diagnostic(),
                                    );
                                }
                            }
                        }
                    } else {
                        let inherent =
                            InherentImplDef::new(target_ty, generic_params, method_names, i.span);
                        self.trait_registry.register_inherent_impl(inherent);
                    }
                }
                Declaration::Variable(v) => {
                    if let Some(explicit_ty) = &v.explicit_type {
                        let ty_id = self.resolve_ast_type(explicit_ty);
                        if let Some(&sym_id) = self.resolver.resolutions.get(&v.name.span) {
                            self.symbol_types.insert(sym_id, ty_id);
                        }
                    }
                }
                Declaration::Constant(c) => {
                    if let Some(explicit_ty) = &c.explicit_type {
                        let ty_id = self.resolve_ast_type(explicit_ty);
                        if let Some(&sym_id) = self.resolver.resolutions.get(&c.name.span) {
                            self.symbol_types.insert(sym_id, ty_id);
                        }
                    }
                }
                Declaration::Import(_) => {}
            }
        }
    }

    fn register_function_signature(&mut self, f: &FunctionDecl) {
        let return_ty = if let Some(ret_ast) = &f.return_type {
            self.resolve_ast_type(ret_ast)
        } else {
            TypeId::UNIT
        };

        let mut param_types = Vec::new();
        let mut param_details = Vec::new();

        for param in &f.parameters {
            let p_ty = self.resolve_ast_type(&param.param_type);
            param_types.push(p_ty);
            param_details.push((
                Some(param.name.name.clone()),
                p_ty,
                param.default_value.is_some(),
            ));

            // Register parameter symbol type
            if let Some(param_sym) = self.find_symbol_by_span(param.name.span) {
                self.symbol_types.insert(param_sym, p_ty);
            }
        }

        let fn_ty = self.type_store.intern(Type::Function {
            params: param_types,
            return_type: return_ty,
        });

        let mut generic_params = Vec::new();
        let mut generic_bounds = Vec::new();
        for p in &f.generics.params {
            if let Some(sym) = self.find_symbol_by_span(p.name.span) {
                if let Some(&pid) = self.generic_param_symbols.get(&sym) {
                    generic_params.push(pid);
                    let bounds: Vec<String> =
                        p.bounds.iter().map(|b| b.name.name.clone()).collect();
                    generic_bounds.push((pid, bounds));
                }
            }
        }

        if let Some(fn_sym) = self.find_symbol_by_span(f.name.span) {
            self.symbol_types.insert(fn_sym, fn_ty);
            self.fn_signatures
                .insert(fn_sym, (param_details.clone(), return_ty));
            self.fn_defs.insert(
                fn_sym,
                FnDef {
                    generic_params,
                    generic_bounds,
                    param_details,
                    return_type,
                },
            );
        }
    }

    // =========================================================================
    // Pass 2: Declaration Checking
    // =========================================================================

    fn check_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Function(f) => {
                self.check_function(f);
            }
            Declaration::Struct(s) => {
                for member in &s.members {
                    if let StructMember::Method(m) = member {
                        self.check_function(m);
                    }
                }
            }
            Declaration::Enum(_) | Declaration::Trait(_) => {}
            Declaration::Impl(i) => {
                for member in &i.members {
                    self.check_function(member);
                }
            }
            Declaration::Variable(v) => {
                let expected = v.explicit_type.as_ref().map(|t| self.resolve_ast_type(t));
                let init_ty = self.check_expr(&v.initializer, expected);

                if let Some(exp) = expected {
                    if !self.type_store.is_assignable(exp, init_ty) {
                        self.emit_mismatch(exp, init_ty, v.initializer.span);
                    }
                } else if let Some(sym) = self.find_symbol_by_span(v.name.span) {
                    self.symbol_types.insert(sym, init_ty);
                }
            }
            Declaration::Constant(c) => {
                let expected = c.explicit_type.as_ref().map(|t| self.resolve_ast_type(t));
                let val_ty = self.check_expr(&c.value, expected);

                if let Some(exp) = expected {
                    if !self.type_store.is_assignable(exp, val_ty) {
                        self.emit_mismatch(exp, val_ty, c.value.span);
                    }
                } else if let Some(sym) = self.find_symbol_by_span(c.name.span) {
                    self.symbol_types.insert(sym, val_ty);
                }
            }
            Declaration::Import(_) => {}
        }
    }

    fn check_function(&mut self, f: &FunctionDecl) {
        let return_ty = if let Some(ret_ast) = &f.return_type {
            self.resolve_ast_type(ret_ast)
        } else {
            TypeId::UNIT
        };

        // Check parameter default value expressions
        for param in &f.parameters {
            if let Some(def_val) = &param.default_value {
                let param_ty = self.resolve_ast_type(&param.param_type);
                let val_ty = self.check_expr(def_val, Some(param_ty));
                if !self.type_store.is_assignable(param_ty, val_ty) {
                    self.emit_mismatch(param_ty, val_ty, def_val.span);
                }
            }
        }

        let prev_return = self.current_fn_return;
        self.current_fn_return = Some(return_ty);

        let body_ty = self.check_block(&f.body, Some(return_ty));

        // Implicit return checking: if function returns non-Unit and final expression does not match
        if return_ty != TypeId::UNIT
            && !self.block_has_explicit_return(&f.body)
            && !self.type_store.is_assignable(return_ty, body_ty)
        {
            let span = f
                .body
                .statements
                .last()
                .map(|s| s.span)
                .unwrap_or(f.body.span);
            self.diagnostics.push(
                SemanticError::ReturnTypeMismatch {
                    expected: self.type_store.format_type(return_ty),
                    found: self.type_store.format_type(body_ty),
                    span,
                }
                .to_diagnostic(),
            );
        }

        self.current_fn_return = prev_return;
    }

    fn block_has_explicit_return(&self, block: &Block) -> bool {
        for stmt in &block.statements {
            if matches!(stmt.kind, StmtKind::Return(_)) {
                return true;
            }
            if let StmtKind::Block(inner) = &stmt.kind
                && self.block_has_explicit_return(inner)
            {
                return true;
            }
        }
        false
    }

    // =========================================================================
    // Statement Checking
    // =========================================================================

    fn check_block(&mut self, block: &Block, expected: Option<TypeId>) -> TypeId {
        let mut last_ty = TypeId::UNIT;

        for (i, stmt) in block.statements.iter().enumerate() {
            let is_last = i == block.statements.len() - 1;
            if is_last && let StmtKind::Expression(expr_stmt) = &stmt.kind {
                last_ty = self.check_expr(&expr_stmt.expr, expected);
                continue;
            }
            self.check_stmt(stmt);
        }

        last_ty
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Variable(v) => {
                let expected = v.explicit_type.as_ref().map(|t| self.resolve_ast_type(t));
                let init_ty = self.check_expr(&v.initializer, expected);

                if let Some(exp) = expected {
                    if !self.type_store.is_assignable(exp, init_ty) {
                        self.emit_mismatch(exp, init_ty, v.initializer.span);
                    }
                    if let Some(sym) = self.find_symbol_by_span(v.name.span) {
                        self.symbol_types.insert(sym, exp);
                    }
                } else if let Some(sym) = self.find_symbol_by_span(v.name.span) {
                    self.symbol_types.insert(sym, init_ty);
                }
            }
            StmtKind::Constant(c) => {
                let expected = c.explicit_type.as_ref().map(|t| self.resolve_ast_type(t));
                let val_ty = self.check_expr(&c.value, expected);

                if let Some(exp) = expected {
                    if !self.type_store.is_assignable(exp, val_ty) {
                        self.emit_mismatch(exp, val_ty, c.value.span);
                    }
                    if let Some(sym) = self.find_symbol_by_span(c.name.span) {
                        self.symbol_types.insert(sym, exp);
                    }
                } else if let Some(sym) = self.find_symbol_by_span(c.name.span) {
                    self.symbol_types.insert(sym, val_ty);
                }
            }
            StmtKind::Expression(e) => {
                self.check_expr(&e.expr, None);
            }
            StmtKind::Return(r) => {
                let expected = self.current_fn_return.unwrap_or(TypeId::UNIT);
                if let Some(val) = &r.value {
                    let val_ty = self.check_expr(val, Some(expected));
                    if !self.type_store.is_assignable(expected, val_ty) {
                        self.diagnostics.push(
                            SemanticError::ReturnTypeMismatch {
                                expected: self.type_store.format_type(expected),
                                found: self.type_store.format_type(val_ty),
                                span: val.span,
                            }
                            .to_diagnostic(),
                        );
                    }
                } else if expected != TypeId::UNIT {
                    self.diagnostics.push(
                        SemanticError::ReturnTypeMismatch {
                            expected: self.type_store.format_type(expected),
                            found: self.type_store.format_type(TypeId::UNIT),
                            span: r.span,
                        }
                        .to_diagnostic(),
                    );
                }
            }
            StmtKind::If(if_stmt) => {
                self.check_if_stmt(if_stmt);
            }
            StmtKind::While(while_stmt) => {
                let cond_ty = self.check_expr(&while_stmt.condition, Some(TypeId::BOOL));
                if cond_ty != TypeId::BOOL && cond_ty != TypeId::ERROR {
                    self.diagnostics.push(
                        SemanticError::ConditionTypeMismatch {
                            found: self.type_store.format_type(cond_ty),
                            span: while_stmt.condition.span,
                        }
                        .to_diagnostic(),
                    );
                }
                self.check_block(&while_stmt.body, None);
            }
            StmtKind::For(for_stmt) => {
                let iter_ty = self.check_expr(&for_stmt.iterable, None);
                let elem_ty = self.extract_iterable_element_type(iter_ty);

                if let Some(sym) = self.find_symbol_by_span(for_stmt.variable.span) {
                    self.symbol_types.insert(sym, elem_ty);
                }

                self.check_block(&for_stmt.body, None);
            }
            StmtKind::Loop(loop_stmt) => {
                self.check_block(&loop_stmt.body, None);
            }
            StmtKind::Match(match_stmt) => {
                self.check_expr(&match_stmt.value, None);
                for arm in &match_stmt.arms {
                    self.check_expr(&arm.body, None);
                }
            }
            StmtKind::Break(_) | StmtKind::Continue(_) => {}
            StmtKind::Block(b) => {
                self.check_block(b, None);
            }
        }
    }

    fn check_if_stmt(&mut self, if_stmt: &IfStmt) {
        let cond_ty = self.check_expr(&if_stmt.condition, Some(TypeId::BOOL));
        if cond_ty != TypeId::BOOL && cond_ty != TypeId::ERROR {
            self.diagnostics.push(
                SemanticError::ConditionTypeMismatch {
                    found: self.type_store.format_type(cond_ty),
                    span: if_stmt.condition.span,
                }
                .to_diagnostic(),
            );
        }
        self.check_block(&if_stmt.then_branch, None);

        if let Some(else_branch) = &if_stmt.else_branch {
            match else_branch {
                ElseStmtBranch::Block(b) => {
                    self.check_block(b, None);
                }
                ElseStmtBranch::ElseIf(nested_if) => {
                    self.check_if_stmt(nested_if);
                }
            }
        }
    }

    fn extract_iterable_element_type(&mut self, iter_ty: TypeId) -> TypeId {
        match self.type_store.get(iter_ty) {
            Type::Generic { name, args } if name == "Range" && !args.is_empty() => args[0],
            Type::List(elem) | Type::Array { element: elem } => *elem,
            _ => TypeId::INT,
        }
    }

    // =========================================================================
    // Expression Checking
    // =========================================================================

    pub fn check_expr(&mut self, expr: &Expr, expected: Option<TypeId>) -> TypeId {
        let ty = match &expr.kind {
            ExprKind::Literal(lit) => self.check_literal(lit, expected, expr.span),
            ExprKind::Identifier(ident) => {
                if let Some(&sym_id) = self.resolver.resolutions.get(&ident.span) {
                    if let Some(&ty) = self.symbol_types.get(&sym_id) {
                        ty
                    } else if let Some(sym) = self.resolver.symbols.get(sym_id) {
                        if sym.kind == SymbolKind::Function {
                            // Standard built-in function type
                            self.type_store.intern(Type::Function {
                                params: Vec::new(),
                                return_type: TypeId::UNIT,
                            })
                        } else {
                            TypeId::ERROR
                        }
                    } else {
                        TypeId::ERROR
                    }
                } else if ident.name == "print" || ident.name == "println" {
                    self.type_store.intern(Type::Function {
                        params: Vec::new(),
                        return_type: TypeId::UNIT,
                    })
                } else if ident.name == "Ok" || ident.name == "Some" || ident.name == "Err" {
                    TypeId::UNIT
                } else {
                    TypeId::ERROR
                }
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => self.check_binary(left, *operator, right, expr.span),

            ExprKind::Unary { operator, operand } => {
                self.check_unary(*operator, operand, expr.span)
            }

            ExprKind::Assignment { target, value, .. } => {
                let target_ty = self.check_expr(target, None);
                let val_ty = self.check_expr(value, Some(target_ty));
                if !self.type_store.is_assignable(target_ty, val_ty) {
                    self.emit_mismatch(target_ty, val_ty, value.span);
                }
                TypeId::UNIT
            }

            ExprKind::Call { callee, arguments } => {
                self.check_call(callee, arguments, expected, expr.span)
            }

            ExprKind::Member { object, .. } | ExprKind::OptionalMember { object, .. } => {
                let obj_ty = self.check_expr(object, None);
                if let Type::Named(_) = self.type_store.get(obj_ty) {
                    obj_ty
                } else {
                    expected.unwrap_or(TypeId::INT)
                }
            }

            ExprKind::Index { object, index } => {
                let obj_ty = self.check_expr(object, None);
                let _ = self.check_expr(index, Some(TypeId::INT));
                match self.type_store.get(obj_ty) {
                    Type::List(elem) | Type::Array { element: elem } => *elem,
                    Type::Map { value, .. } => *value,
                    _ => expected.unwrap_or(TypeId::INT),
                }
            }

            ExprKind::StructInit { name, fields } => {
                let _ = self.resolve_type_path(name);
                for field in fields {
                    self.check_expr(&field.value, None);
                }
                if let Some(first) = name.segments.first() {
                    self.type_store.intern(Type::Named(first.name.clone()))
                } else {
                    TypeId::ERROR
                }
            }

            ExprKind::Array(elements) => self.check_array(elements, expected, expr.span),

            ExprKind::Map(entries) => self.check_map(entries, expected),

            ExprKind::Lambda { parameters, body } => {
                let body_ty = self.check_expr(body, None);
                let param_types = vec![TypeId::INT; parameters.len()];
                self.type_store.intern(Type::Function {
                    params: param_types,
                    return_type: body_ty,
                })
            }

            ExprKind::If(if_expr) => self.check_if_expr(if_expr, expected, expr.span),

            ExprKind::Match { value, arms } => {
                self.check_expr(value, None);
                let mut arm_ty = expected.unwrap_or(TypeId::UNIT);
                for arm in arms {
                    arm_ty = self.check_expr(&arm.body, expected);
                }
                arm_ty
            }

            ExprKind::Await(operand) | ExprKind::Spawn(operand) => {
                self.check_expr(operand, expected)
            }

            ExprKind::Reference { mutable, operand } => {
                let inner = self.check_expr(operand, None);
                self.type_store.intern(Type::Reference {
                    mutable: *mutable,
                    inner,
                })
            }

            ExprKind::Block(b) => self.check_block(b, expected),

            ExprKind::ForceUnwrap(operand) => {
                let inner_opt = self.check_expr(operand, None);
                if let Type::Optional(unwrapped) = self.type_store.get(inner_opt) {
                    *unwrapped
                } else {
                    inner_opt
                }
            }
        };

        self.expression_types.insert(expr.span, ty);
        ty
    }

    fn check_literal(&mut self, lit: &Literal, expected: Option<TypeId>, _span: Span) -> TypeId {
        match lit {
            Literal::Integer(_) => {
                if let Some(exp) = expected
                    && self.type_store.get(exp).is_integer()
                {
                    return exp;
                }
                TypeId::INT
            }
            Literal::Float(_) => {
                if let Some(exp) = expected
                    && self.type_store.get(exp).is_float()
                {
                    return exp;
                }
                TypeId::FLOAT
            }
            Literal::String(_) => TypeId::STRING,
            Literal::Char(_) => TypeId::CHAR,
            Literal::Bool(_) => TypeId::BOOL,
            Literal::None => {
                if let Some(exp) = expected
                    && let Type::Optional(_) = self.type_store.get(exp)
                {
                    return exp;
                }
                self.type_store.intern(Type::Optional(TypeId::ERROR))
            }
        }
    }

    fn check_binary(
        &mut self,
        left: &Expr,
        op: BinaryOperator,
        right: &Expr,
        span: Span,
    ) -> TypeId {
        match op {
            BinaryOperator::Add => {
                let l_ty = self.check_expr(left, None);
                let r_ty = self.check_expr(right, Some(l_ty));

                if l_ty == TypeId::STRING && r_ty == TypeId::STRING {
                    return TypeId::STRING;
                }

                if self.type_store.get(l_ty).is_numeric()
                    && self.type_store.get(r_ty).is_numeric()
                    && l_ty == r_ty
                {
                    return l_ty;
                }

                self.emit_cannot_apply_binary("+", l_ty, r_ty, span);
                TypeId::ERROR
            }
            BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => {
                let op_str = match op {
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Modulo => "%",
                    _ => unreachable!(),
                };
                let l_ty = self.check_expr(left, None);
                let r_ty = self.check_expr(right, Some(l_ty));

                if self.type_store.get(l_ty).is_numeric()
                    && self.type_store.get(r_ty).is_numeric()
                    && l_ty == r_ty
                {
                    return l_ty;
                }

                self.emit_cannot_apply_binary(op_str, l_ty, r_ty, span);
                TypeId::ERROR
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                let l_ty = self.check_expr(left, None);
                let r_ty = self.check_expr(right, Some(l_ty));

                if !self.type_store.type_equals(l_ty, r_ty) {
                    let op_str = if op == BinaryOperator::Equal {
                        "=="
                    } else {
                        "!="
                    };
                    self.emit_cannot_apply_binary(op_str, l_ty, r_ty, span);
                }
                TypeId::BOOL
            }
            BinaryOperator::Greater
            | BinaryOperator::Less
            | BinaryOperator::GreaterEqual
            | BinaryOperator::LessEqual => {
                let op_str = match op {
                    BinaryOperator::Greater => ">",
                    BinaryOperator::Less => "<",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::LessEqual => "<=",
                    _ => unreachable!(),
                };
                let l_ty = self.check_expr(left, None);
                let r_ty = self.check_expr(right, Some(l_ty));

                let l_is_num = self.type_store.get(l_ty).is_numeric();
                let r_is_num = self.type_store.get(r_ty).is_numeric();

                if !l_is_num || !r_is_num || l_ty != r_ty {
                    self.emit_cannot_apply_binary(op_str, l_ty, r_ty, span);
                }
                TypeId::BOOL
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                let l_ty = self.check_expr(left, Some(TypeId::BOOL));
                let r_ty = self.check_expr(right, Some(TypeId::BOOL));

                if l_ty != TypeId::BOOL && l_ty != TypeId::ERROR {
                    self.diagnostics.push(
                        SemanticError::ConditionTypeMismatch {
                            found: self.type_store.format_type(l_ty),
                            span: left.span,
                        }
                        .to_diagnostic(),
                    );
                }
                if r_ty != TypeId::BOOL && r_ty != TypeId::ERROR {
                    self.diagnostics.push(
                        SemanticError::ConditionTypeMismatch {
                            found: self.type_store.format_type(r_ty),
                            span: right.span,
                        }
                        .to_diagnostic(),
                    );
                }
                TypeId::BOOL
            }
            BinaryOperator::Range | BinaryOperator::RangeInclusive => {
                let l_ty = self.check_expr(left, Some(TypeId::INT));
                let r_ty = self.check_expr(right, Some(l_ty));
                self.type_store.intern(Type::Generic {
                    name: "Range".to_string(),
                    args: vec![r_ty],
                })
            }
        }
    }

    fn check_unary(&mut self, op: UnaryOperator, operand: &Expr, span: Span) -> TypeId {
        match op {
            UnaryOperator::Not => {
                let op_ty = self.check_expr(operand, Some(TypeId::BOOL));
                if op_ty != TypeId::BOOL && op_ty != TypeId::ERROR {
                    self.diagnostics.push(
                        SemanticError::CannotApplyUnary {
                            op: "!".to_string(),
                            operand: self.type_store.format_type(op_ty),
                            span,
                        }
                        .to_diagnostic(),
                    );
                }
                TypeId::BOOL
            }
            UnaryOperator::Negate | UnaryOperator::Positive => {
                let op_str = if op == UnaryOperator::Negate {
                    "-"
                } else {
                    "+"
                };
                let op_ty = self.check_expr(operand, None);
                if !self.type_store.get(op_ty).is_numeric() && op_ty != TypeId::ERROR {
                    self.diagnostics.push(
                        SemanticError::CannotApplyUnary {
                            op: op_str.to_string(),
                            operand: self.type_store.format_type(op_ty),
                            span,
                        }
                        .to_diagnostic(),
                    );
                }
                op_ty
            }
            UnaryOperator::Reference => {
                let inner = self.check_expr(operand, None);
                self.type_store.intern(Type::Reference {
                    mutable: false,
                    inner,
                })
            }
            UnaryOperator::MutableReference => {
                let inner = self.check_expr(operand, None);
                self.type_store.intern(Type::Reference {
                    mutable: true,
                    inner,
                })
            }
        }
    }

    fn check_call(
        &mut self,
        callee: &Expr,
        arguments: &[Argument],
        expected: Option<TypeId>,
        span: Span,
    ) -> TypeId {
        // Special built-in functions
        if let ExprKind::Identifier(ident) = &callee.kind {
            if ident.name == "print" || ident.name == "println" {
                for arg in arguments {
                    self.check_expr(&arg.value, None);
                }
                return TypeId::UNIT;
            }
            if ident.name == "Ok" {
                if let Some(exp) = expected
                    && let Type::Generic { name, args } = self.type_store.get(exp)
                    && name == "Result"
                    && !args.is_empty()
                {
                    let _ = self.check_expr(&arguments[0].value, Some(args[0]));
                    return exp;
                }
                let val_ty = if !arguments.is_empty() {
                    self.check_expr(&arguments[0].value, None)
                } else {
                    TypeId::UNIT
                };
                return self.type_store.intern(Type::Generic {
                    name: "Result".to_string(),
                    args: vec![val_ty, TypeId::ERROR],
                });
            }
            if ident.name == "Err" {
                if let Some(exp) = expected
                    && let Type::Generic { name, args } = self.type_store.get(exp)
                    && name == "Result"
                    && args.len() >= 2
                {
                    let _ = self.check_expr(&arguments[0].value, Some(args[1]));
                    return exp;
                }
                let err_ty = if !arguments.is_empty() {
                    self.check_expr(&arguments[0].value, None)
                } else {
                    TypeId::STRING
                };
                return self.type_store.intern(Type::Generic {
                    name: "Result".to_string(),
                    args: vec![TypeId::ERROR, err_ty],
                });
            }
            if ident.name == "Some" {
                if let Some(exp) = expected
                    && let Type::Optional(inner) = self.type_store.get(exp)
                {
                    let _ = self.check_expr(&arguments[0].value, Some(*inner));
                    return exp;
                }
                let val_ty = if !arguments.is_empty() {
                    self.check_expr(&arguments[0].value, None)
                } else {
                    TypeId::UNIT
                };
                return self.type_store.intern(Type::Optional(val_ty));
            }
        }

        // Callee function type resolution
        let callee_ty = self.check_expr(callee, None);

        match self.type_store.get(callee_ty).clone() {
            Type::Function {
                params,
                return_type,
            } => {
                // If we have precise signature details (default params, names)
                if let ExprKind::Identifier(ident) = &callee.kind
                    && let Some(&fn_sym) = self.resolver.resolutions.get(&ident.span)
                    && let Some((param_details, _)) = self.fn_signatures.get(&fn_sym).cloned()
                {
                    let min_args = param_details.iter().filter(|(_, _, def)| !*def).count();
                    let max_args = param_details.len();

                    if arguments.len() < min_args || arguments.len() > max_args {
                        self.diagnostics.push(
                            SemanticError::ArgumentCountMismatch {
                                expected: min_args,
                                found: arguments.len(),
                                span,
                            }
                            .to_diagnostic(),
                        );
                    }

                    for (i, arg) in arguments.iter().enumerate() {
                        let expected_param_ty = if let Some(arg_name) = &arg.name {
                            param_details
                                .iter()
                                .find(|(name, _, _)| name.as_deref() == Some(&arg_name.name))
                                .map(|(_, ty, _)| *ty)
                                .unwrap_or(TypeId::ERROR)
                        } else if i < param_details.len() {
                            param_details[i].1
                        } else {
                            TypeId::ERROR
                        };

                        let actual_arg_ty = self.check_expr(&arg.value, Some(expected_param_ty));
                        if expected_param_ty != TypeId::ERROR
                            && !self
                                .type_store
                                .is_assignable(expected_param_ty, actual_arg_ty)
                        {
                            self.diagnostics.push(
                                SemanticError::ArgumentTypeMismatch {
                                    expected: self.type_store.format_type(expected_param_ty),
                                    found: self.type_store.format_type(actual_arg_ty),
                                    span: arg.value.span,
                                }
                                .to_diagnostic(),
                            );
                        }
                    }

                    return return_type;
                }

                // Generic function signature checking
                if arguments.len() != params.len() {
                    self.diagnostics.push(
                        SemanticError::ArgumentCountMismatch {
                            expected: params.len(),
                            found: arguments.len(),
                            span,
                        }
                        .to_diagnostic(),
                    );
                }

                for (i, arg) in arguments.iter().enumerate() {
                    let expected_ty = params.get(i).copied().unwrap_or(TypeId::ERROR);
                    let actual_ty = self.check_expr(&arg.value, Some(expected_ty));
                    if expected_ty != TypeId::ERROR
                        && !self.type_store.is_assignable(expected_ty, actual_ty)
                    {
                        self.diagnostics.push(
                            SemanticError::ArgumentTypeMismatch {
                                expected: self.type_store.format_type(expected_ty),
                                found: self.type_store.format_type(actual_ty),
                                span: arg.value.span,
                            }
                            .to_diagnostic(),
                        );
                    }
                }

                return_type
            }
            Type::Error => TypeId::ERROR,
            _ => {
                self.diagnostics.push(
                    SemanticError::NotCallable {
                        found: self.type_store.format_type(callee_ty),
                        span: callee.span,
                    }
                    .to_diagnostic(),
                );
                TypeId::ERROR
            }
        }
    }

    fn check_array(&mut self, elements: &[Expr], expected: Option<TypeId>, _span: Span) -> TypeId {
        let expected_elem = expected.and_then(|exp| match self.type_store.get(exp) {
            Type::List(elem) | Type::Array { element: elem } => Some(*elem),
            _ => None,
        });

        if elements.is_empty() {
            let elem = expected_elem.unwrap_or(TypeId::ERROR);
            return self.type_store.intern(Type::List(elem));
        }

        let first_ty = self.check_expr(&elements[0], expected_elem);

        for elem in &elements[1..] {
            let elem_ty = self.check_expr(elem, Some(first_ty));
            if !self.type_store.is_assignable(first_ty, elem_ty) {
                self.emit_mismatch(first_ty, elem_ty, elem.span);
            }
        }

        self.type_store.intern(Type::List(first_ty))
    }

    fn check_map(
        &mut self,
        entries: &[sumer_ast::expression::MapEntry],
        expected: Option<TypeId>,
    ) -> TypeId {
        let (expected_key, expected_val) = expected
            .and_then(|exp| match self.type_store.get(exp) {
                Type::Map { key, value } => Some((*key, *value)),
                _ => None,
            })
            .unwrap_or((TypeId::STRING, TypeId::INT));

        if entries.is_empty() {
            return self.type_store.intern(Type::Map {
                key: expected_key,
                value: expected_val,
            });
        }

        let k_ty = self.check_expr(&entries[0].key, Some(expected_key));
        let v_ty = self.check_expr(&entries[0].value, Some(expected_val));

        for entry in &entries[1..] {
            let this_k = self.check_expr(&entry.key, Some(k_ty));
            let this_v = self.check_expr(&entry.value, Some(v_ty));

            if !self.type_store.is_assignable(k_ty, this_k) {
                self.emit_mismatch(k_ty, this_k, entry.key.span);
            }
            if !self.type_store.is_assignable(v_ty, this_v) {
                self.emit_mismatch(v_ty, this_v, entry.value.span);
            }
        }

        self.type_store.intern(Type::Map {
            key: k_ty,
            value: v_ty,
        })
    }

    fn check_if_expr(&mut self, if_expr: &IfExpr, expected: Option<TypeId>, span: Span) -> TypeId {
        let cond_ty = self.check_expr(&if_expr.condition, Some(TypeId::BOOL));
        if cond_ty != TypeId::BOOL && cond_ty != TypeId::ERROR {
            self.diagnostics.push(
                SemanticError::ConditionTypeMismatch {
                    found: self.type_store.format_type(cond_ty),
                    span: if_expr.condition.span,
                }
                .to_diagnostic(),
            );
        }

        let then_ty = self.check_block(&if_expr.then_branch, expected);

        if let Some(else_branch) = &if_expr.else_branch {
            let else_ty = match else_branch {
                ElseExprBranch::Block(b) => self.check_block(b, Some(then_ty)),
                ElseExprBranch::ElseIf(nested_if) => {
                    self.check_if_expr(nested_if, Some(then_ty), span)
                }
            };

            if !self.type_store.is_assignable(then_ty, else_ty) {
                self.emit_mismatch(then_ty, else_ty, span);
            }
            then_ty
        } else {
            TypeId::UNIT
        }
    }

    // =========================================================================
    // AST Type Resolution
    // =========================================================================

    pub fn resolve_ast_type(&mut self, ast_type: &AstType) -> TypeId {
        match &ast_type.kind {
            AstTypeKind::Named(path) => self.resolve_type_path(path),
            AstTypeKind::Generic { name, arguments } => {
                let resolved_args: Vec<TypeId> = arguments
                    .iter()
                    .map(|arg| self.resolve_ast_type(arg))
                    .collect();
                if let Some(first) = name.segments.first() {
                    match first.name.as_str() {
                        "Option" if resolved_args.len() == 1 => {
                            self.type_store.intern(Type::Optional(resolved_args[0]))
                        }
                        "List" if resolved_args.len() == 1 => {
                            self.type_store.intern(Type::List(resolved_args[0]))
                        }
                        "Map" if resolved_args.len() == 2 => self.type_store.intern(Type::Map {
                            key: resolved_args[0],
                            value: resolved_args[1],
                        }),
                        _ => self.type_store.intern(Type::Generic {
                            name: first.name.clone(),
                            args: resolved_args,
                        }),
                    }
                } else {
                    TypeId::ERROR
                }
            }
            AstTypeKind::Optional { inner } => {
                let inner_ty = self.resolve_ast_type(inner);
                self.type_store.intern(Type::Optional(inner_ty))
            }
            AstTypeKind::Reference { inner } => {
                let inner_ty = self.resolve_ast_type(inner);
                self.type_store.intern(Type::Reference {
                    mutable: false,
                    inner: inner_ty,
                })
            }
            AstTypeKind::MutableReference { inner } => {
                let inner_ty = self.resolve_ast_type(inner);
                self.type_store.intern(Type::Reference {
                    mutable: true,
                    inner: inner_ty,
                })
            }
            AstTypeKind::Function {
                parameters,
                return_type,
            } => {
                let param_types = parameters
                    .iter()
                    .map(|p| self.resolve_ast_type(p))
                    .collect();
                let ret_ty = self.resolve_ast_type(return_type);
                self.type_store.intern(Type::Function {
                    params: param_types,
                    return_type: ret_ty,
                })
            }
            AstTypeKind::Tuple { elements } => {
                let elem_types = elements.iter().map(|e| self.resolve_ast_type(e)).collect();
                self.type_store.intern(Type::Tuple(elem_types))
            }
            AstTypeKind::Array { element, .. } => {
                let elem_ty = self.resolve_ast_type(element);
                self.type_store.intern(Type::Array { element: elem_ty })
            }
        }
    }

    fn resolve_type_path(&mut self, path: &sumer_ast::types::TypePath) -> TypeId {
        if let Some(first) = path.segments.first() {
            match first.name.as_str() {
                "Unit" => TypeId::UNIT,
                "Bool" => TypeId::BOOL,
                "Int8" => TypeId::INT8,
                "Int16" => TypeId::INT16,
                "Int32" => TypeId::INT32,
                "Int64" => TypeId::INT64,
                "Int128" => TypeId::INT128,
                "UInt8" => TypeId::UINT8,
                "UInt16" => TypeId::UINT16,
                "UInt32" => TypeId::UINT32,
                "UInt64" => TypeId::UINT64,
                "UInt128" => TypeId::UINT128,
                "Float32" => TypeId::FLOAT32,
                "Float64" => TypeId::FLOAT64,
                "Char" => TypeId::CHAR,
                "String" => TypeId::STRING,
                "Byte" => TypeId::BYTE,
                "Int" => TypeId::INT,
                "UInt" => TypeId::UINT,
                "Float" => TypeId::FLOAT,
                _ => {
                    // Check if name was registered in the resolver's scope or resolutions
                    if self.resolver.resolutions.contains_key(&path.span)
                        || self
                            .resolver
                            .scopes
                            .lookup_type(self.resolver.current_scope, &first.name)
                            .is_some()
                    {
                        self.type_store.intern(Type::Named(first.name.clone()))
                    } else {
                        TypeId::ERROR
                    }
                }
            }
        } else {
            TypeId::ERROR
        }
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    fn find_symbol_by_span(&self, span: Span) -> Option<SymbolId> {
        self.resolver.resolutions.get(&span).copied().or_else(|| {
            self.resolver
                .symbols
                .iter()
                .find(|s| s.span == span)
                .map(|s| s.id)
        })
    }

    fn emit_mismatch(&mut self, expected: TypeId, found: TypeId, span: Span) {
        if expected == TypeId::ERROR || found == TypeId::ERROR {
            return;
        }
        self.diagnostics.push(
            SemanticError::TypeMismatch {
                expected: self.type_store.format_type(expected),
                found: self.type_store.format_type(found),
                span,
            }
            .to_diagnostic(),
        );
    }

    fn emit_cannot_apply_binary(&mut self, op: &str, left: TypeId, right: TypeId, span: Span) {
        if left == TypeId::ERROR || right == TypeId::ERROR {
            return;
        }
        self.diagnostics.push(
            SemanticError::CannotApplyBinary {
                op: op.to_string(),
                left: self.type_store.format_type(left),
                right: self.type_store.format_type(right),
                span,
            }
            .to_diagnostic(),
        );
    }
}
