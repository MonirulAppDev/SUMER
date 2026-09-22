ঠিক আছে। এখন আমরা **SUMER Compiler Specification v0.1**-এ যাব। এখানে মূল লক্ষ্য হলো Gemini Agent-কে code লেখার আগে compiler-এর architecture একদম পরিষ্কার করে দেওয়া।

# SUMER Compiler Specification v0.1

**Compiler implementation language:** Rust
**SUMER source extension:** `.sm`
**Compiler executable:** `sumer`
**Compiler internal name:** `sumerc`

---

## 1. Compiler-এর মূল Architecture

SUMER compiler হবে একটি multi-stage compiler:

```text
                 main.sm
                    │
                    ▼
              ┌───────────┐
              │   Lexer   │
              └─────┬─────┘
                    │
                 Tokens
                    │
                    ▼
              ┌───────────┐
              │  Parser   │
              └─────┬─────┘
                    │
                   AST
                    │
                    ▼
          ┌──────────────────┐
          │ Semantic Analyzer│
          └────────┬─────────┘
                   │
             Typed AST / HIR
                   │
                   ▼
          ┌──────────────────┐
          │  Type Checker    │
          └────────┬─────────┘
                   │
                   ▼
                  HIR
                   │
                   ▼
             ┌───────────┐
             │    MIR    │
             └─────┬─────┘
                   │
                   ▼
             ┌───────────┐
             │ Optimizer │
             └─────┬─────┘
                   │
                   ▼
              ┌─────────┐
              │ Backend │
              └────┬────┘
                   │
        ┌──────────┼───────────┐
        ▼          ▼           ▼
      LLVM        WASM      Future Backends
        │
        ▼
  Native Machine Code
```

---

# 2. Compiler Design Philosophy

আমাদের compiler-এর design হবে:

### Modular

প্রতিটি stage আলাদা module।

### Testable

প্রতিটি stage independently test করা যাবে।

### Incremental

পরবর্তীতে incremental compilation যোগ করা যাবে।

### Extensible

নতুন backend যোগ করা সহজ হবে।

### IDE-friendly

Compiler diagnostics এমনভাবে তৈরি হবে যাতে LSP ব্যবহার করতে পারে।

### Fast

Unnecessary parsing/type-checking avoid করার foundation থাকবে।

---

# 3. Repository Structure

Initial repository:

```text
sumer/
│
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
│
├── crates/
│   │
│   ├── sumer-lexer/
│   ├── sumer-parser/
│   ├── sumer-ast/
│   ├── sumer-span/
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
│
├── examples/
│
├── docs/
│
└── std/
```

---

# 4. কেন Multiple Crates?

আমরা শুরুতেই সব code এক জায়গায় রাখব না।

খারাপ:

```text
src/
├── lexer.rs
├── parser.rs
├── compiler.rs
├── types.rs
├── ast.rs
├── main.rs
```

শুরুতে এটা সহজ মনে হলেও project বড় হলে dependency mess তৈরি হবে।

বরং:

```text
sumer-lexer
     ↓
sumer-parser
     ↓
sumer-ast
     ↓
sumer-sema
     ↓
sumer-hir
     ↓
sumer-mir
     ↓
sumer-codegen
```

প্রতিটি component-এর responsibility পরিষ্কার থাকবে।

---

# 5. `sumer-span`

সব compiler diagnostic-এর foundation হবে source location system।

প্রতিটি token/node-এর সাথে:

```text
FileId
Start
End
```

থাকবে।

Example:

```text
main.sm:10:15
```

Compiler error:

```text
main.sm:10:15

error: unknown variable `username`

10 | print(username)
               ^^^^^^^^
```

---

# 6. Token System

Lexer source code থেকে tokens তৈরি করবে।

Example:

```sumer
let age: Int = 30
```

Tokens:

```text
LET
IDENT("age")
COLON
IDENT("Int")
EQUAL
INT_LITERAL(30)
```

---

# 7. Token Categories

### Keywords

v0.1:

```text
let
var
const

fn
return

struct
enum
trait
impl

if
else

for
while
loop

match

break
continue

import
pub

async
await
spawn

unsafe
```

---

# 8. Literals

Lexer support করবে:

