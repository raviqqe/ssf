mod type_unfolder;

use crate::types::*;
use type_unfolder::TypeUnfolder;

pub(crate) fn unfold(algebraic: &Algebraic) -> Algebraic {
    Algebraic::with_tags(
        algebraic
            .constructors()
            .iter()
            .map(|(tag, constructor)| {
                (
                    *tag,
                    Constructor::new(
                        constructor
                            .elements()
                            .iter()
                            .map(|type_| canonicalize(&TypeUnfolder::new(algebraic).unfold(type_)))
                            .collect(),
                        constructor.is_boxed(),
                    ),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn unfold_() {
        for (type_, unfolded_type) in &[
            (
                Algebraic::new(vec![Constructor::boxed(vec![Value::Index(0).into()])]).into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Value::Index(0).into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::unboxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Value::Index(1).into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::unboxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Algebraic::new(vec![Constructor::unboxed(vec![
                        Value::Index(1).into(),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![]),
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Value::Index(2).into(),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![]),
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Algebraic::new(vec![Constructor::boxed(vec![Value::Index(2).into()])])
                            .into(),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
            ),
        ] {
            assert_eq!(&unfold(type_), unfolded_type);
        }
    }
}
