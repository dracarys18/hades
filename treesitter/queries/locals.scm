; Definitions
(function_definition
  name: (identifier) @local.definition.function)

(parameter
  name: (identifier) @local.definition.parameter)

(var_decl
  name: (identifier) @local.definition.var)

(struct_definition
  name: (identifier) @local.definition.type)

; Scopes
(function_definition) @local.scope
(struct_definition)   @local.scope
(block)               @local.scope

; References
(identifier) @local.reference
