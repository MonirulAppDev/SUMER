ঠিক আছে। এখন আমরা **implementation-এ যাব না**। আগে language-এর grammar formally lock করি। এই document-টাই পরে Gemini Agent-কে implementation করার জন্য দেওয়া হবে।

# SUMER Grammar Specification v0.1

**Status:** Draft → Implementation Baseline
**File extension:** `.sm`
**Grammar notation:** EBNF-style
**Parser target:** Recursive-descent / Pratt parser compatible

---

## 1. Design Goal

SUMER grammar-এর মূল লক্ষ্য:

> **Readable like modern languages + predictable for compiler + extensible for future domains.**

আমরা syntax এমন রাখব যাতে:

- Beginner-এর জন্য readable হয়
- Parser বানানো সহজ হয়
- Rust/C++ level control possible হয়
- Future syntax breaking change কম হয়
- Web/mobile/backend/embedded সব domain-এ একই core syntax থাকে

---

# 2. Lexical Grammar

প্রথমে দেখি source code কীভাবে tokens-এ ভাঙবে।

### 2.1 Identifier

```ebnf
identifier = identifier-start , { identifier-continue } ;

identifier-start =
      letter
    | "_" ;

identifier-continue =
      letter
    | digit
    | "_" ;
```

Examples:

```sumer
name
userName
_user
user123
calculateTotal
```

Invalid:

```sumer
123user
user-name
```

---

# 3. Keywords

SUMER v0.1-এর reserved keywords:

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
in
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

true
false

Some
None
Ok
Err
```

Future reserved keywords রাখা যেতে পারে, কিন্তু v0.1 parser শুধুমাত্র actual keywords চিনবে।

---

# 4. Literals

## Integer

```ebnf
integer-literal =
    decimal-integer
  | hexadecimal-integer
  | binary-integer
  | octal-integer ;

decimal-integer = digit , { digit | "_" } ;

hexadecimal-integer =
    "0x" , hex-digit , { hex-digit | "_" } ;

binary-integer =
    "0b" , binary-digit , { binary-digit | "_" } ;

octal-integer =
    "0o" , octal-digit , { octal-digit | "_" } ;
```

Examples:

```sumer
100
1_000_000
0xFF
0b1010
0o755
```

---

# 5. Floating Point

```ebnf
float-literal =
    digit , { digit } , "." , digit , { digit }
    [ exponent ] ;

exponent =
    ("e" | "E") , [ "+" | "-" ] , digit , { digit } ;
```

Examples:

```sumer
3.14
10.5
1.25e10
2.5E-4
```

---

# 6. Boolean

```sumer
true
false
```

Grammar:

```ebnf
boolean-literal =
    "true"
  | "false" ;
```

---

# 7. Character

```sumer
'A'
'Z'
'1'
'\n'
'\t'
```

```ebnf
char-literal =
    "'" , char-content , "'" ;
```

---

# 8. String

Single-line:

```sumer
"Hello"
"Hello SUMER"
```

Interpolation:

```sumer
let name = "Monir"

print("Hello {name}")
```

Multiline:

```sumer
let text = """
Hello
SUMER
Language
"""
```

Grammar:

```ebnf
string-literal =
    '"' , { string-character } , '"'
  | '"""' , { multiline-character } , '"""' ;
```

String interpolation will be handled by the lexer/parser as part of string processing rather than treating `{name}` as ordinary text.

---

# 9. Program

A `.sm` file is a program.

```ebnf
program =
    { attribute }
    { declaration } ;
```

Example:

```sumer
import math

const PI = 3.14

fn main() {
    print("Hello")
}
```

---

# 10. Declaration

```ebnf
declaration =
      function-declaration
    | struct-declaration
    | enum-declaration
    | trait-declaration
    | impl-declaration
    | variable-declaration
    | constant-declaration
    | import-declaration ;
```

Visibility:

```sumer
pub fn hello() {
}
```

`pub` applies to declarations where allowed.

---

# 11. Attributes

Example:

```sumer
@derive(Debug)
@derive(Json)
struct User {
    id: Int
}
```

Grammar:

```ebnf
attribute =
    "@" , identifier
    [ "(" , argument-list , ")" ] ;
