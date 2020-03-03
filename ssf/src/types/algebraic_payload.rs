use super::constructor::Constructor;
use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AlgebraicPayload {
    Boxed(Constructor),
    Unboxed(Constructor),
}

impl AlgebraicPayload {
    pub fn constructor(&self) -> &Constructor {
        match self {
            Self::Boxed(constructor) => constructor,
            Self::Unboxed(constructor) => constructor,
        }
    }

    pub fn elements(&self) -> &[Type] {
        self.constructor().elements()
    }

    pub fn is_enum(&self) -> bool {
        self.constructor().is_enum()
    }

    pub fn to_id(&self) -> String {
        match self {
            Self::Boxed(constructor) => format!("*{}", constructor.to_id()),
            Self::Unboxed(constructor) => format!("{}", constructor.to_id()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_id() {
        assert_eq!(
            &AlgebraicPayload::Boxed(Constructor::new(vec![])).to_id(),
            "*{}"
        );
        assert_eq!(
            &AlgebraicPayload::Unboxed(Constructor::new(vec![])).to_id(),
            "{}"
        );
    }
}
