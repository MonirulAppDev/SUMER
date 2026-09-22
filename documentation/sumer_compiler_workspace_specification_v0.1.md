ঠিক আছে। এখন আমরা **implementation-ready workspace architecture** lock করব। এই ধাপ শেষ হলে প্রথমবার Gemini Agent-কে actual coding task দেওয়া যাবে।

# SUMER Compiler Workspace Specification v0.1

**Compiler implementation:** Rust
**Rust edition:** 2024
**Language source:** `.sm`
**Compiler CLI:** `sumer`
**Internal compiler:** `sumerc` conceptually
**Initial target:** Windows x86_64

---

# 1. Repository Structure

Final baseline:

```text
sumer/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
│
├── crates/
│   ├── sumer-span/
│   ├── sumer-lexer/
│   ├── sumer-ast/
│   ├── sumer-parser/
│   ├── sumer-diagnostics/
│   ├── sumer-sema/
│   ├── sumer-types/
│   ├── sumer-hir/
│   ├── sumer-mir/
│   ├── sumer-codegen/
│   ├── sumer-driver/
│   └── sumer-cli/
│
├── tests/
│   ├── lexer/
│   ├── parser/
│   ├── sema/
│   ├── typecheck/
│   └── integration/
│
├── examples/
│   ├── hello.sm
│   ├── functions.sm
│   └── structs.sm
│
├── std/
│
├── docs/
│   ├── language/
│   ├── compiler/
│   └── design/
│
└── scripts/
```

---

# 2. Why Multiple Crates?

আমরা সব compiler code এক project-এর `src/`-এ রাখব না।

Bad:

```text
src/
├── lexer.rs
├── parser.rs
├── ast.rs
├── typechecker.rs
├── llvm.rs
├── main.rs
└── ...
```

Project বড় হলে dependency mess হবে।

Instead:

```text
Lexer
  ↓
Parser
  ↓
AST
  ↓
Sema
  ↓
HIR
  ↓
MIR
  ↓
Codegen
```

প্রতিটি layer আলাদা crate।

---

# 3. Dependency Graph

এই dependency direction খুব carefully maintain করতে হবে:

```text id="f9f5s6"
sumer-span
    ↑
    │
sumer-lexer
    │
    ├───────────────┐
    ↓               ↓
sumer-ast      sumer-diagnostics
    ↑
    │
sumer-parser
    │
    ↓
sumer-sema
    │
    ├── sumer-types
    ↓
sumer-hir
    ↓
sumer-mir
    ↓
sumer-codegen
    ↓
sumer-driver
    ↓
sumer-cli
```

আর CLI সবকিছুর উপরে থাকবে।

---

# 4. Critical Architecture Rule

**Circular dependency strictly forbidden.**

Example:

```text
sumer-lexer → sumer-parser
```

allowed হতে পারে।

কিন্তু:

```text
sumer-parser → sumer-lexer
sumer-lexer → sumer-parser
```

এটা হবে না।

আর:

```text
sumer-ast → sumer-sema
```

এটাও হবে না।

AST কখনো semantic analyzer-এর ওপর depend করবে না।

---

# 5. `sumer-span`

Purpose:

> Source location tracking.

Structure:

```text id="h0cvmt"
sumer-span/
└── src/
    ├── lib.rs
    ├── span.rs
    └── source.rs
```

Core types:

```rust id="8k5x0g"
pub struct SourceId(pub u32);

pub struct Span {
    pub source: SourceId,
    pub start: u32,
    pub end: u32,
}
```

Future:

```text
SourceMap
FileId
LineMap
```

---

# 6. `sumer-lexer`

Purpose:

> `.sm` source → tokens

Structure:

```text id="b0z8a1"
sumer-lexer/
└── src/
    ├── lib.rs
    ├── token.rs
    ├── lexer.rs
    └── keyword.rs
```

Responsibilities:

```text
source
 ↓
characters
 ↓
tokens
```

Lexer **AST জানবে না**।

---

# 7. `sumer-ast`

Purpose:

> Parser-এর output representation.

Structure:

```text id="sgq1p8"
sumer-ast/
└── src/
    ├── lib.rs
    ├── program.rs
    ├── declaration.rs
    ├── statement.rs
    ├── expression.rs
    ├── types.rs
    ├── pattern.rs
    ├── attribute.rs
    ├── generics.rs
    └── identifier.rs
```

এখানে আমরা আগের AST specification-এর types রাখব।

---

# 8. `sumer-parser`

Purpose:

```text
Token Stream → AST
```