```

Examples:

```sumer
@derive(Debug)
@derive(Json)
@test
@inline
@target("windows")
```

Attributes are intentionally generic.

Compiler/library features can later define their meaning.

---

# 12. Function Declaration

Basic:

```sumer
fn add(a: Int, b: Int) -> Int {
    return a + b
}
```

Grammar:

```ebnf
function-declaration =
    [ "pub" ]
    [ "async" ]
    "fn"
    identifier
    generic-parameters?
    "(" , parameter-list? , ")"
    return-type?
    block ;
```

Return type:

```ebnf
return-type =
    "->" , type ;
```

---

# 13. Parameters

```ebnf
parameter-list =
    parameter , { "," , parameter } ;

parameter =
    identifier , ":" , type
  | identifier , ":" , type , "=" , expression ;
```

Examples:

```sumer
fn greet(name: String)
```

Default:

```sumer
fn greet(name: String = "World")
```

---

# 14. Named Arguments

Call:

```sumer
createUser(
    name: "Monir",
    age: 30
)
```

Grammar:

```ebnf
argument-list =
    argument , { "," , argument } ;

argument =
    [ identifier , ":" ]
    expression ;
```

---

# 15. Struct

```sumer
struct User {
    id: Int
    name: String
    age: Int
}
```

Grammar:

```ebnf
struct-declaration =
    [ "pub" ]
    "struct"
    identifier
    generic-parameters?
    "{"
    { struct-member }
    "}" ;

struct-member =
      field-declaration
    | function-declaration ;

field-declaration =
    [ "pub" ]
    identifier , ":" , type ;
```

---

# 16. Struct Initialization

```sumer
let user = User {
    id: 1,
    name: "Monir",
    age: 30
}
```

Grammar:

```ebnf
struct-expression =
    identifier
    "{"
    [ field-initializer-list ]
    "}" ;

field-initializer-list =
    field-initializer , { "," , field-initializer } ;

field-initializer =
    identifier , ":" , expression ;
```

Trailing comma should be allowed:

```sumer
User {
    id: 1,
    name: "Monir",
}
```

---

# 17. Enum

Simple:

```sumer
enum Status {
    Active
    Inactive
    Suspended
}
```

Grammar:

```ebnf
enum-declaration =
    [ "pub" ]
    "enum"
    identifier
    generic-parameters?
    "{"
    { enum-variant }
    "}" ;

enum-variant =
    identifier
    [ variant-data ] ;
```

---

# 18. Enum Data

Tuple-style:

```sumer
enum Result {
    Success(String)
    Error(Int, String)
}
```

Named:

```sumer
enum Payment {
    Cash
    Card(number: String)
    Mobile(provider: String)
}
```

Grammar:

```ebnf
variant-data =
      "(" , [ type-list ] , ")"
    | "(" , [ parameter-list ] , ")" ;
```

Implementation-এর সময় ambiguity avoid করার জন্য আমরা **named fields** এবং **positional fields** আলাদা AST node হিসেবে রাখব।

---

# 19. Trait

```sumer
trait Printable {
    fn print()
}
```

Grammar:

```ebnf
trait-declaration =
    [ "pub" ]
    "trait"
    identifier
    generic-parameters?
    [ trait-bounds ]
    "{"
    { trait-member }
    "}" ;

trait-member =
    function-signature
  | function-declaration ;
```

Function signature:

```sumer
fn print()
```

অথবা:

```sumer
fn print() -> String
```

---

# 20. Trait Bounds

Example:

```sumer
trait Printable: Debug + Clone {
}
```

Grammar:

```ebnf
trait-bounds =
    ":" , type-bound , { "+" , type-bound } ;

type-bound =
    identifier ;
```

Generic example:

```sumer
fn printValue<T: Printable>(value: T) {
}
```

---

# 21. Impl

```sumer
impl Printable for User {
    fn print() {
        print(self.name)
    }
}
```

Grammar:

```ebnf
impl-declaration =
    "impl"
    generic-parameters?
    [ trait-type , "for" ]
    type
    "{"
    { function-declaration }
    "}" ;
