use super::algebraic::Algebraic;
use super::type_::Type;
use super::value::Value;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Function {
    arguments: Vec<Type>,
    result: Value,
}

impl Function {
    pub fn new(arguments: Vec<Type>, result: impl Into<Value>) -> Self {
        Self {
            arguments,
            result: result.into(),
        }
    }

    pub fn arguments(&self) -> &[Type] {
        &self.arguments
    }

    pub fn result(&self) -> &Value {
        &self.result
    }

    pub fn to_id(&self) -> String {
        format!(
            "({}->{})",
            self.arguments
                .iter()
                .map(|argument| argument.to_id())
                .collect::<Vec<_>>()
                .join("->"),
            self.result.to_id()
        )
    }

    pub(crate) fn unfold(&self, algebraic_type: &Algebraic) -> Self {
        Self {
            arguments: self
                .arguments
                .iter()
                .map(|argument| argument.unfold(algebraic_type))
                .collect(),
            result: self.result.unfold(algebraic_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::primitive::Primitive;
    use super::*;

    #[test]
    fn to_id() {
        assert_eq!(
            &Function::new(vec![Primitive::Float64.into()], Primitive::Float64).to_id(),
            "(Float64->Float64)"
        );
        assert_eq!(
            &Function::new(
                vec![Primitive::Float64.into(), Primitive::Float64.into()],
                Primitive::Float64
            )
            .to_id(),
            "(Float64->Float64->Float64)"
        );
        assert_eq!(
            &Function::new(
                vec![Function::new(vec![Primitive::Float64.into()], Primitive::Float64).into()],
                Primitive::Float64
            )
            .to_id(),
            "((Float64->Float64)->Float64)"
        );
    }

    #[test]
    fn arguments() {
        assert_eq!(
            Function::new(vec![Primitive::Float64.into()], Primitive::Float64).arguments(),
            &[Primitive::Float64.into()]
        );
    }

    #[test]
    fn result() {
        assert_eq!(
            Function::new(vec![Primitive::Float64.into()], Primitive::Float64).result(),
            &Primitive::Float64.into()
        );
    }
}
