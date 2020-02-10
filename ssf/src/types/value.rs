use super::algebraic::Algebraic;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Algebraic(Algebraic),
    Number,
}

impl Value {
    pub fn to_id(&self) -> String {
        match self {
            Self::Algebraic(algebraic) => algebraic.to_id(),
            Self::Number => "Number".into(),
        }
    }
}
