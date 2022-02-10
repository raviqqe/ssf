use super::{constructor::Constructor, unfold::unfold};
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
    use super::*;

    #[test]
    #[should_panic]
    fn new_with_no_constructor() {
        Algebraic::new(vec![]);
    }
}
