use std::fmt;

use crate::ast::Name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Bool,
    Int,
    Unit,
    Arrow(Box<Ty>, Box<Ty>),
}

impl<'a> fmt::Display for Ty {
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binding<'a> {
    pub name: Name<'a>,
    pub ty: Ty,
}
