; Locals for scope tracking

; Definitions
(function_definition
  (identifier) @local.definition.function)

(parameter
  (identifier) @local.definition.parameter)

(var_decl
  (identifier) @local.definition.var)

; Scopes
(function_definition) @local.scope
(block) @local.scope

; References
(identifier) @local.reference
