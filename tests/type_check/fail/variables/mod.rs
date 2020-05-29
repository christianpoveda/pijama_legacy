use crate::{test_type, util::DummyLoc};

use pijama::ty::TyError;
use pijama::LangError;

test_type!(
    unbounded,
    Err(LangError::Ty(TyError::Unbound("x".to_owned().loc())))
);
