use super::constructor::Constructor;
use super::unfold::unfold;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Algebraic {
    constructors: BTreeMap<u64, Constructor>,
}

impl Algebraic {
    pub fn new(constructors: Vec<Constructor>) -> Self {
        Self::with_tags(
            constructors
                .into_iter()
                .enumerate()
                .map(|(tag, constructor)| (tag as u64, constructor))
                .collect(),
        )
    }

    pub fn with_tags(constructors: BTreeMap<u64, Constructor>) -> Self {
        if constructors.is_empty() {
            panic!("no constructors in algebraic data type");
        }

        Self { constructors }
    }

    pub fn constructors(&self) -> &BTreeMap<u64, Constructor> {
        &self.constructors
    }

    pub fn is_singleton(&self) -> bool {
        self.constructors.len() == 1
    }

    pub fn is_enum(&self) -> bool {
        self.constructors
            .iter()
            .all(|(_, constructor)| constructor.is_enum())
    }

    pub fn unfold(&self) -> Self {
        unfold(self)
    }
}

#[cfg(test)]
mod tests {
    use super::super::primitive::Primitive;
    use super::*;

    #[test]
    #[should_panic]
    fn new_with_no_constructor() {
        Algebraic::new(vec![]);
    }

    #[test]
    fn to_id() {
        assert_eq!(
            &Algebraic::new(vec![Constructor::boxed(vec![])]).to_id(),
            "{0:{}}"
        );
        assert_eq!(
            &Algebraic::new(vec![Constructor::boxed(vec![]), Constructor::boxed(vec![])]).to_id(),
            "{0:{},1:{}}"
        );
        assert_eq!(
            &Algebraic::new(vec![
                Constructor::boxed(vec![]),
                Constructor::boxed(vec![Primitive::Float64.into()])
            ])
            .to_id(),
            "{0:{},1:{Float64}}"
        );
        assert_eq!(
            &Algebraic::new(vec![
                Constructor::boxed(vec![Primitive::Float64.into()]),
                Constructor::boxed(vec![])
            ])
            .to_id(),
            "{0:{Float64},1:{}}"
        );
        assert_eq!(
            &Algebraic::with_tags(vec![(42, Constructor::boxed(vec![]))].into_iter().collect())
                .to_id(),
            "{2a:{}}"
        );
    }
}