```text
Integer
Float
String
Character
Boolean
```

Examples:

```sumer
123
123.45
"Hello"
'A'
true
false
```

---

# 9. Operators

Lexer:

```text
+
-
*
/
%
=

==
!=
>
<
>=
<=

&&
||
!

+=
-=
*=
/=
%=

?
&

..
..=
=>
```

---

# 10. Delimiters

```text
(
)
{
}
[
]

:
,
.
;
```

**Important:** SUMER v0.1 semicolon optional করবে।

Valid:

```sumer
let x = 10
let y = 20
```

No need:

```sumer
let x = 10;
```

তবে parser future compatibility-এর জন্য optional `;` accept করতে পারে।

---

# 11. Lexer Error

Invalid character:

```text
@
```

যদি context অনুযায়ী invalid হয়:

```text
error: unexpected character '@'
```

Unknown token কখনো silently ignore করা যাবে না।

---

# 12. Parser

Parser token stream থেকে AST তৈরি করবে।

Input:

```sumer
let x = 10 + 20
```

AST:

```text
LetDecl
├── name: x
└── value
    └── BinaryExpr(+)
        ├── Integer(10)
        └── Integer(20)
```

---

# 13. AST Design

AST হবে semantic analysis-এর জন্য যথেষ্ট expressive।

Root:

```text
Program
```

এর মধ্যে:

```text
Declaration[]
```

---

# 14. Declarations

AST declaration types:

```text
Function
Struct
Enum
Trait
Impl
Variable
Constant
Import
```

---

# 15. Statements

```text
ExpressionStmt
LetStmt
ReturnStmt
IfStmt
WhileStmt
ForStmt
LoopStmt
BreakStmt
ContinueStmt
BlockStmt
```

---

# 16. Expressions

```text
Literal
Identifier
Binary
Unary
Call
MemberAccess
Index
Array
Map
Lambda
IfExpression
MatchExpression
StructInit
```

---

# 17. Function AST

Source:

```sumer
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

AST:

```text
Function
├── name: add
├── parameters
│   ├── a: Int
│   └── b: Int
├── return_type: Int
└── body
    └── Block
        └── Binary(+)
```

---

# 18. Type AST

Initially:

```text
TypeExpr
```

Variants:

```text
Named
Generic
Function
Reference
MutableReference
Optional
```

Examples:

```sumer
Int
List<Int>
(Int, Int) -> Int
&User
&mut User
User?
```

---

# 19. Semantic Analysis

Parser শুধু syntax check করবে।

Semantic analyzer করবে:

```text
Name resolution
Scope resolution
Duplicate declaration detection
Visibility
Type checking
Trait checking
Generic validation
Return validation
Borrowing checks
```

---

# 20. Scope System

Example:

```sumer
fn main() {
    let x = 10

    if true {
        let y = 20
        print(x)
        print(y)
    }

    print(x)
}
```

Valid:

```text
inner scope → outer variable access
```

Invalid:

```text
outer scope → inner variable access
```

Example:

```sumer
if true {
    let y = 20
}

print(y)
```

Error:

```text
error: cannot find value `y` in this scope
```

---

# 21. Symbol Table

Compiler internally maintain করবে:

```text
SymbolTable
```

প্রতিটি scope:

```text
Scope
├── symbols
├── parent
└── children
```

Symbol:

```text
name
kind
type
visibility
source_span
```

---

# 22. Type System

Compiler-এর type system হবে static।

Example:

```sumer
let age: Int = 30
```

valid।

কিন্তু:

```sumer
let age: Int = "30"
```

invalid।

Diagnostic:

```text
error: expected `Int`, found `String`
```

---

# 23. Type Inference

Valid:

```sumer
let age = 30
```

Compiler infer:

```text
age: Int
```

আর:

```sumer
let price = 10.5
```

infer:

```text
Float64
```

---

# 24. Type Representation

Internal compiler type:

```text
Type
├── Bool
├── Int
├── UInt
├── Float
├── Char
├── String
├── Unit
├── Never
├── Array
├── List
├── Map
├── Set
├── Function
├── Struct
├── Enum
├── Generic
├── Reference
├── MutableReference
├── Option
├── Result
└── Unknown
```

---

# 25. Unit Type

Function returning nothing:

```sumer
fn printHello() {
    print("Hello")
}
```

Internal return type:

```text
Unit
```

---

# 26. Never Type

For code that never returns:

```sumer
fn panic(message: String) -> Never {
    ...
}
```

Future use:

```text
panic
exit
infinite loop
```

---

# 27. Type Compatibility

Exact type matching first priority।

Implicit dangerous conversion নয়।

Example:

```sumer
let x: Int = 10
let y: Float64 = x
```

Error:

```text
cannot implicitly convert Int to Float64
```

Developer must write:

```sumer
let y = x.toFloat()
```

---

# 28. HIR

AST developer-friendly syntax preserve করবে।

HIR compiler-friendly representation হবে।

Example:

```sumer
let x = 10 + 20
```

HIR:

```text
Let
x
=
Add
  Const 10
  Const 20
