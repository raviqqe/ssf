use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Constructor {
    elements: Vec<Type>,
}

impl Constructor {
    pub fn new(elements: Vec<Type>) -> Self {
        Self { elements }
    }

    pub fn elements(&self) -> &[Type] {
        &self.elements
    }

    pub fn is_enum(&self) -> bool {
        self.elements.is_empty()
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
    use super::super::primitive::Primitive;
    use super::*;

    #[test]
    fn to_id() {
        assert_eq!(&Constructor::new(vec![]).to_id(), "{}");
        assert_eq!(
            &Constructor::new(vec![Primitive::Float64.into()]).to_id(),
            "{Float64}"
        );
        assert_eq!(
            &Constructor::new(vec![Primitive::Float64.into(), Primitive::Float64.into()]).to_id(),
            "{Float64,Float64}"
        );
    }
}
