use super::type_equality_checker::TypeEqualityChecker;
use crate::types::{Algebraic, Constructor, Function, Type, Value};

pub struct TypeCanonicalizer<'a> {
    types: Vec<&'a Algebraic>,
}

impl<'a> TypeCanonicalizer<'a> {
    pub fn new() -> Self {
        Self { types: vec![] }
    }

    pub fn canonicalize(&self, type_: &Type) -> Type {
        match type_ {
            Type::Value(value_type) => self.canonicalize_value_type(value_type).into(),
            Type::Function(function) => Function::new(
                function
                    .arguments()
                    .iter()
                    .map(|argument| self.canonicalize(argument))
                    .collect(),
                self.canonicalize_value_type(function.result()),
            )
            .into(),
        }
    }

    fn canonicalize_value_type(&self, type_: &Value) -> Value {
        match type_ {
            Value::Algebraic(algebraic) => {
                for (index, parent_type) in self.types.iter().enumerate() {
                    if TypeEqualityChecker::new(&self.types)
                        .equal_algebraics(algebraic, parent_type)
                    {
                        return Value::Index(index);
                    }
                }

                let other = self.push_type(algebraic);

                Algebraic::new(
                    algebraic
                        .constructors()
                        .iter()
                        .map(|constructor| {
                            Constructor::new(
                                constructor
                                    .elements()
                                    .iter()
                                    .map(|element| other.canonicalize(element))
                                    .collect(),
                            )
                        })
                        .collect(),
                )
                .into()
            }
            _ => type_.clone(),
        }
    }

    fn push_type(&'a self, type_: &'a Algebraic) -> Self {
        Self {
            types: [type_].iter().chain(&self.types).cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize() {
        for (type_, canonical_type) in &[
            (Value::Number.into(), Value::Number.into()),
            (
                Function::new(vec![Value::Number.into()], Value::Number.into()).into(),
                Function::new(vec![Value::Number.into()], Value::Number.into()).into(),
            ),
        ] {
            assert_eq!(
                &TypeCanonicalizer::new().canonicalize(type_),
                canonical_type
            );
        }
    }
}
