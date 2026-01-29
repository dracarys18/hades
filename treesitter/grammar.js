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
  rules: {
    source_file: $ => repeat($._definition),
    _definition: $ => choice(
      $.function_definition,
      $.var_decl,
      $.function_call
    ),

    function_definition: $ => seq(
      'fn',
      $.identifier,
      $.parameter_list,
      ':',
      $._type,
      $.block
    ),
    var_decl: $ => seq(
      'let',
      $.identifier,
      '=',
      choice(
        $.expression,
        $.identifier,
        $.function_call_expr,
        $.value_literal,
        $.structInit
      ),
      ';'
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
      $.identifier,
      ':',
      $._type
    ),
    value_literal: $ => choice(
      $.number,
      $.string,
      $.boolean
    ),
    _type: $ => choice(
      'bool',
      'int',
      'void',
      $.identifier  // for custom types
    ),
    block: $ => seq(
      '{',
      repeat($._statement),
      '}'
    ),
    call_parameter_list: $ => seq(
      '(',
      optional(seq(
        choice($.expression, $.identifier, $.value_literal),
        repeat(seq(',', choice($.expression, $.identifier, $.value_literal)))
      )),
      ')'
    ),
    function_call: $ => seq(
      $.identifier,
      $.call_parameter_list,
      ';'
    ),
    function_call_expr: $ => seq(
      $.identifier,
      $.call_parameter_list
    ),
    _statement: $ => choice(
      $.return_statement,
      $.var_decl,
      $.function_call,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.assignment_statement
    ),
    return_statement: $ => seq(
      'return',
      choice($.expression, $.identifier, $.value_literal),
      ';'
    ),
    if_statement: $ => seq(
      'if',
      '(',
      choice($.expression, $.identifier, $.value_literal),
      ')',
      $.block,
      optional(seq('else', choice($.block, $.if_statement)))
    ),
    while_statement: $ => seq(
      'while',
      '(',
      choice($.expression, $.identifier, $.value_literal),
      ')',
      $.block
    ),
    for_statement: $ => seq(
      'for',
      '(',
      optional($.var_decl),
      choice($.expression, $.identifier, $.value_literal),
      ';',
      optional($.assignment_statement),
      ')',
      $.block
    ),
    assignment_statement: $ => seq(
      $.identifier,
      '=',
      choice($.expression, $.identifier, $.value_literal, $.function_call_expr, $.structInit),
      ';'
    ),
    unary_expression: $ => prec(10, choice(
      seq('!', choice($.identifier, $.value_literal, $.expression)),
      seq('-', choice($.identifier, $.value_literal, $.expression)),
    )),
    binary_expression: $ => {
      const table = [
        [prec.left, 1, '='],
        [prec.left, 2, '||'],
        [prec.left, 3, '&&'],
        [prec.left, 4, '|'],
        [prec.left, 5, '&'],
        [prec.left, 6, choice('==', '!=')],
        [prec.left, 7, choice('<', '<=', '>', '>=')],
        [prec.left, 8, choice('+', '-')],
        [prec.left, 9, choice('*', '/', '%')],
      ];

      return choice(...table.map(([fn, precedence, operator]) =>
        fn(precedence, seq(
          field('left', choice($.expression, $.identifier, $.value_literal, $.function_call_expr)),
          field('operator', operator),
          field('right', choice($.expression, $.identifier, $.value_literal, $.function_call_expr))
        ))
      ));
    },
    fieldInit: $ => seq(
      $.identifier,
      ':',
      choice($.identifier, $.value_literal, $.expression),
      optional(',')
    ),
    structInit: $ => seq(
      $.identifier,
      '{',
      repeat($.fieldInit),
      '}',
    ),
    expression: $ => choice(
      $.unary_expression,
      $.binary_expression
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
