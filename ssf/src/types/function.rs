use super::type_::Type;
use super::value::Value;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Function {
    arguments: Vec<Type>,
    result: Rc<Value>,
}

impl Function {
    pub fn new(arguments: Vec<Type>, result: Value) -> Self {
        Self {
            arguments,
            result: Rc::new(result),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_id() {
        assert_eq!(
            &Function::new(vec![Value::Float64.into()], Value::Float64).to_id(),
            "(Float64->Float64)"
        );
        assert_eq!(
            &Function::new(
                vec![Value::Float64.into(), Value::Float64.into()],
                Value::Float64
            )
            .to_id(),
            "(Float64->Float64->Float64)"
        );
        assert_eq!(
            &Function::new(
                vec![Function::new(vec![Value::Float64.into()], Value::Float64).into()],
                Value::Float64
            )
            .to_id(),
            "((Float64->Float64)->Float64)"
        );
    }

    #[test]
    fn arguments() {
        assert_eq!(
            Function::new(vec![Value::Float64.into()], Value::Float64).arguments(),
            &[Value::Float64.into()]
        );
    }

    #[test]
    fn result() {
        assert_eq!(
            Function::new(vec![Value::Float64.into()], Value::Float64).result(),
            &Value::Float64
        );
    }
}