```

এখানে syntactic details কমে যাবে।

---

# 29. MIR

MIR হবে lower-level intermediate representation।

Example:

```text
v1 = const 10
v2 = const 20
v3 = add v1, v2
store x, v3
```

এখানে control-flow explicit হবে।

---

# 30. Basic Blocks

MIR:

```text
BasicBlock 0
    condition
    branch -> Block1 / Block2

BasicBlock 1
    ...

BasicBlock 2
    ...
```

এটা optimization-এর জন্য গুরুত্বপূর্ণ।

---

# 31. Control Flow Graph

Compiler maintain করবে:

```text
CFG
```

যাতে:

```text
if
loop
while
match
return
break
continue
```

optimize করা যায়।

---

# 32. Ownership

SUMER v0.1 compiler-এর foundation-এ ownership concept থাকবে।

প্রতিটি value-এর ownership state:

```text
Owned
Moved
Borrowed
MutablyBorrowed
Dropped
```

---

# 33. Move

Example:

```sumer
let a = User(...)
let b = a
```

যদি `User` move-only হয়:

```text
a → moved
b → owner
```

তারপর:

```sumer
print(a)
```

error:

```text
use of moved value `a`
```

---

# 34. Copy Types

Primitive types:

```text
Int
Float
Bool
Char
```

এগুলো Copy হতে পারে।

তাই:

```sumer
let a = 10
let b = a

print(a)
```

valid।

---

# 35. Borrowing

Immutable:

```sumer
fn printUser(user: &User) {
}
```

Mutable:

```sumer
fn updateUser(user: &mut User) {
}
```

Compiler rules:

```text
many immutable borrows
OR
one mutable borrow
```

একই সময়ে দুটো নয়।

---

# 36. Lifetime

v0.1 implementation-এ lifetime syntax expose না করেও compiler lifetime inference করতে পারবে যেখানে সম্ভব।

Future explicit syntax reserved:

```text
'a
'b
```

---

# 37. Error System

Compiler error architecture:

```text
Diagnostic
├── severity
├── code
├── message
├── primary_span
├── secondary_spans
├── notes
└── suggestions
```

Example:

```text
error[E0308]: type mismatch

  --> main.sm:4:13
   |
 4 | let age: Int = "30"
   |                 ^^^^
   |
   = expected Int
   = found String
```

---

# 38. Warning System

```text
warning[W0001]: unused variable `x`
```

Compiler কখনো warning-কে error হিসেবে treat করবে না unless configured:

```bash
sumer build --deny-warnings
```

---

# 39. Codegen Architecture

`sumerc` backend interface:

```text
trait CodegenBackend {
    fn compile(...)
    fn emit(...)
}
```

Initial backend:

```text
LLVM
```

Future:

```text
WASM
Custom Embedded
GPU
```

---

# 40. LLVM Strategy

আমরা LLVM directly language frontend হিসেবে ব্যবহার করব না।

Architecture:

```text
SUMER
 ↓
AST
 ↓
HIR
 ↓
MIR
 ↓
SUMER Optimizer
 ↓
LLVM IR
 ↓
LLVM
 ↓
Native
```

এতে SUMER-এর নিজস্ব optimization infrastructure থাকবে।

---

# 41. LLVM Backend

Initial target:

```text
x86_64-pc-windows-msvc
```

কারণ development environment Windows।

তারপর:

```text
x86_64-linux
aarch64-linux
aarch64-macos
```

---

# 42. Compiler Driver

User-facing executable:

```bash
sumer
```

Internal pipeline:

```text
sumer
 ↓
