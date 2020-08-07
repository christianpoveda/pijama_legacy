use pijama_common::{location::Located, Local};
use pijama_hir::{LetKind, Term as HirTerm};

use crate::Term;

pub fn remove_names(term: Located<HirTerm<'_>>) -> Term {
    Context::default().remove_names(term.content)
}

#[derive(Default)]
struct Context<'a> {
    inner: Vec<Local<'a>>,
}

impl<'a> Context<'a> {
    fn remove_names(&mut self, term: HirTerm<'a>) -> Term {
        match term {
            HirTerm::Lit(lit) => lit.into(),
            HirTerm::Var(name) => {
                let (index, _) = self
                    .inner
                    .iter()
                    .rev()
                    .enumerate()
                    .find(|(_, name2)| name == **name2)
                    .unwrap();
                Term::Var(index)
            }
            HirTerm::Abs(name, _, body) => {
                self.inner.push(name);
                let body = self.remove_names(body.content);
                self.inner.pop().unwrap();
                Term::Abs(Box::new(body))
            }
            HirTerm::UnaryOp(op, t1) => {
                let t1 = self.remove_names(t1.content);
                Term::UnaryOp(op, Box::new(t1))
            }
            HirTerm::BinaryOp(op, t1, t2) => {
                let t1 = self.remove_names(t1.content);
                let t2 = self.remove_names(t2.content);
                Term::BinaryOp(op, Box::new(t1), Box::new(t2))
            }
            HirTerm::App(t1, t2) => {
                let t1 = self.remove_names(t1.content);
                let t2 = self.remove_names(t2.content);
                Term::App(Box::new(t1), Box::new(t2))
            }
            HirTerm::Let(kind, name, t1, t2) => {
                let t1 = if let LetKind::Rec(_) = kind {
                    // if the let binding is recursive we are dealing with a recursive function and
                    // we need its name inside the context to lower its body.
                    //
                    // Also the indices must be shifted by one because the function will be wrapped
                    // in an additional abstraction.
                    //
                    // Both things are satisfied by just pushing the name of the function into the
                    // context.
                    self.inner.push(name.content);
                    Term::Fix(Box::new(Term::Abs(Box::new(self.remove_names(t1.content)))))
                } else {
                    // if the let binding is non-recursive, we first lower the binded term, and
                    // then we make its name availabe by pushing it into the context
                    let t1 = self.remove_names(t1.content);
                    self.inner.push(name.content);
                    t1
                };

                let t2 = self.remove_names(t2.content);
                self.inner.pop().unwrap();
                Term::App(Box::new(Term::Abs(Box::new(t2))), Box::new(t1))
            }
            HirTerm::Cond(t1, t2, t3) => {
                let t1 = self.remove_names(t1.content);
                let t2 = self.remove_names(t2.content);
                let t3 = self.remove_names(t3.content);
                Term::Cond(Box::new(t1), Box::new(t2), Box::new(t3))
            }
            HirTerm::PrimFn(prim) => Term::PrimFn(prim),
        }
    }
}
