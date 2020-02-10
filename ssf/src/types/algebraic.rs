use super::constructor::Constructor;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Algebraic {
    constructors: Vec<Constructor>,
}

impl Algebraic {
    pub fn new(mut constructors: Vec<Constructor>) -> Self {
        constructors.sort();
        Self { constructors }
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
    use super::super::value::Value;
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
                Constructor::new(vec![Value::Number.into()]),
                Constructor::new(vec![])
            ])
            .to_id(),
            &Algebraic::new(vec![
                Constructor::new(vec![]),
                Constructor::new(vec![Value::Number.into()])
            ])
            .to_id()
        );
    }
}