sumer-driver
 ↓
lexer
 ↓
parser
 ↓
sema
 ↓
hir
 ↓
mir
 ↓
codegen
```

---

# 43. CLI Commands v0.1

Initially implement:

```bash
sumer new
sumer check
sumer run
sumer build
```

Later:

```bash
sumer test
sumer fmt
sumer lint
sumer add
sumer remove
sumer update
sumer publish
sumer doc
sumer repl
```

---

# 44. `sumer check`

Only compile-check করবে:

```bash
sumer check
```

Pipeline:

```text
Source
→ Lexer
→ Parser
→ Semantic
→ Type checking
```

Native binary generate করবে না।

---

# 45. `sumer build`

Full pipeline:

```text
Source
→ Lexer
→ Parser
→ Semantic
→ HIR
→ MIR
→ Optimize
→ LLVM
→ Object
→ Linker
→ Executable
```

---

# 46. `sumer run`

Development convenience:

```text
sumer run
```

Internally:

```text
build
↓
execute
```

---

# 47. Incremental Compilation

v0.1 architecture incremental-ready হবে।

Compiler cache:

```text
target/
└── debug/
    └── .sumer/
```

Future:

```text
Source changed
    ↓
Dependency graph
    ↓
Only affected modules compile
```

v0.1-এ full recompilation acceptable।

---

# 48. Diagnostics Must Be Excellent

এটা আমরা শুরু থেকেই গুরুত্ব দেব।

Bad:

```text
type error
```

Good:

```text
error[E0308]: type mismatch

expected:
    Int

found:
    String

help:
    convert the value using `.toInt()`
```

---

# 49. Compiler Error Codes

Category:

```text
E0001 → Syntax
E0002 → Name resolution
E0003 → Type
E0004 → Borrow
E0005 → Move
E0006 → Trait
E0007 → Generic
E0008 → Import
E0009 → Internal
```

এই system future-proof হবে।

---

# 50. Testing Strategy

প্রতিটি compiler stage-এর tests থাকবে।

```text
tests/
├── lexer/
├── parser/
├── types/
├── sema/
├── hir/
├── mir/
├── codegen/
└── integration/
```

---

# 51. Lexer Test

Input:

```sumer
let x = 10
```

Expected:

```text
LET
IDENT
EQUAL
INT
```

---

# 52. Parser Test

Input:

```sumer
let x = 10 + 20
```

Expected AST:

```text
Let
 └── Binary(+)
```

---

# 53. Type Test

Valid:

```sumer
let x: Int = 10
```

Invalid:

```sumer
let x: Int = "hello"
```

---

# 54. Integration Test

Source:

```sumer
fn main() {
    let x = 10
    print(x)
}
```

Expected:

```text
Hello executable
Exit code: 0
```

---

# 55. Golden Tests

Compiler output-এর snapshot testing থাকবে।

Example:

```text
input:
main.sm

expected:
diagnostic snapshot
```

এতে compiler refactor করলেও behavior detect করা যাবে।

---

# 56. Compiler Development Language

Rust project:

```toml
[package]
name = "sumer"
version = "0.1.0"
edition = "2024"
```

আমরা Rust stable ecosystem use করব।

---

# 57. Dependencies Philosophy

শুরুতে dependency minimum রাখা হবে।

Potential:

```text
logos       → lexer
thiserror   → compiler errors
ariadne     → diagnostics
clap        → CLI
serde       → configuration
toml        → project config
```

LLVM integration পরে।

---

# 58. Why Rust?

SUMER compiler-এর জন্য Rust:

```text
Memory safety
High performance
Strong type system
Excellent tooling
LLVM ecosystem
Cross-platform
```

এবং compiler development-এর জন্য খুব ভালো fit।

---

# 59. Bootstrap Strategy

প্রথম SUMER compiler:

```text
SUMER compiler → Rust
```

Future:

```text
SUMER compiler
     ↓
SUMER compiler written in SUMER
```

Long-term লক্ষ্য:

> **Self-hosting SUMER compiler.**

---

# 60. Self Hosting

Future architecture:

```text
Stage 0
Rust compiler
     ↓