```

This supports both:

```sumer
impl User {
    fn greet() {
    }
}
```

and:

```sumer
impl Printable for User {
    fn print() {
    }
}
```

---

# 22. Variables

Immutable:

```sumer
let name = "Monir"
```

Mutable:

```sumer
var age = 30
```

Explicit type:

```sumer
let age: Int = 30
```

Grammar:

```ebnf
variable-declaration =
    [ "pub" ]
    ( "let" | "var" )
    identifier
    [ ":" , type ]
    "="
    expression ;
```

---

# 23. Constants

```sumer
const MAX_USERS = 1000
```

Grammar:

```ebnf
constant-declaration =
    [ "pub" ]
    "const"
    identifier
    [ ":" , type ]
    "="
    expression ;
```

---

# 24. Import

Simple:

```sumer
import math
```

Nested:

```sumer
import web.http
```

Specific:

```sumer
import user.User
```

Grammar:

```ebnf
import-declaration =
    "import"
    module-path ;

module-path =
    identifier , { "." , identifier } ;
```

Future:

```sumer
import math.{sin, cos}
```

এটা v0.1-এ parser-ready করা জরুরি নয়।

---

# 25. Statements

```ebnf
statement =
      variable-declaration
    | constant-declaration
    | expression-statement
    | return-statement
    | if-statement
    | while-statement
    | for-statement
    | loop-statement
    | match-statement
    | break-statement
    | continue-statement
    | block ;
```

---

# 26. Block

```ebnf
block =
    "{"
    { statement }
    "}" ;
```

Example:

```sumer
fn main() {
    let x = 10
    let y = 20

    print(x + y)
}
```

---

# 27. Return

```sumer
return
```

অথবা:

```sumer
return value
```

Grammar:

```ebnf
return-statement =
    "return" , [ expression ] ;
```

---

# 28. If

```sumer
if age >= 18 {
    print("Adult")
}
```

Else:

```sumer
if age >= 18 {
    print("Adult")
} else {
    print("Minor")
}
```

Grammar:

```ebnf
if-statement =
    "if"
    expression
    block
    [ "else" , ( if-statement | block ) ] ;
```

---

# 29. If Expression

SUMER supports:

```sumer
let status = if age >= 18 {
    "adult"
} else {
    "minor"
}
```

Grammar:

```ebnf
if-expression =
    "if"
    expression
    block
    "else"
    block ;
```

Important rule:

> `if` used as an expression must have an `else`.

---

# 30. While

```sumer
while count < 10 {
    count += 1
}
```

Grammar:

```ebnf
while-statement =
    "while"
    expression
    block ;
```

---

# 31. For

```sumer
for item in items {
    print(item)
}
```

Grammar:

```ebnf
for-statement =
    "for"
    pattern
    "in"
    expression
    block ;
```

Range:

```sumer
for i in 0..10 {
    print(i)
}
```

Inclusive:

```sumer
for i in 0..=10 {
    print(i)
}
```

---

# 32. Loop

Infinite loop:

```sumer
loop {
    work()
}
```

Grammar:

```ebnf
loop-statement =
    "loop"
    block ;
```

---

# 33. Break

```sumer
break
```

Future label support:

```sumer
break outer
```

কিন্তু v0.1:

```ebnf
break-statement =
    "break" ;
```

---

# 34. Continue

```sumer
continue
```

Grammar:

```ebnf
continue-statement =
    "continue" ;
```

---

# 35. Match

Example:

```sumer
match status {
    Status.Active => print("Active")
    Status.Inactive => print("Inactive")
}
```

Grammar:

```ebnf
match-statement =
    "match"
    expression
    "{"
    { match-arm }
    "}" ;

match-arm =
    pattern
    "=>"
    expression
    [ "," ] ;
```

Block:

```sumer
match status {
    Status.Active => {
        print("Active")
    }

    Status.Inactive => {
        print("Inactive")
    }
}
```

---

# 36. Match Expression

```sumer
let message = match status {
    Status.Active => "Running"
    Status.Inactive => "Stopped"
}
```

Same `match` AST can represent both statement and expression depending on context.

---

# 37. Pattern

Patterns are used in:

- `match`
- `for`
- future destructuring
- future `let`

Base grammar:

```ebnf
pattern =
      identifier-pattern
    | literal-pattern
    | wildcard-pattern
    | enum-pattern
    | struct-pattern
    | tuple-pattern ;
