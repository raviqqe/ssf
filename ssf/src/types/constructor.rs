use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Constructor {
    elements: Vec<Type>,
    boxed: bool,
}

impl Constructor {
    pub const fn new(elements: Vec<Type>, boxed: bool) -> Self {
        Self { elements, boxed }
    }

    pub const fn boxed(elements: Vec<Type>) -> Self {
        Self::new(elements, true)
    }

    pub const fn unboxed(elements: Vec<Type>) -> Self {
        Self::new(elements, false)
    }

    pub fn elements(&self) -> &[Type] {
        &self.elements
    }

    pub fn is_boxed(&self) -> bool {
        self.boxed
    }

    pub fn is_enum(&self) -> bool {
        self.elements.is_empty() && !self.boxed
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
    fn is_enum() {
        assert!(Constructor::unboxed(vec![]).is_enum());
        assert!(!Constructor::boxed(vec![]).is_enum());
        assert!(!Constructor::unboxed(vec![Primitive::Float64.into()]).is_enum());
        assert!(!Constructor::boxed(vec![Primitive::Float64.into()]).is_enum());
    }

    #[test]
    fn to_id() {
        assert_eq!(&Constructor::boxed(vec![]).to_id(), "{}");
        assert_eq!(
            &Constructor::boxed(vec![Primitive::Float64.into()]).to_id(),
            "{Float64}"
        );
        assert_eq!(
            &Constructor::boxed(vec![Primitive::Float64.into(), Primitive::Float64.into()]).to_id(),
            "{Float64,Float64}"
        );
    }
}
