use super::algebraic::Algebraic;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Algebraic(Algebraic),
    Index(usize),
    Number,
}

impl Value {
    pub fn to_id(&self) -> String {
        match self {
            Self::Algebraic(algebraic) => algebraic.to_id(),
            Self::Index(index) => format!("{}", index),
            Self::Number => "Number".into(),
        }
    }

    pub fn into_algebraic(self) -> Option<Algebraic> {
        match self {
            Self::Algebraic(algebraic) => Some(algebraic),
            _ => None,
        }
    }
}

impl From<Algebraic> for Value {
    fn from(algebraic: Algebraic) -> Self {
        Self::Algebraic(algebraic)
    }
}
