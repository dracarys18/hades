; Catch-all for identifiers (must come first so specific rules below override it)
(identifier) @variable

; Keywords
[
  "fn"
  "let"
  "return"
  "if"
  "else"
  "while"
  "for"
  "import"
  "module"
  "struct"
  "as"
] @keyword

; self as keyword (in parameter position)
((identifier) @keyword
 (#eq? @keyword "self"))

; Import prefix (std / self)
(import_statement
  prefix: (identifier) @keyword.import)

; Import module name
(import_statement
  module: (identifier) @module)

; Module declaration name
(module_declaration
  name: (identifier) @module)

; Builtin types
[
  "int"
  "float"
  "bool"
  "void"
  "string"
  "char"
  "Self"
] @type.builtin

; Cast target type in as_expression
(as_expression
  target_type: _ @type.builtin)

; Array type — size literal inside [N]
(array_type (number) @number)

; Struct definition name
(struct_definition
  name: (identifier) @type)

; Struct field names
(struct_field
  name: (identifier) @variable.member)

; Function definitions
(function_definition
  name: (identifier) @function)

; Function calls (statement form)
(function_call
  name: (identifier) @function.call)

; Function calls (expression form)
(function_call_expr
  name: (identifier) @function.call)

; Method calls (statement form)
(method_call
  object: (identifier) @variable
  method: (identifier) @function.method)

; Method calls (expression form)
(method_call_expr
  object: (identifier) @variable
  method: (identifier) @function.method)

; Qualified calls: module::function(...)
(qualified_call
  module: (identifier) @module
  name:   (identifier) @function.call)

(qualified_call_expr
  module: (identifier) @module
  name:   (identifier) @function.call)

; "::" path separator
"::" @punctuation.special

; Field access: object.field
(field_access
  object: (identifier) @variable
  field:  (identifier) @variable.member)

; Parameters
(parameter
  name: (identifier) @variable.parameter)

; Variable declarations — name only
(var_decl
  name: (identifier) @variable)

; Struct init — struct name is a type
(structInit
  name: (identifier) @type)

; Struct field init names
(fieldInit
  name: (identifier) @variable.member)

; Compound assignment target
(compound_assignment
  target: (identifier) @variable)

; Assignment target
(assignment_statement
  target: (identifier) @variable)

; Compound assignment operators
(compound_assignment
  operator: _ @operator)

; Literals
(number)  @number
(string)  @string
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
  "["
  "]"
  ";"
  ":"
  ","
  "."
] @punctuation.delimiter
