/**
 * @file Treesitter parser for Hades Programming Language
 * @author Karthikey Hegde <me@karthihegde.dev>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "hades",
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
        $.identifier,
        $.function_call,
        $.value_literal
      ),
      ';'
    ),
    parameter_list: $ => seq(
      '(',
      ')'
    ),
    value_literal: $ => choice(
      $.number,
      $.string,
    ),
    _type: $ => choice(
      'bool',
      'int'
    ),
    block: $ => seq(
      '{',
      repeat($._statement),
      '}'
    ),
    call_parameter_list: $ => seq(
      '(',
      repeat(choice($.identifier, $.value_literal)),
      ')'
    ),
    function_call: $ => seq(
      $.identifier,
      $.call_parameter_list,
      ';'
    ),
    _statement: $ => choice(
      $.return_statement,
      $.var_decl,
      $.function_call
    ),
    return_statement: $ => seq(
      'return',
      choice($.expression, $.identifier, $.value_literal),
      ';'
    ),
    unary_expression: $ => choice(
      seq('!', $.identifier),
      seq('-', $.identifier),
    ),
    binary_expression: $ => choice(
      seq(choice($.identifier, $.number), '+', choice($.identifier, $.number)),
      seq(choice($.identifier, $.number), '-', choice($.identifier, $.number)),
      seq(choice($.identifier, $.number), '*', choice($.identifier, $.number)),
      seq(choice($.identifier, $.number), '/', choice($.identifier, $.number)),
      seq(choice($.identifier), '||', choice($.identifier)),
      seq(choice($.identifier), '&&', choice($.identifier)),
      seq(choice($.identifier), '|', choice($.identifier)),
      seq(choice($.identifier), '&', choice($.identifier)),
      seq(choice($.identifier), '=', choice($.identifier, $.value_literal)),
      seq(choice($.identifier, $.number), '%', choice($.identifier, $.number)),
      seq(choice($.identifier, $.number), '>', choice($.identifier, $.number)),
      seq(choice($.identifier, $.number), '<', choice($.identifier, $.number)),
      seq(choice($.identifier, $.number), '>=', choice($.identifier, $.number)),
      seq(choice($.identifier, $.number), '<=', choice($.identifier, $.number)),
      seq(choice($.identifier, $.value_literal), '==', choice($.identifier, $.value_literal)),
    ),
    expression: $ => choice(
      $.unary_expression,
      $.binary_expression
    ),

    identifier: $ => /[a-z]+/,
    number: $ => /\d+/,
    string: $ => /"(?:[^"\\]|\\.)*"/
  }
});
