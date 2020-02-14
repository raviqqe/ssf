use crate::types::{Algebraic, Constructor, Type, Value};

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
            .all(|(one, other)| checker.equal_constructors(one, other))
    }

    fn equal_values(&self, one: &Value, other: &Value) -> bool {
        match (one, other) {
            (Value::Number, Value::Number) => true,
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
    use crate::types::{Algebraic, Function};

    #[test]
    fn equal() {
        for (one, other) in &[
            (Value::Number.into(), Value::Number.into()),
            (
                Function::new(vec![Value::Number.into()], Value::Number.into()).into(),
                Function::new(vec![Value::Number.into()], Value::Number.into()).into(),
            ),
            (
                Algebraic::new(vec![Constructor::new(vec![Value::Number.into()])]).into(),
                Algebraic::new(vec![Constructor::new(vec![Value::Number.into()])]).into(),
            ),
            (
                Algebraic::new(vec![Constructor::new(vec![Value::Index(0).into()])]).into(),
                Algebraic::new(vec![Constructor::new(vec![Algebraic::new(vec![
                    Constructor::new(vec![Value::Index(0).into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::new(vec![Value::Index(0).into()])]).into(),
                Algebraic::new(vec![Constructor::new(vec![Algebraic::new(vec![
                    Constructor::new(vec![Value::Index(1).into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::new(vec![Function::new(
                    vec![Value::Number.into()],
                    Value::Index(0).into(),
                )
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::new(vec![Function::new(
                    vec![Value::Number.into()],
                    Algebraic::new(vec![Constructor::new(vec![Function::new(
                        vec![Value::Number.into()],
                        Value::Index(0).into(),
                    )
                    .into()])])
                    .into(),
                )
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::new(vec![Function::new(
                    vec![Value::Number.into()],
                    Value::Index(0).into(),
                )
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::new(vec![Function::new(
                    vec![Value::Number.into()],
                    Algebraic::new(vec![Constructor::new(vec![Function::new(
                        vec![Value::Number.into()],
                        Value::Index(1).into(),
                    )
                    .into()])])
                    .into(),
                )
                .into()])])
                .into(),
            ),
        ] {
            assert!(TypeEqualityChecker::new(&[]).equal(one, other));
        }

        for (one, other) in &[
            (
                Value::Number.into(),
                Function::new(vec![Value::Number.into()], Value::Number.into()).into(),
            ),
            (
                Function::new(
                    vec![Value::Number.into(), Value::Number.into()],
                    Value::Number.into(),
                )
                .into(),
                Function::new(vec![Value::Number.into()], Value::Number.into()).into(),
            ),
            (
                Algebraic::new(vec![Constructor::new(vec![Value::Number.into()])]).into(),
                Algebraic::new(vec![Constructor::new(vec![
                    Value::Number.into(),
                    Value::Number.into(),
                ])])
                .into(),
            ),
        ] {
            assert!(!TypeEqualityChecker::new(&[]).equal(one, other));
        }
    }
}