Structure:

```text id="0fjxli"
sumer-parser/
└── src/
    ├── lib.rs
    ├── parser.rs
    ├── declarations.rs
    ├── statements.rs
    ├── expressions.rs
    ├── types.rs
    ├── patterns.rs
    ├── attributes.rs
    └── generics.rs
```

Expression parser:

> **Pratt Parser**

এটা এখানে implement হবে।

---

# 9. `sumer-diagnostics`

Purpose:

Compiler error system।

Structure:

```text id="y8m3vw"
sumer-diagnostics/
└── src/
    ├── lib.rs
    ├── diagnostic.rs
    ├── severity.rs
    ├── error_code.rs
    └── renderer.rs
```

Core:

```rust id="9v4h6d"
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}
```

Diagnostic:

```rust id="g6w2h4"
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub primary_span: Option<Span>,
    pub secondary_spans: Vec<Span>,
    pub notes: Vec<String>,
    pub suggestions: Vec<String>,
}
```

---

# 10. Error Code System

Baseline:

```text id="z4zz7s"
E0001 Syntax Error
E0002 Name Resolution Error
E0003 Type Error
E0004 Borrow Error
E0005 Move Error
E0006 Trait Error
E0007 Generic Error
E0008 Import Error
E0009 Internal Compiler Error
```

Later:

```text
W0001 Unused Variable
W0002 Unused Import
W0003 Unreachable Code
```

---

# 11. `sumer-types`

এখানে semantic type system থাকবে।

Important distinction:

```text
sumer-ast::Type
```

এবং:

```text
sumer-types::Ty
```

এক জিনিস নয়।

AST:

```sumer
let x: User
```

Semantic type:

```text
Ty::Struct(UserId)
```

---

# 12. `sumer-sema`

Purpose:

```text
AST
 ↓
Name Resolution
 ↓
Scope Analysis
 ↓
Type Checking
 ↓
Trait Checking
 ↓
Borrow Checking
```

Structure:

```text id="e2z9v8"
sumer-sema/
└── src/
    ├── lib.rs
    ├── analyzer.rs
    ├── resolver.rs
    ├── scopes.rs
    ├── symbols.rs
    ├── type_checker.rs
    ├── borrow_checker.rs
    └── traits.rs
```

---

# 13. Semantic Pipeline

একবারে সব checker implement করা হবে না।

Phase:

```text id="k7b5ph"
Phase 1
AST
 ↓
Name Resolution

Phase 2
Name Resolution
 ↓
Basic Type Checking

Phase 3
Generic Checking

Phase 4
Trait Checking

Phase 5
Ownership / Move Checking

Phase 6
Borrow Checking
```

এভাবে incremental হবে।

---

# 14. `sumer-hir`

HIR = High-level Intermediate Representation.

AST-এর তুলনায় simpler এবং compiler-friendly।

```text id="4ql1om"
AST
 ↓
Semantic Analysis
 ↓
HIR
```

AST-এর syntax-specific complexity এখানে কমে যাবে।

উদাহরণ:

```sumer id="9of0rf"
let x = 10 + 20
```

HIR:

```text id="j2v2lq"
Let
 ├── LocalId(0)
 └── BinaryAdd
      ├── Const(10)
      └── Const(20)
```

---

# 15. `sumer-mir`

MIR = Mid-level Intermediate Representation.

এখানে control-flow explicit হবে।

Example:

```sumer id="h6xq9p"
if x > 10 {
    print("big")
} else {
    print("small")
}
```

MIR conceptually:

```text id="5vqzq6"
BasicBlock0
    compare x > 10
    branch BasicBlock1 / BasicBlock2

BasicBlock1
    print("big")
    goto BasicBlock3

BasicBlock2
    print("small")
    goto BasicBlock3

BasicBlock3
    return
```

এখান থেকেই optimization সহজ হবে।

---

# 16. `sumer-codegen`

Initial backend:

> LLVM

Structure:

```text id="9n4i8j"
sumer-codegen/
└── src/
    ├── lib.rs
    ├── llvm/
    │   ├── mod.rs
    │   ├── context.rs
    │   ├── types.rs
    │   ├── functions.rs
    │   ├── expressions.rs
    │   └── statements.rs
    └── target.rs
```

Important:

**LLVM frontend হবে না।**

SUMER:

```text
SUMER AST
   ↓
SUMER HIR
   ↓
SUMER MIR
   ↓
LLVM IR
```

তারপর LLVM native machine code generate করবে।

---