```

---

# 38. Identifier Pattern

```sumer
user
```

```ebnf
identifier-pattern =
    identifier ;
```

---

# 39. Wildcard

```sumer
_
```

Grammar:

```ebnf
wildcard-pattern =
    "_" ;
```

Example:

```sumer
match value {
    0 => print("zero")
    _ => print("other")
}
```

---

# 40. Literal Pattern

```sumer
1
"hello"
true
```

```ebnf
literal-pattern =
      integer-literal
    | float-literal
    | string-literal
    | char-literal
    | boolean-literal ;
```

---

# 41. Enum Pattern

```sumer
Payment.Cash
```

```sumer
Payment.Card(number)
```

```sumer
Payment.Mobile(provider)
```

Grammar:

```ebnf
enum-pattern =
    path
    [ "(" , pattern-list , ")" ] ;
```

---

# 42. Struct Pattern

Future-friendly syntax:

```sumer
User {
    name,
    age
}
```

Grammar:

```ebnf
struct-pattern =
    identifier
    "{"
    [ field-pattern-list ]
    "}" ;
```

---

# 43. Expression Grammar

এখানে সবচেয়ে important part.

SUMER expression grammar Pratt parser-friendly হবে।

Conceptually:

```ebnf
expression =
    assignment-expression ;
```

---

# 44. Assignment

```sumer
x = 10
x += 5
x -= 2
x *= 3
x /= 2
x %= 2
```

```ebnf
assignment-expression =
    logical-or-expression
    [ assignment-operator , assignment-expression ] ;

assignment-operator =
      "="
    | "+="
    | "-="
    | "*="
    | "/="
    | "%=" ;
```

Assignment right-associative।

---

# 45. Logical OR

```sumer
a || b
```

```ebnf
logical-or-expression =
    logical-and-expression
    { "||" , logical-and-expression } ;
```

---

# 46. Logical AND

```sumer
a && b
```

```ebnf
logical-and-expression =
    equality-expression
    { "&&" , equality-expression } ;
```

---

# 47. Equality

```sumer
a == b
a != b
```

```ebnf
equality-expression =
    comparison-expression
    { ( "==" | "!=" ) , comparison-expression } ;
```

---

# 48. Comparison

```sumer
a > b
a < b
a >= b
a <= b
```

```ebnf
comparison-expression =
    range-expression
    {
        ( ">" | "<" | ">=" | "<=" )
        , range-expression
    } ;
```

---

# 49. Range

```sumer
0..10
0..=10
```

```ebnf
range-expression =
    additive-expression
    [
        ( ".." | "..=" )
        , additive-expression
    ] ;
```

---

# 50. Addition / Subtraction

```sumer
a + b
a - b
```

```ebnf
additive-expression =
    multiplicative-expression
    {
        ( "+" | "-" )
        , multiplicative-expression
    } ;
```

---

# 51. Multiplication

```sumer
a * b
a / b
a % b
```

```ebnf
multiplicative-expression =
    unary-expression
    {
        ( "*" | "/" | "%" )
        , unary-expression
    } ;
```

---

# 52. Unary

```sumer
-x
!enabled
```

References:

```sumer
&value
&mut value
```

Grammar:

```ebnf
unary-expression =
      ( "!" | "-" | "+" ) , unary-expression
    | "&" , [ "mut" ] , unary-expression
    | postfix-expression ;
```

---

# 53. Postfix

Postfix operations:

```sumer
user.name
user?.name
user.method()
items[0]
function()
value!
```

Grammar:

```ebnf
postfix-expression =
    primary-expression
    {
          member-access
        | optional-member-access
        | function-call
        | index-expression
        | force-unwrap
    } ;
```

---

# 54. Member Access

```sumer
user.name
```

```ebnf
member-access =
    "." , identifier ;
