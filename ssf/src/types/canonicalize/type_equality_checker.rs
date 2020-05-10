use crate::types::*;

pub struct TypeEqualityChecker<'a> {
    pairs: Vec<(&'a Algebraic, &'a Algebraic)>,
}

impl<'a> TypeEqualityChecker<'a> {
    pub fn new(types: &'a [&'a Algebraic]) -> Self {
        Self {
            pairs: types.iter().cloned().zip(types.iter().cloned()).collect(),
        }
    }

    pub fn equal_algebraics(&self, one: &Algebraic, other: &Algebraic) -> bool {
        if one.constructors().len() != other.constructors().len() {
            return false;
        } else if self.pairs.contains(&(one, other)) {
            return true;
        }

        let checker = self.push_pair(one, other);

        one.constructors()
            .iter()
            .zip(other.constructors())
            .all(|(one, other)| one.0 == other.0 && checker.equal_constructors(one.1, other.1))
    }

    fn equal_values(&self, one: &Value, other: &Value) -> bool {
        match (one, other) {
            (Value::Primitive(one), Value::Primitive(other)) => one == other,
            (Value::Algebraic(one), Value::Algebraic(other)) => self.equal_algebraics(one, other),
            (Value::Index(index), Value::Algebraic(other)) => {
                self.equal_algebraics(self.pairs[*index].0, other)
            }
            (Value::Algebraic(other), Value::Index(index)) => {
                self.equal_algebraics(other, self.pairs[*index].1)
            }
            (Value::Index(one), Value::Index(other)) => {
                self.equal_algebraics(self.pairs[*one].0, self.pairs[*other].1)
            }
            _ => false,
        }
    }

    fn equal(&self, one: &Type, other: &Type) -> bool {
        match (one, other) {
            (Type::Value(one), Type::Value(other)) => self.equal_values(one, other),
            (Type::Function(one), Type::Function(other)) => {
                one.arguments().len() == other.arguments().len()
                    && one
                        .arguments()
                        .iter()
                        .zip(other.arguments())
                        .all(|(one, other)| self.equal(one, other))
                    && self.equal_values(one.result(), other.result())
            }
            (_, _) => false,
        }
    }

    fn equal_constructors(&self, one: &Constructor, other: &Constructor) -> bool {
        one.elements().len() == other.elements().len()
            && one.is_boxed() == other.is_boxed()
            && one
                .elements()
                .iter()
                .zip(other.elements())
                .all(|(one, other)| self.equal(one, other))
    }

    fn push_pair(&'a self, one: &'a Algebraic, other: &'a Algebraic) -> Self {
        Self {
            pairs: [(one, other)]
                .iter()
                .chain(self.pairs.iter())
                .copied()
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal() {
        for (one, other) in &[
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
                Algebraic::new(vec![Constructor::boxed(vec![Value::Index(0).into()])]).into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Value::Index(0).into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Value::Index(0).into()])]).into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Value::Index(1).into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Value::Index(0).into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Value::Index(1).into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    vec![Primitive::Float64.into()],
                    Value::Index(0),
                )
                .into()])])
                .into(),
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
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    vec![Primitive::Float64.into()],
                    Value::Index(0),
                )
                .into()])])
                .into(),
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
            ),
        ] {
            assert!(TypeEqualityChecker::new(&[]).equal(one, other));
        }
    }

    #[test]
    fn not_equal() {
        for (one, other) in &[
            (
                Primitive::Float64.into(),
                Function::new(vec![Primitive::Float64.into()], Primitive::Float64).into(),
            ),
            (
                Function::new(
                    vec![Primitive::Float64.into(), Primitive::Float64.into()],
                    Primitive::Float64,
                )
                .into(),
                Function::new(vec![Primitive::Float64.into()], Primitive::Float64).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Primitive::Float64.into()])]).into(),
                Algebraic::new(vec![Constructor::boxed(vec![
                    Primitive::Float64.into(),
                    Primitive::Float64.into(),
                ])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Primitive::Float64.into()])]).into(),
                Algebraic::new(vec![Constructor::unboxed(vec![Primitive::Float64.into()])]).into(),
            ),
        ] {
            assert!(!TypeEqualityChecker::new(&[]).equal(one, other));
        }
    }
}
