/**
 * @file Treesitter parser for Hades Programming Language
 * @author Karthikey Hegde <me@karthihegde.dev>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "hades",
  word: $ => $.identifier,
  extras: $ => [
    /\s/,
    $.comment
  ],
  conflicts: $ => [
    [$._call_arg, $.expression],
    [$.as_expression, $.expression],
    [$._expr_value, $.expression],
    [$.unary_expression, $.deref_expr],
  ],
  rules: {
    source_file: $ => repeat($._definition),

    _definition: $ => choice(
      $.function_definition,
      $.struct_definition,
      $.var_decl,
      $.function_call,
      $.method_call,
      $.qualified_call,
      $.import_statement,
      $.module_declaration,
    ),

    import_statement: $ => seq(
      'import',
      field('prefix', $.identifier),
      '::',
      field('module', $.identifier),
    ),

    module_declaration: $ => seq(
      'module',
      field('name', $.identifier),
    ),

    function_definition: $ => seq(
      'fn',
      field('name', $.identifier),
      $.parameter_list,
      ':',
      $._type,
      $.block
    ),

    struct_definition: $ => seq(
      'struct',
      field('name', $.identifier),
      '{',
      repeat($.struct_member),
      '}'
    ),

    struct_member: $ => choice(
      $.struct_field,
      $.function_definition
    ),

    struct_field: $ => seq(
      field('name', $.identifier),
      ':',
      field('type', $._type),
      optional(',')
    ),

    var_decl: $ => seq(
      'let',
      field('name', $.identifier),
      optional(seq(':', field('type', $._type))),
      '=',
      $._expr_value,
      ';'
    ),

    _expr_value: $ => choice(
      $.as_expression,
      $.expression,
      $.array_index,
      $.field_access,
      $.method_call_expr,
      $.qualified_call_expr,
      $.function_call_expr,
      $.identifier,
      $.value_literal,
      $.structInit,
      $.deref_expr
    ),

    as_expression: $ => seq(
      field('expr', choice(
        $.expression,
        $.array_index,
        $.field_access,
        $.method_call_expr,
        $.qualified_call_expr,
        $.function_call_expr,
        $.identifier,
        $.value_literal,
        $.deref_expr
      )),
      'as',
      field('target_type', $._base_type)
    ),

    parameter_list: $ => seq(
      '(',
      optional(seq(
        $.parameter,
        repeat(seq(',', $.parameter))
      )),
      ')'
    ),

    parameter: $ => seq(
      field('name', $.identifier),
      ':',
      field('type', $._type)
    ),

    value_literal: $ => choice(
      $.number,
      $.string,
      $.boolean,
      $.array_literal,
      'null'
    ),

    array_literal: $ => seq(
      '[',
      optional(seq(
        choice($.identifier, $.number, $.string, $.boolean),
        repeat(seq(',', choice($.identifier, $.number, $.string, $.boolean)))
      )),
      ']'
    ),

    array_index: $ => seq(
      field('array', $.identifier),
      '[',
      field('index', choice($.identifier, $.number, $.expression)),
      ']'
    ),

    field_access: $ => seq(
      field('object', choice($.identifier, $.array_index, $.field_access, $.deref_expr)),
      '.',
      field('field', $.identifier)
    ),

    method_call_expr: $ => seq(
      field('object', choice($.identifier, $.field_access)),
      '.',
      field('method', $.identifier),
      $.call_parameter_list
    ),

    method_call: $ => seq(
      field('object', choice($.identifier, $.field_access)),
      '.',
      field('method', $.identifier),
      $.call_parameter_list,
      ';'
    ),

    array_type: $ => seq('[', $.number, ']', $._base_type),

    pointer_type: $ => seq('&', $._type),

    _base_type: $ => choice(
      'bool',
      'int',
      'float',
      'void',
      'string',
      'char',
      'Self',
      $.identifier
    ),

    _type: $ => choice(
      $._base_type,
      $.array_type,
      $.pointer_type
    ),

    block: $ => seq(
      '{',
      repeat($._statement),
      '}'
    ),

    _call_arg: $ => choice(
      $.as_expression,
      $.expression,
      $.array_index,
      $.field_access,
      $.method_call_expr,
      $.qualified_call_expr,
      $.function_call_expr,
      $.identifier,
      $.value_literal,
      $.deref_expr
    ),

    call_parameter_list: $ => seq(
      '(',
      optional(seq(
        $._call_arg,
        repeat(seq(',', $._call_arg))
      )),
      ')'
    ),

    function_call: $ => seq(
      field('name', $.identifier),
      $.call_parameter_list,
      ';'
    ),

    function_call_expr: $ => seq(
      field('name', $.identifier),
      $.call_parameter_list
    ),

    qualified_call: $ => seq(
      field('module', $.identifier),
      '::',
      field('name', $.identifier),
      $.call_parameter_list,
      ';'
    ),

    qualified_call_expr: $ => seq(
      field('module', $.identifier),
      '::',
      field('name', $.identifier),
      $.call_parameter_list
    ),

    continue_statement: $ => seq('continue', ';'),

    break_statement: $ => seq('break', ';'),

    _statement: $ => choice(
      $.return_statement,
      $.continue_statement,
      $.break_statement,
      $.var_decl,
      $.function_call,
      $.method_call,
      $.qualified_call,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.assignment_statement,
      $.compound_assignment,
      $.import_statement,
      $.module_declaration,
      $.function_definition,
      $.struct_definition,
    ),

    return_statement: $ => seq(
      'return',
      $._expr_value,
      ';'
    ),

    if_statement: $ => seq(
      'if',
      '(',
      $._expr_value,
      ')',
      $.block,
      optional(seq('else', choice($.block, $.if_statement)))
    ),

    while_statement: $ => seq(
      'while',
      '(',
      $._expr_value,
      ')',
      $.block
    ),

    for_statement: $ => seq(
      'for',
      optional($.var_decl),
      $._expr_value,
      ';',
      optional(choice($.assignment_statement, $.compound_assignment)),
      $.block
    ),

    assignment_statement: $ => seq(
      field('target', choice($.field_access, $.array_index, $.identifier, $.deref_expr)),
      '=',
      $._expr_value,
      ';'
    ),

    compound_assignment: $ => seq(
      field('target', choice($.field_access, $.array_index, $.identifier, $.deref_expr)),
      field('operator', choice('+=', '-=', '*=', '/=')),
      choice(
        $.expression,
        $.array_index,
        $.field_access,
        $.method_call_expr,
        $.qualified_call_expr,
        $.function_call_expr,
        $.identifier,
        $.value_literal
      ),
      optional(';')
    ),

    unary_expression: $ => prec(10, choice(
      seq('!', choice($.identifier, $.value_literal, $.expression)),
      seq('-', choice($.identifier, $.value_literal, $.expression)),
      seq('&', choice($.identifier, $.field_access, $.array_index)),
      seq('*', choice($.identifier, $.field_access, $.array_index, $.parenthesized_expression)),
    )),

    deref_expr: $ => prec(10, seq(
      '*',
      choice($.identifier, $.parenthesized_expression)
    )),

    binary_expression: $ => {
      const table = [
        [prec.left, 2, '||'],
        [prec.left, 3, '&&'],
        [prec.left, 4, '|'],
        [prec.left, 5, '&'],
        [prec.left, 6, choice('==', '!=')],
        [prec.left, 7, choice('<', '<=', '>', '>=')],
        [prec.left, 8, choice('+', '-')],
        [prec.left, 9, choice('*', '/', '%')],
      ];

      const operand = $ => choice(
        $.expression,
        $.array_index,
        $.field_access,
        $.method_call_expr,
        $.function_call_expr,
        $.qualified_call_expr,
        $.identifier,
        $.value_literal
      );

      return choice(...table.map(([fn, precedence, operator]) =>
        fn(precedence, seq(
          field('left', operand($)),
          field('operator', operator),
          field('right', operand($))
        ))
      ));
    },

    parenthesized_expression: $ => seq(
      '(',
      choice($.expression, $.array_index, $.field_access, $.identifier, $.value_literal),
      ')'
    ),

    expression: $ => choice(
      $.unary_expression,
      $.binary_expression,
      $.parenthesized_expression,
      $.deref_expr
    ),

    fieldInit: $ => seq(
      field('name', $.identifier),
      ':',
      $._expr_value,
      optional(',')
    ),

    structInit: $ => seq(
      field('name', $.identifier),
      '{',
      repeat($.fieldInit),
      '}',
    ),

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
    number: $ => /\d+/,
    string: $ => /"(?:[^"\\]|\\.)*"/,
    boolean: $ => choice('true', 'false'),
    comment: $ => token(choice(
      seq('//', /.*/),
      seq('/*', /[^*]*\*+(?:[^/*][^*]*\*+)*/, '/')
    ))
  }
});