```

---

# 55. Optional Member Access

```sumer
user?.name
```

```ebnf
optional-member-access =
    "?." , identifier ;
```

---

# 56. Function Call

```sumer
add(10, 20)
```

```ebnf
function-call =
    "("
    [ argument-list ]
    ")" ;
```

---

# 57. Index

```sumer
items[0]
map["name"]
```

```ebnf
index-expression =
    "[" , expression , "]" ;
```

---

# 58. Force Unwrap

```sumer
user!
```

Grammar:

```ebnf
force-unwrap =
    "!" ;
```

This is specifically for `Option`/nullable values.

---

# 59. Primary Expressions

```ebnf
primary-expression =
      literal
    | identifier
    | struct-expression
    | array-expression
    | map-expression
    | lambda-expression
    | if-expression
    | match-expression
    | "(" , expression , ")" ;
```

---

# 60. Literal

```ebnf
literal =
      integer-literal
    | float-literal
    | string-literal
    | char-literal
    | boolean-literal
    | "None" ;
```

`Some(...)`, `Ok(...)`, `Err(...)` normal function-like constructors হিসেবে parse করা যাবে।

---

# 61. Array / List

```sumer
let numbers = [1, 2, 3, 4]
```

Grammar:

```ebnf
array-expression =
    "["
    [ expression-list ]
    "]" ;
```

---

# 62. Map

Proposed syntax:

```sumer
let users = {
    "one": 1,
    "two": 2
}
```

Grammar:

```ebnf
map-expression =
    "{"
    [ map-entry-list ]
    "}" ;

map-entry-list =
    map-entry , { "," , map-entry } ;

map-entry =
    expression , ":" , expression ;
```

**Important:** `{}` ambiguity with block will be resolved based on parser context.

---

# 63. Lambda

Basic:

```sumer
let add = (a, b) => a + b
```

Grammar:

```ebnf
lambda-expression =
    "("
    [ parameter-list ]
    ")"
    "=>"
    expression ;
```

Possible single parameter:

```sumer
let square = x => x * x
```

v0.1 parser-এর জন্য আমরা initially parenthesized form support করতে পারি:

```sumer
(a) => a * a
```

এতে parser simpler থাকবে।

---

# 64. Generic Parameters

Example:

```sumer
fn max<T: Comparable>(a: T, b: T) -> T {
    ...
}
```

Grammar:

```ebnf
generic-parameters =
    "<"
    generic-parameter
    { "," , generic-parameter }
    ">" ;

generic-parameter =
    identifier
    [ ":" , type-bounds ] ;
```

---

# 65. Generic Type

```sumer
List<Int>
Map<String, User>
Box<String>
```

Grammar:

```ebnf
generic-type =
    identifier
    "<"
    type
    { "," , type }
    ">" ;
```

---

# 66. Type Grammar

Core:

```ebnf
type =
      primitive-type
    | named-type
    | generic-type
    | function-type
    | reference-type
    | optional-type
    | tuple-type
    | array-type
    | "(" , type , ")" ;
```

---

# 67. Primitive Types

```text
Bool

Int8
Int16
Int32
Int64
Int128

UInt8
UInt16
UInt32
UInt64
UInt128

Float32
Float64

Char
String
Byte

Int
UInt
Float
```

Grammar:

```ebnf
primitive-type =
      "Bool"
    | "Int8"
    | "Int16"
    | "Int32"
    | "Int64"
    | "Int128"
    | "UInt8"
    | "UInt16"
    | "UInt32"
    | "UInt64"
    | "UInt128"
    | "Float32"
    | "Float64"
    | "Char"
    | "String"
    | "Byte"
    | "Int"
    | "UInt"
    | "Float" ;
```

---

# 68. Named Type

```sumer
User
Database
HttpClient
```

```ebnf
named-type =
    path ;
```

---

# 69. Optional Type

```sumer
User?
String?
Int?
```

Grammar:

```ebnf
optional-type =
    type , "?" ;
```

Internally:

```text
User?
```

becomes conceptually:

```text
Option<User>
```

---

# 70. Reference Type

Immutable:

```sumer
&T
```

Mutable:

```sumer
&mut T
```

Grammar:

```ebnf
reference-type =
    "&"
    [ "mut" ]
    type ;
