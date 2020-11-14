use super::type_equality_checker::TypeEqualityChecker;
use crate::types::*;

pub struct TypeCanonicalizer<'a> {
    types: Vec<&'a Algebraic>,
}

impl<'a> TypeCanonicalizer<'a> {
    pub fn new() -> Self {
        Self { types: vec![] }
    }

    pub fn canonicalize(&self, type_: &Type) -> Type {
        match type_ {
            Type::Algebraic(algebraic) => {
                for (index, parent_type) in self.types.iter().enumerate() {
                    if TypeEqualityChecker::new(&self.types)
                        .equal_algebraics(algebraic, parent_type)
                    {
                        return Type::Index(index);
                    }
                }

                let other = self.push_type(algebraic);

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
                                        .map(|element| other.canonicalize(element))
                                        .collect(),
                                    constructor.is_boxed(),
                                ),
                            )
                        })
                        .collect(),
                )
                .into()
            }
            Type::Function(function) => Function::new(
                self.canonicalize(function.argument()),
                self.canonicalize(function.result()),
            )
            .into(),
            _ => type_.clone(),
        }
    }

    fn push_type(&'a self, type_: &'a Algebraic) -> Self {
        Self {
            types: [type_].iter().chain(&self.types).cloned().collect(),
        }
    }
}
