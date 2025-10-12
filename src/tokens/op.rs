use super::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assoc {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct OpInfo {
    pub prec: u8,
    pub assoc: Assoc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Assign,
    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,
    PlusEqual,
    MinusEqual,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
    BoleanAnd,
    BooleanOr,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    pub fn from_str(op: &str) -> Option<Self> {
        match op {
            "=" => Some(Op::Assign),
            "+" => Some(Op::Plus),
            "-" => Some(Op::Minus),
            "*" => Some(Op::Multiply),
            "/" => Some(Op::Divide),
            "%" => Some(Op::Mod),
            "+=" => Some(Op::PlusEqual),
            "-=" => Some(Op::MinusEqual),
            "==" => Some(Op::EqualEqual),
            "!=" => Some(Op::BangEqual),
            ">" => Some(Op::Greater),
            ">=" => Some(Op::GreaterEqual),
            "<" => Some(Op::Less),
            "<=" => Some(Op::LessEqual),
            "and" => Some(Op::And),
            "or" => Some(Op::Or),
            "&&" => Some(Op::BoleanAnd),
            "||" => Some(Op::BooleanOr),
            "!" => Some(Op::Not),
            "&" => Some(Op::BitAnd),
            "|" => Some(Op::BitOr),
            "^" => Some(Op::BitXor),
            "~" => Some(Op::BitNot),
            "<<" => Some(Op::Shl),
            ">>" => Some(Op::Shr),
            _ => None,
        }
    }

    pub fn from_token(token: &Token) -> Option<Self> {
        match token.kind() {
            TokenKind::Assign => Some(Op::Assign),
            TokenKind::Plus => Some(Op::Plus),
            TokenKind::Minus => Some(Op::Minus),
            TokenKind::Multiply => Some(Op::Multiply),
            TokenKind::Divide => Some(Op::Divide),
            TokenKind::PlusEqual => Some(Op::PlusEqual),
            TokenKind::MinusEqual => Some(Op::MinusEqual),
            TokenKind::EqualEqual => Some(Op::EqualEqual),
            TokenKind::BangEqual => Some(Op::BangEqual),
            TokenKind::Greater => Some(Op::Greater),
            TokenKind::GreaterEqual => Some(Op::GreaterEqual),
            TokenKind::Less => Some(Op::Less),
            TokenKind::LessEqual => Some(Op::LessEqual),
            TokenKind::And => Some(Op::And),
            TokenKind::Or => Some(Op::Or),
            TokenKind::BoleanAnd => Some(Op::BoleanAnd),
            TokenKind::BooleanOr => Some(Op::BooleanOr),
            TokenKind::Bang => Some(Op::Not),
            _ => None,
        }
    }

    pub fn get_precedence(&self) -> Option<OpInfo> {
        match self {
            Op::Multiply | Op::Divide | Op::Mod | Op::Mul | Op::Div => Some(OpInfo {
                prec: 5,
                assoc: Assoc::Left,
            }),
            Op::Plus | Op::Minus | Op::Add | Op::Sub => Some(OpInfo {
                prec: 4,
                assoc: Assoc::Left,
            }),
            Op::Greater
            | Op::Less
            | Op::GreaterEqual
            | Op::LessEqual
            | Op::Gt
            | Op::Lt
            | Op::Ge
            | Op::Le => Some(OpInfo {
                prec: 3,
                assoc: Assoc::Left,
            }),
            Op::EqualEqual | Op::BangEqual | Op::Eq | Op::Ne => Some(OpInfo {
                prec: 2,
                assoc: Assoc::Left,
            }),
            Op::And | Op::BoleanAnd | Op::BitAnd => Some(OpInfo {
                prec: 1,
                assoc: Assoc::Left,
            }),
            Op::Or | Op::BooleanOr | Op::BitOr | Op::BitXor => Some(OpInfo {
                prec: 0,
                assoc: Assoc::Left,
            }),
            _ => None,
        }
    }
}
