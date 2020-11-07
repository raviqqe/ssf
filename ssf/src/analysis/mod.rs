mod error;
mod type_check;

pub use error::AnalysisError;
pub(crate) use type_check::check_types;
pub use type_check::TypeCheckError;
