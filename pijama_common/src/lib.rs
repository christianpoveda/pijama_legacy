pub mod location;

use std::fmt::{Debug, Display, Formatter, Result};

/// The different binary operators that Pijama's syntax supports.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum BinOp {
    /// Addition operator.
    Add,
    /// Subtraction operator.
    Sub,
    /// Multiplication operator.
    Mul,
    /// Division operator.
    Div,
    /// Remainder/Modulo operator.
    Rem,
    /// Logical And operator.
    And,
    /// Logical Or operator.
    Or,
    /// Bitwise And operator.
    BitAnd,
    /// Bitwise Or operator.
    BitOr,
    /// Bitwise Xor operator.
    BitXor,
    /// Right-shift operator.
    Shr,
    /// Left-shift operator.
    Shl,
    /// Equality operator.
    Eq,
    /// Not Equal operator.
    Neq,
    /// Less Than operator.
    Lt,
    /// Greater Than operator.
    Gt,
    /// Less Than Or Equal operator.
    Lte,
    /// Greater Than Or Equal operator.
    Gte,
}

impl<'a> Display for BinOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use BinOp::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
            Rem => write!(f, "%"),
            And => write!(f, "&&"),
            Or => write!(f, "||"),
            BitAnd => write!(f, "&"),
            BitOr => write!(f, "|"),
            BitXor => write!(f, "^"),
            Shr => write!(f, ">>"),
            Shl => write!(f, "<<"),
            Eq => write!(f, "=="),
            Neq => write!(f, "!="),
            Lt => write!(f, "<"),
            Gt => write!(f, ">"),
            Lte => write!(f, "<="),
            Gte => write!(f, ">="),
        }
    }
}

/// The unary operators that Pijama's syntax supports.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum UnOp {
    /// Arithmetic Negation operator.
    Neg,
    /// Logical Negation operator.
    Not,
}

impl<'a> Display for UnOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use UnOp::*;
        match self {
            Not => write!(f, "!"),
            Neg => write!(f, "-"),
        }
    }
}

/// The literal values that Pijama's syntax supports.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Literal {
    /// Boolean Literal.
    Bool(bool),
    /// Unit Literal.
    Unit,
    /// Numeric Literal.
    Number(i64),
}

impl From<i64> for Literal {
    fn from(n: i64) -> Self {
        Literal::Number(n)
    }
}

impl From<bool> for Literal {
    fn from(b: bool) -> Self {
        Literal::Bool(b)
    }
}

impl<'a> Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Literal::*;
        match self {
            Bool(b) => write!(f, "{}", b),
            Unit => write!(f, "unit"),
            Number(num) => write!(f, "{}", num),
        }
    }
}

/// The primitives that Pijama's syntax supports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Primitive {
    /// Built-in Print primitive.
    Print,
}

impl<'a> Display for Primitive {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Primitive::*;

        match self {
            Print => write!(f, "print"),
        }
    }
}

/// Represents the name of a variable or non-primitive function in the AST.
#[derive(Debug, Clone, Copy)]
pub enum Local<'a> {
    Name(&'a str),
    Wildcard,
    Temp(usize),
}

impl<'a> Display for Local<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Local::Name(name) => write!(f, "{}", name),
            Local::Wildcard => write!(f, "_"),
            Local::Temp(id) => write!(f, "#{}", id),
        }
    }
}

impl<'a> PartialEq for Local<'a> {
    fn eq(&self, other: &Self) -> bool {
        if let (Local::Name(n1), Local::Name(n2)) = (self, other) {
            n1 == n2
        } else {
            false
        }
    }
}

impl<'a> Eq for Local<'a> {}
