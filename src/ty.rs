use std::fmt;

use crate::ast::Name;

mod ty_check;

pub use ty_check::{expect_ty, ty_check, TyError, TyResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Bool,
    Int,
    Unit,
    Arrow(Box<Ty>, Box<Ty>),
    BuiltIn(Box<Ty>),
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Ty::*;
        match self {
            Bool => write!(f, "Bool"),
            Int => write!(f, "Int"),
            Unit => write!(f, "Unit"),
            Arrow(t1, t2) => {
                if let Arrow(_, _) = t1.as_ref() {
                    write!(f, "({}) -> {}", t1, t2)
                } else {
                    write!(f, "{} -> {}", t1, t2)
                }
            }
            BuiltIn(_) => write!(f, "Builtin"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Binding<'a> {
    pub name: Name<'a>,
    pub ty: Ty,
}
