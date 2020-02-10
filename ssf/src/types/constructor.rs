use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Constructor {
    elements: Vec<Type>,
}

impl Constructor {
    pub fn new(elements: Vec<Type>) -> Self {
        Self { elements }
    }

    pub fn to_id(&self) -> String {
        format!(
            "{{{}}}",
            self.elements
                .iter()
                .map(|element| element.to_id())
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::value::Value;
    use super::*;

    #[test]
    fn to_id() {
        assert_eq!(&Constructor::new(vec![]).to_id(), "{}");
        assert_eq!(
            &Constructor::new(vec![Value::Number.into()]).to_id(),
            "{Number}"
        );
        assert_eq!(
            &Constructor::new(vec![Value::Number.into(), Value::Number.into()]).to_id(),
            "{Number,Number}"
        );
    }
}