SUMER compiler

Stage 1
SUMER compiler compiles SUMER

Stage 2
SUMER compiler
     ↓
SUMER compiler
```

এটাকে হবে:

```text
SUMER Self-Hosting
```

---

# 61. Bootstrap Compiler

Initial compiler-এর নাম:

```text
sumerc-bootstrap
```

Language mature হলে:

```text
sumerc
```

নিজেই নিজের source compile করবে।

---

# 62. Version Architecture

```text
SUMER Language
     │
     ├── v0.1
     ├── v0.2
     ├── v0.3
     │
     └── v1.0
```

Compiler:

```text
sumerc 0.1
```

Language version:

```text
SUMER 0.1
```

দুটো independent version হলেও compatibility matrix maintain করতে হবে।

---

# 63. First Implementation Milestone

আমাদের প্রথম লক্ষ্য **full compiler নয়**।

প্রথম milestone:

```text
SUMER source
    ↓
Lexer
    ↓
Parser
    ↓
AST
    ↓
Pretty print AST
```

যেমন:

```bash
sumer parse hello.sm
```

Output:

```text
Program
└── Function main
    └── Call print
        └── String "Hello, SUMER!"
```

এটা কাজ করলে compiler foundation proven হবে।

---

# 64. Milestone 2

```text
Lexer
+
Parser
+
AST
+
Semantic Analyzer
+
Type Checker
```

তারপর:

```bash
sumer check
```

---

# 65. Milestone 3

Interpreter:

```text
sumer run
```

এখানে LLVM ছাড়াই SUMER program execute করতে পারব।

এটি debugging এবং language design-এর জন্য extremely useful।

---

# 66. Milestone 4

Native compiler:

```text
SUMER
 ↓
MIR
 ↓
LLVM
 ↓
.exe
```

তারপর:

```bash
sumer build
```

একটা সত্যিকারের native executable তৈরি হবে।

---

# 67. Development Rule

প্রতিটি milestone-এর আগে:

```text
Specification
↓
Tests
↓
Implementation
↓
Manual verification
↓
Next milestone
```

Gemini Agent-কে কখনো:

> "Build the entire SUMER compiler."

দেওয়া হবে না।

বরং ছোট, verifiable tasks:

> "Implement only the lexer according to SUMER Lexer Specification v0.1. Do not implement parser or semantic analysis."

এতে agent-generated code অনেক বেশি controllable হবে।

---

# 68. প্রথম Gemini Agent Task

এখন **আমরা এখনও implementation শুরু করছি না**—কিন্তু next actual coding task হবে:

### `SUMER Compiler Bootstrap — Project Initialization`

Gemini-কে প্রথম prompt-এ এই কাজগুলো করানো হবে:

```text
Create the SUMER compiler bootstrap repository.

Use Rust.

Create a Cargo workspace.

Create these crates:

sumer-span
sumer-lexer
sumer-ast
sumer-parser
sumer-diagnostics
sumer-types
sumer-sema
sumer-hir
sumer-mir
sumer-codegen
sumer-driver
sumer-cli

Do NOT implement compiler logic yet.

Only create:
- workspace
- crate dependencies
- basic lib/main files
- module boundaries
- README
- .gitignore
- basic test setup

The project must compile successfully with:

cargo check
cargo test
```

কিন্তু **এই prompt এখনই Gemini-কে দেওয়ার আগে**, একটা জিনিস আমাদের আরও freeze করা দরকার: **SUMER v0.1-এর exact grammar / EBNF**।

কারণ compiler-এর parser implementation সরাসরি grammar-এর উপর নির্ভর করবে।

পরের ধাপে আমরা বানাব:

# `SUMER Grammar Specification v0.1`

যেখানে থাকবে:

```text
Program
Declaration
Function
Struct
Enum
Trait
Impl
Statement
Expression
Type
Pattern
Generic
Lambda
Operator precedence
Block
Import
Attribute
```

এবং প্রতিটির **formal EBNF grammar**। এটা শেষ হলেই Gemini-কে প্রথম implementation prompt দিয়ে actual compiler coding শুরু করা নিরাপদ হবে।