# 17. `sumer-driver`

Compiler orchestration।

এখানে pipeline connect হবে:

```text id="u3a7ed"
Source
 ↓
Lexer
 ↓
Parser
 ↓
Sema
 ↓
HIR
 ↓
MIR
 ↓
Codegen
```

Structure:

```text id="x4f9l8"
sumer-driver/
└── src/
    ├── lib.rs
    ├── pipeline.rs
    ├── session.rs
    ├── config.rs
    └── compilation.rs
```

---

# 18. `sumer-cli`

User-facing executable:

```bash id="2f0k7b"
sumer
```

Structure:

```text id="uxsp3q"
sumer-cli/
└── src/
    ├── main.rs
    ├── commands/
    │   ├── mod.rs
    │   ├── new.rs
    │   ├── check.rs
    │   ├── parse.rs
    │   ├── build.rs
    │   ├── run.rs
    │   └── fmt.rs
    └── cli.rs
```

---

# 19. Initial CLI

v0.1:

```bash
sumer parse hello.sm
```

v0.2:

```bash
sumer check hello.sm
```

Later:

```bash
sumer build
sumer run
```

Eventually:

```bash
sumer new
sumer init
sumer test
sumer fmt
sumer clean
sumer add
sumer remove
sumer update
sumer publish
sumer doc
sumer repl
```

---

# 20. Root Cargo Workspace

Root `Cargo.toml` conceptually:

```toml id="yq5v9h"
[workspace]
resolver = "2"

members = [
    "crates/sumer-span",
    "crates/sumer-lexer",
    "crates/sumer-ast",
    "crates/sumer-parser",
    "crates/sumer-diagnostics",
    "crates/sumer-types",
    "crates/sumer-sema",
    "crates/sumer-hir",
    "crates/sumer-mir",
    "crates/sumer-codegen",
    "crates/sumer-driver",
    "crates/sumer-cli",
]

[workspace.package]
edition = "2024"
version = "0.1.0"
license = "MIT"
```

প্রথম implementation-এ unnecessary dependencies add করা যাবে না।

---

# 21. Dependency Policy

Gemini-কে একটা important rule দিতে হবে:

> **Dependency add করার আগে determine করতে হবে dependency genuinely necessary কিনা।**

Potential dependencies:

```text
logos       → lexer
thiserror   → Rust errors
ariadne     → diagnostics
clap        → CLI
serde       → config
toml        → sumer.toml
```

LLVM আসবে অনেক পরে।

প্রথম milestone-এ LLVM dependency **একদম লাগবে না**।

---

# 22. Initial Dependency Graph

First milestone:

```text id="4px3ne"
sumer-cli
   ↓
sumer-driver
   ↓
sumer-parser
   ↓
sumer-ast
   ↓
sumer-span

sumer-parser
   ↓
sumer-lexer
   ↓
sumer-span

sumer-diagnostics
   ↓
sumer-span
```

---

# 23. Testing Architecture

Compiler project-এ testing mandatory।

```text id="j0zjmw"
tests/
├── lexer/
├── parser/
├── ast/
├── sema/
├── typecheck/
├── diagnostics/
└── integration/
```

Rust unit tests থাকবে crate-এর ভিতরেও।

Example:

```rust id="3s1q3d"
#[test]
fn lex_integer() {
    // ...
}
```

Integration:

```text id="ezn1jk"
tests/integration/hello.rs
```

---

# 24. Golden Tests

Parser-এর জন্য golden/snapshot-style tests রাখব।

Input:

```sumer id="qkr5p6"
fn main() {
    print("Hello")
}
```

Expected AST:

```text id="j8j9u6"
Program
└── Function(main)
    └── Block
        └── Call(print)
            └── String("Hello")
```

Source পরিবর্তন করলে expected AST intentionalভাবে update করতে হবে।

---

# 25. Compiler Development Rule

সবচেয়ে important workflow:

```text id="f8w6g4"
Specification
      ↓
Test
      ↓
Implementation
      ↓
Run Tests
      ↓
Manual Verification
      ↓
Commit
      ↓
Next Feature
```

**Specification ছাড়া random coding নয়।**

---

# 26. Git Commit Strategy

প্রতিটি ছোট milestone আলাদা commit:

```text id="9ksyq5"
chore: initialize workspace
feat: add source spans
feat: add token model
feat: implement lexer
feat: add ast model
feat: implement parser
feat: add parser diagnostics
test: add lexer test suite
test: add parser golden tests
```

এতে Gemini ভুল implementation করলে rollback করা সহজ হবে।

