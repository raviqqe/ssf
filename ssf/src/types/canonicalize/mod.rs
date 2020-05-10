mod type_canonicalizer;
mod type_equality_checker;

use crate::types::Type;
use type_canonicalizer::TypeCanonicalizer;

pub(crate) fn canonicalize(type_: &Type) -> Type {
    TypeCanonicalizer::new().canonicalize(type_)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Algebraic, Constructor, Function, Primitive, Value};

    #[test]
    fn canonicalize_() {
        for (type_, canonical_type) in &[
            (Primitive::Float64.into(), Primitive::Float64.into()),
            (
                Function::new(vec![Primitive::Float64.into()], Primitive::Float64).into(),
                Function::new(vec![Primitive::Float64.into()], Primitive::Float64).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Primitive::Float64.into()])]).into(),
                Algebraic::new(vec![Constructor::boxed(vec![Primitive::Float64.into()])]).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Value::Index(0).into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Value::Index(0).into()])]).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Value::Index(1).into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Value::Index(0).into()])]).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    vec![Primitive::Float64.into()],
                    Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                        vec![Primitive::Float64.into()],
                        Value::Index(0),
                    )
                    .into()])]),
                )
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    vec![Primitive::Float64.into()],
                    Value::Index(0),
                )
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    vec![Primitive::Float64.into()],
                    Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                        vec![Primitive::Float64.into()],
                        Value::Index(1),
                    )
                    .into()])]),
                )
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    vec![Primitive::Float64.into()],
                    Value::Index(0),
                )
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Value::Index(2).into(),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Value::Index(2).into(),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
            ),
        ] {
            assert_eq!(&canonicalize(type_), canonical_type);
        }
    }
}
