; Keywords
[
  "fn"
  "let"
  "return"
  "if"
  "else"
  "while"
  "for"
] @keyword

; Types
[
  "int"
  "bool"
  "void"
] @type.builtin

; Function definitions - higher priority
(function_definition
  (identifier) @function) @function.definition

; Function calls
(function_call
  (identifier) @function.call)

(function_call_expr
  (identifier) @function.call)

; Parameters
(parameter
  (identifier) @parameter)

; Variables in declarations
(var_decl
  (identifier) @variable)

; Literals
(number) @number
(string) @string
(boolean) @boolean

; Operators
[
  "+"
  "-"
  "*"
  "/"
  "%"
  "="
  "=="
  "!="
  "<"
  "<="
  ">"
  ">="
  "&&"
  "||"
  "&"
  "|"
  "!"
] @operator

; Comments
(comment) @comment

; Punctuation
[
  "("
  ")"
  "{"
  "}"
  ";"
  ":"
  ","
] @punctuation.delimiter

; Catch remaining identifiers as variables (lowest priority)
(identifier) @variable
