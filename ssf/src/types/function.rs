use super::type_::Type;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Function {
    argument: Arc<Type>,
    result: Arc<Type>,
}

impl Function {
    pub fn new(argument: impl Into<Type>, result: impl Into<Type>) -> Self {
        Self {
            argument: argument.into().into(),
            result: result.into().into(),
        }
    }

    pub fn argument(&self) -> &Type {
        &self.argument
    }

    pub fn result(&self) -> &Type {
        &self.result
    }

    pub fn to_id(&self) -> String {
        format!("({}->{})", self.argument.to_id(), self.result.to_id())
    }
}

#[cfg(test)]
mod tests {
    use super::super::primitive::Primitive;
    use super::*;

    #[test]
    fn to_id() {
        assert_eq!(
            &Function::new(Primitive::Float64, Primitive::Float64).to_id(),
            "(Float64->Float64)"
        );
        assert_eq!(
            &Function::new(
                Primitive::Float64,
                Function::new(Primitive::Float64, Primitive::Float64),
            )
            .to_id(),
            "(Float64->(Float64->Float64))"
        );
        assert_eq!(
            &Function::new(
                Function::new(Primitive::Float64, Primitive::Float64),
                Primitive::Float64
            )
            .to_id(),
            "((Float64->Float64)->Float64)"
        );
    }

    #[test]
    fn argument() {
        assert_eq!(
            Function::new(Primitive::Float64, Primitive::Float64).argument(),
            &Primitive::Float64.into()
        );
    }

    #[test]
    fn result() {
        assert_eq!(
            Function::new(Primitive::Float64, Primitive::Float64).result(),
            &Primitive::Float64.into()
        );
    }
}
