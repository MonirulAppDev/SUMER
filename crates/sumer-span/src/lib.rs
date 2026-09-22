//! Source location and file tracking for the SUMER compiler.
//!
//! This crate provides foundational abstractions for tracking source files,
//! byte-level spans, and human-readable line/column mapping.
//!
//! # Core Abstractions
//!
//! - [`SourceId`]: Unique lightweight identifier for a registered source file.
//! - [`Span`]: Half-open byte range `[start, end)` referencing a specific [`SourceId`].
//! - [`SourceFile`]: Manages source content, metadata, and precomputes line indices.
//! - [`SourceMap`]: Registry assigning sequential [`SourceId`]s and providing location queries.
//! - [`LineColumn`]: 1-based human-readable line and column diagnostic coordinates.

pub mod source;
pub mod span;

pub use source::{LineColumn, SourceFile, SourceMap};
pub use span::{SourceId, Span};
