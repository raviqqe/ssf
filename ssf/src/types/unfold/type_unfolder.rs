use crate::types::*;

pub struct TypeUnfolder {
    algebraic_type: Algebraic,
    index: usize,
}

impl TypeUnfolder {
    pub fn new(algebraic_type: &Algebraic) -> Self {
        Self {
            algebraic_type: algebraic_type.clone(),
            index: 0,
        }
    }

    pub fn unfold(&self, type_: &Type) -> Type {
        match type_ {
            Type::Function(function) => Function::new(
                function
                    .arguments()
                    .iter()
                    .map(|argument| self.unfold(argument))
                    .collect(),
                self.unfold_value(function.result()),
            )
            .into(),
            Type::Value(value) => self.unfold_value(value).into(),
        }
    }

    fn unfold_value(&self, value: &Value) -> Value {
        match value {
            Value::Algebraic(algebraic) => self.unfold_algebraic(algebraic).into(),
            Value::Index(index) => {
                if *index == self.index {
                    self.algebraic_type.clone().into()
                } else {
                    Value::Index(*index)
                }
            }
            Value::Primitive(_) => value.clone(),
        }
    }

    fn unfold_algebraic(&self, algebraic: &Algebraic) -> Algebraic {
        let other = self.increment_index();

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
                                .map(|type_| other.unfold(type_))
                                .collect(),
                            constructor.is_boxed(),
                        ),
                    )
                })
                .collect(),
        )
    }

    fn increment_index(&self) -> Self {
        Self {
            algebraic_type: self.algebraic_type.clone(),
            index: self.index + 1,
        }
    }
}
