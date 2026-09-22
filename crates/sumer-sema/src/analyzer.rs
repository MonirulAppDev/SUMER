//! Public semantic analyzer pipeline driver and semantic result container.

use std::collections::HashMap;

use sumer_ast::program::Program;
use sumer_diagnostics::Diagnostics;
use sumer_span::Span;

use crate::resolver::Resolver;
use crate::scope::ScopeTree;
use crate::symbol::{SymbolId, SymbolTable};
use crate::typeck::{TypeChecker, TypeckResult};

/// The complete output of the semantic analysis phase.
///
/// Contains all emitted diagnostics along with decoupled symbol, scope,
/// resolution, and type checking metadata for downstream compiler phases.
#[derive(Clone, Debug)]
pub struct SemanticResult {
    /// Diagnostics accumulated during semantic validation (errors, warnings).
    pub diagnostics: Diagnostics,
    /// Arena storing all declared and built-in symbols.
    pub symbols: SymbolTable,
    /// Hierarchy of lexical scopes constructed during analysis.
    pub scopes: ScopeTree,
    /// Map from AST node spans to their resolved [`SymbolId`].
    pub resolutions: HashMap<Span, SymbolId>,
    /// Type checking results and interned type store.
    pub typeck: TypeckResult,
}

impl SemanticResult {
    /// Returns `true` if any diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Returns `true` if semantic analysis completed with zero errors.
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    /// Returns the number of diagnostics emitted.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns `true` if no diagnostics were emitted.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// The main entry point for running semantic analysis on a SUMER AST [`Program`].
#[derive(Clone, Debug, Default)]
pub struct SemanticAnalyzer {
    _private: (),
}

impl SemanticAnalyzer {
    /// Creates a new `SemanticAnalyzer`.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Executes declaration collection, scope building, lexical name resolution, and type checking.
    pub fn analyze(self, program: &Program) -> SemanticResult {
        let mut resolver = Resolver::new();
        resolver.resolve_program(program);

        let mut diagnostics = resolver.diagnostics.clone();

        let type_checker = TypeChecker::new(&resolver);
        let typeck = type_checker.check_program(program);

        for diag in typeck.diagnostics.iter() {
            diagnostics.push(diag.clone());
        }

        SemanticResult {
            diagnostics,
            symbols: resolver.symbols,
            scopes: resolver.scopes,
            resolutions: resolver.resolutions,
            typeck,
        }
    }
}

/// Convenience function running semantic analysis on the provided [`Program`].
pub fn analyze(program: &Program) -> SemanticResult {
    SemanticAnalyzer::new().analyze(program)
}
