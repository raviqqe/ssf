mod error;
mod sort_global_variables;
mod type_check;

pub use error::AnalysisError;
pub(crate) use sort_global_variables::sort_global_variables;
pub(crate) use type_check::check_types;
pub use type_check::TypeCheckError;