```

---

# 71. Function Type

Future-ready:

```sumer
(Int, Int) -> Int
```

Grammar:

```ebnf
function-type =
    "("
    [ type-list ]
    ")"
    "->"
    type ;
```

---

# 72. Tuple Type

```sumer
(Int, String, Bool)
```

Grammar:

```ebnf
tuple-type =
    "("
    type
    ","
    type
    { "," , type }
    ")" ;
```

---

# 73. Array Type

Fixed-size array:

```sumer
[Int; 10]
```

Grammar:

```ebnf
array-type =
    "["
    type
    ";"
    expression
    "]" ;
```

Dynamic collections like `List<T>` remain library types.

---

# 74. Operator Precedence

Parser implementation-এর জন্য precedence table:

| Priority | Operator                     | Associativity |     |      |
| -------- | ---------------------------- | ------------- | --- | ---- |
| 1        | `=` `+=` `-=` `*=` `/=` `%=` | Right         |     |      |
| 2        | `                            |               | `   | Left |
| 3        | `&&`                         | Left          |     |      |
| 4        | `==` `!=`                    | Left          |     |      |
| 5        | `>` `<` `>=` `<=`            | Left          |     |      |
| 6        | `..` `..=`                   | Left          |     |      |
| 7        | `+` `-`                      | Left          |     |      |
| 8        | `*` `/` `%`                  | Left          |     |      |
| 9        | unary `!` `-` `+` `&`        | Right         |     |      |
| 10       | `.` `?.` `()` `[]` `!`       | Left          |     |      |

Example:

```sumer
a + b * c
```

means:

```text
a + (b * c)
```

not:

```text
(a + b) * c
```

---

# 75. Method Call

```sumer
user.greet()
```

Parser এটাকে:

```text
MemberAccess(
    object = user,
    member = greet
)
```

তারপর:

```text
Call(...)
```

হিসেবে represent করতে পারবে।

---

# 76. Chained Calls

Allowed:

```sumer
users
    .filter(...)
    .map(...)
    .sort(...)
```

অথবা single line:

```sumer
users.filter(...).map(...).sort(...)
```

এটা language-এর fluent programming support করবে।

---

# 77. Async Grammar

Function:

```sumer
async fn download(url: String) -> Result<Data, Error> {
    ...
}
```

Grammar already:

```ebnf
function-declaration =
    [ "pub" ]
    [ "async" ]
    "fn"
    ...
```

Await:

```sumer
let data = await download(url)
```

Grammar:

```ebnf
await-expression =
    "await" , expression ;
```

---

# 78. Spawn

```sumer
let task = spawn download(url)
```

Grammar:

```ebnf
spawn-expression =
    "spawn" , expression ;
```

---

# 79. Unsafe Block

```sumer
unsafe {
    rawPointer.write(value)
}
```

Grammar:

```ebnf
unsafe-block =
    "unsafe"
    block ;
```

এটা expression বা statement হিসেবে compiler context অনুযায়ী represent করা যাবে।

---

# 80. Time Literals

SUMER-এর domain support-এর জন্য:

```sumer
500ms
2s
5min
1h
```

এগুলো v0.1 core parser-এ primitive literal না করে **lexer-supported special literal** হিসেবে রাখা যেতে পারে।

Conceptually:

```text
500ms
```

→

```text
DurationLiteral(500, Millisecond)
```

Supported:

```text
ns
us
ms
s
min
h
```

---

# 81. Complete Example

এখন সব grammar মিলিয়ে একটি valid SUMER program:

```sumer
import math

@derive(Debug)
struct User {
    id: Int
    name: String
    age: Int

    fn greet() {
        print("Hello {self.name}")
    }
}

enum Status {
    Active
    Inactive
}

fn isAdult(age: Int) -> Bool {
    age >= 18
}

