use super::Token;

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
}

impl Op {
    pub fn from_str(op: &str) -> Option<Self> {
        match op {
            "=" => Some(Op::Assign),
            "+" => Some(Op::Plus),
            "-" => Some(Op::Minus),
            "*" => Some(Op::Multiply),
            "/" => Some(Op::Divide),
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
            _ => None,
        }
    }

    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Assign => Some(Op::Assign),
            Token::Plus => Some(Op::Plus),
            Token::Minus => Some(Op::Minus),
            Token::Multiply => Some(Op::Multiply),
            Token::Divide => Some(Op::Divide),
            Token::PlusEqual => Some(Op::PlusEqual),
            Token::MinusEqual => Some(Op::MinusEqual),
            Token::EqualEqual => Some(Op::EqualEqual),
            Token::BangEqual => Some(Op::BangEqual),
            Token::Greater => Some(Op::Greater),
            Token::GreaterEqual => Some(Op::GreaterEqual),
            Token::Less => Some(Op::Less),
            Token::LessEqual => Some(Op::LessEqual),
            Token::And => Some(Op::And),
            Token::Or => Some(Op::Or),
            Token::BoleanAnd => Some(Op::BoleanAnd),
            Token::BooleanOr => Some(Op::BooleanOr),
            _ => None,
        }
    }

    pub fn get_precedence(&self) -> Option<OpInfo> {
        match self {
            Op::Multiply | Op::Divide => Some(OpInfo {
                prec: 5,
                assoc: Assoc::Left,
            }),
            Op::Plus | Op::Minus => Some(OpInfo {
                prec: 4,
                assoc: Assoc::Left,
            }),
            Op::Greater | Op::Less | Op::GreaterEqual | Op::LessEqual => Some(OpInfo {
                prec: 3,
                assoc: Assoc::Left,
            }),
            Op::EqualEqual | Op::BangEqual => Some(OpInfo {
                prec: 2,
                assoc: Assoc::Left,
            }),
            Op::And => Some(OpInfo {
                prec: 1,
                assoc: Assoc::Left,
            }),
            Op::Or => Some(OpInfo {
                prec: 0,
                assoc: Assoc::Left,
            }),
            _ => None,
        }
    }
}
