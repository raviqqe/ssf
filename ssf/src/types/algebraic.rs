use super::algebraic_payload::AlgebraicPayload;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Algebraic {
    constructors: Vec<AlgebraicPayload>,
}

impl Algebraic {
    pub fn new(constructors: Vec<AlgebraicPayload>) -> Self {
        Self { constructors }
    }

    pub fn constructors(&self) -> &[AlgebraicPayload] {
        &self.constructors
    }

    pub fn is_singleton(&self) -> bool {
        self.constructors.len() == 1
    }

    pub fn is_enum(&self) -> bool {
        self.constructors
            .iter()
            .all(|constructor| constructor.is_enum())
    }

    pub fn to_id(&self) -> String {
        format!(
            "{{{}}}",
            self.constructors
                .iter()
                .map(|constructor| constructor.to_id())
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
        assert_eq!(
            &Algebraic::new(vec![Constructor::new(vec![])]).to_id(),
            "{{}}"
        );
        assert_eq!(
            &Algebraic::new(vec![Constructor::new(vec![]), Constructor::new(vec![])]).to_id(),
            "{{},{}}"
        );
        assert_eq!(
            &Algebraic::new(vec![
                Constructor::new(vec![]),
                Constructor::new(vec![Primitive::Float64.into()])
            ])
            .to_id(),
            "{{},{Float64}}"
        );
        assert_eq!(
            &Algebraic::new(vec![
                Constructor::new(vec![Primitive::Float64.into()]),
                Constructor::new(vec![])
            ])
            .to_id(),
            "{{Float64},{}}"
        );
    }
}