fn main() {
    let user = User {
        id: 1,
        name: "Monir",
        age: 30,
    }

    user.greet()

    if isAdult(user.age) {
        print("Adult")
    } else {
        print("Minor")
    }

    let numbers = [1, 2, 3, 4, 5]

    for number in numbers {
        print(number)
    }

    match Status.Active {
        Status.Active => print("Active")
        Status.Inactive => print("Inactive")
    }
}
```

---

# 82. Parser Architecture

এখন grammar অনুযায়ী parser architecture হবে:

```text
Source Code
    ↓
Lexer
    ↓
Token Stream
    ↓
Parser
    ├── Declaration Parser
    ├── Statement Parser
    ├── Expression Parser
    │      └── Pratt Parser
    ├── Type Parser
    ├── Pattern Parser
    └── Attribute Parser
    ↓
AST
```

বিশেষ করে **Expression Parser-এর জন্য Pratt Parser** ব্যবহার করাই ভালো। এতে operator precedence future-এ extend করা সহজ হবে।

---

# 83. AST Design Principle

একটা important architectural rule:

> **AST যেন source syntax-এর exact copy না হয়; compiler-এর জন্য meaningful structure represent করে।**

Example:

```sumer
a + b * c
```

AST:

```text
Binary(+)
├── Identifier(a)
└── Binary(*)
    ├── Identifier(b)
    └── Identifier(c)
```

এতে semantic analyzer সহজে বুঝবে:

```text
a + (b * c)
```

---

# 84. Error Recovery

Parser প্রথম error দেখেই পুরো compilation stop করবে না।

Example:

```sumer
fn main( {
    let x =
    print(x)
}
```

Parser ideally multiple diagnostics collect করবে:

```text
error[E0001]: expected parameter or ')'
error[E0001]: expected expression after '='
```

Goal:

> **একবার compile করলে যত reasonable syntax error সম্ভব একসাথে দেখানো।**

---

# 85. Grammar Rules for v0.1

এখন কিছু বিষয় deliberately **v0.1-এ বাদ** রাখছি:

### বাদ:

```text
Macros
Operator overloading
Custom operators
Pattern guards
Lifetimes syntax
Dependent types
Compile-time metaprogramming
Reflection syntax
Actor syntax
GPU kernel syntax
Language-level UI syntax
SQL syntax
Embedded-specific syntax
```

এগুলো future extension।

কারণ SUMER-এর core grammar unnecessarily বড় করা যাবে না।

---

# 86. Core Language Boundary

সবচেয়ে গুরুত্বপূর্ণ architecture rule:

```text
SUMER CORE
──────────────
Variables
Functions
Types
Struct
Enum
Trait
Generics
Pattern Matching
Ownership
References
Modules
Async syntax
Error handling
```

এর বাইরে:

```text
HTTP       → library
PostgreSQL → library
Redis      → library
Flutter-like UI → framework
Android    → platform SDK
iOS        → platform SDK
AI/ML      → framework
GPU        → framework
Game       → framework
ROS        → package
Cloud      → package
```

অর্থাৎ:

> **Language core small থাকবে, ecosystem huge হবে।**

---

# 87. Final Grammar Pipeline

আমাদের parser implementation শেষে flow হবে:

```text
hello.sm
   │
   ▼
Lexer
   │
   ▼
Tokens
   │
   ▼
Parser
   │
   ├── declarations
   ├── statements
   ├── expressions
   ├── types
   ├── patterns
   └── attributes
   │
   ▼
AST
   │
   ▼
Pretty Printer
```

প্রথম compiler milestone-এর output হবে:

```bash
sumer parse hello.sm
```

যেটা AST print করবে।

উদাহরণ:

```text
Program
├── Function main
│   └── Block
│       └── Call
│           ├── Identifier: print
│           └── String: "Hello, SUMER!"
```

**এখনও LLVM, native `.exe`, ownership checker বা AI framework implement করা হবে না।**

পরবর্তী ধাপে আমরা এই grammar-কে ভিত্তি করে **`SUMER AST Specification v0.1`** বানাব। সেখানে প্রতিটি grammar construct-এর exact Rust AST structure, node hierarchy, `Span`, `Token`, `Expr`, `Stmt`, `Type`, `Pattern` design lock করব। তারপরই Gemini-কে প্রথম ছোট implementation task দেওয়া যাবে।