---

# 27. প্রথম Milestone

এখন আমাদের actual implementation target:

## Milestone 1 — Lexer + Parser Foundation

Scope:

```text id="xg0z5r"
Workspace
  ↓
Span
  ↓
Token
  ↓
Lexer
  ↓
AST
  ↓
Parser
  ↓
AST Printer
```

Support:

```text
Identifiers
Keywords
Integers
Floats
Strings
Characters
Booleans

Operators
Delimiters

Functions
Variables
Expressions
Blocks
Basic statements
```

---

# 28. Milestone 1-এ যা থাকবে না

এগুলো **এখনই implement করা নিষেধ**:

```text id="gblt9m"
❌ LLVM
❌ Native executable
❌ Ownership checker
❌ Borrow checker
❌ Type checker
❌ Generics semantic validation
❌ Trait resolution
❌ Async runtime
❌ Package manager
❌ Standard library
❌ HTTP
❌ Database
❌ UI
❌ AI
```

এতে Gemini scope creep করবে না।

---

# 29. First Working Goal

শেষে আমরা এই file চালাতে চাই:

```text id="t5s2k6"
hello.sm
```

```sumer id="z3t9tq"
fn main() {
    print("Hello, SUMER!")
}
```

Command:

```bash id="ryh7j9"
sumer parse hello.sm
```

Output:

```text id="0m0k8u"
Program
└── Function
    ├── name: main
    ├── parameters: []
    ├── return_type: None
    └── body
        └── Call
            ├── callee: print
            └── argument:
                String("Hello, SUMER!")
```

**এই output পাওয়া গেলেই প্রথম milestone successful।**

---

# 30. প্রথম Gemini Agent Prompt

এখন আমরা actual implementation শুরু করতে পারি।

Gemini Agent-কে **এই prompt-টাই প্রথমে** দিতে হবে:

```text
You are implementing the SUMER programming language compiler.

Read and follow the SUMER Language Specification, SUMER Grammar Specification,
SUMER AST Specification, and SUMER Compiler Workspace Specification provided
in the project documentation.

IMPORTANT DEVELOPMENT RULES:

1. Do NOT implement the entire compiler.
2. Do NOT implement LLVM.
3. Do NOT implement semantic analysis.
4. Do NOT implement type checking.
5. Do NOT implement ownership or borrow checking.
6. Do NOT implement async runtime.
7. Do NOT add unnecessary dependencies.
8. Do NOT redesign the language syntax.
9. Do NOT change the documented architecture without explaining why.
10. Work only on the currently requested milestone.

CURRENT MILESTONE:

Milestone 1 — Compiler Workspace Foundation.

TASK 1 ONLY:

Create the Rust Cargo workspace for SUMER.

Required structure:

sumer/
├── Cargo.toml
├── crates/
│   ├── sumer-span/
│   ├── sumer-lexer/
│   ├── sumer-ast/
│   ├── sumer-parser/
│   ├── sumer-diagnostics/
│   ├── sumer-types/
│   ├── sumer-sema/
│   ├── sumer-hir/
│   ├── sumer-mir/
│   ├── sumer-codegen/
│   ├── sumer-driver/
│   └── sumer-cli/
├── tests/
├── examples/
├── docs/
└── scripts/

For this task:

- Initialize the Cargo workspace.
- Create all workspace crates.
- Use Rust edition 2024.
- Keep dependencies minimal.
- Make sure the workspace compiles with `cargo check`.
- Create basic `lib.rs` files for library crates.
- Create a basic `main.rs` for sumer-cli.
- Do NOT implement lexer/parser/AST yet.
- Do NOT add LLVM yet.
- Do NOT add semantic analysis yet.
- Do NOT add any unnecessary framework.

After implementation:

1. Run `cargo check --workspace`.
2. Fix all compilation errors.
3. Show the final directory structure.
4. Summarize exactly what was created.
5. Do not proceed to the next task automatically.

STOP after Task 1.
```

### Gemini-এর জন্য Rule

Gemini যদি বলে:

> “I also implemented lexer/parser because it would be useful…”

**Allow করবে না।**

বলবে:

```text
Stop. Do not implement the next milestone.
Only complete the currently requested task.
```

আমরা একেকটা brick বসিয়ে compiler বানাব। এতে SUMER-এর architecture শুরু থেকেই controlled থাকবে।

**পরের ধাপ হবে `TASK 2 — Source Span + Source File System`**, যেখানে `sumer-span` crate বাস্তবে implement করব।
