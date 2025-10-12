/// Imports
use crate::{
    cx::module::ModuleCx,
    errors::TypeckError,
    typ::{PreludeType, Typ},
};
use oil_common::{address::Address, bail};

/// Unify implementation
impl<'pkg, 'cx> ModuleCx<'pkg, 'cx> {
    pub fn unify(&mut self, l1: &Address, t1: &Typ, l2: &Address, t2: &Typ) -> Typ {
        if t1 != t2 {
            match (&t1, &t2) {
                (Typ::Prelude(a), Typ::Prelude(b)) => match (a, b) {
                    (PreludeType::Int, PreludeType::Float) => Typ::Prelude(PreludeType::Float),
                    (PreludeType::Float, PreludeType::Int) => Typ::Prelude(PreludeType::Float),
                    _ => bail!(TypeckError::CouldNotUnify {
                        src: self.module.source.clone(),
                        first_span: l1.span.clone().into(),
                        t1: t1.clone(),
                        second_span: l2.span.clone().into(),
                        t2: t2.clone()
                    }),
                },
                (Typ::Dyn, t) | (t, Typ::Dyn) => match t {
                    Typ::Void => bail!(TypeckError::CouldNotUnify {
                        src: self.module.source.clone(),
                        first_span: l1.span.clone().into(),
                        t1: t1.clone(),
                        second_span: l2.span.clone().into(),
                        t2: t2.clone()
                    }),
                    _ => Typ::Dyn,
                },
                _ => bail!(TypeckError::CouldNotUnify {
                    src: self.module.source.clone(),
                    first_span: l1.span.clone().into(),
                    t1: t1.clone(),
                    second_span: l2.span.clone().into(),
                    t2: t2.clone()
                }),
            }
        } else {
            t1.clone()
        }
    }
}
