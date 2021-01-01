#[cfg(test)]
use lazy_static::lazy_static;

pub struct CompileConfiguration {
    pub malloc_function_name: String,
    pub realloc_function_name: String,
}

#[cfg(test)]
lazy_static! {
    pub static ref COMPILE_CONFIGURATION: std::sync::Arc<CompileConfiguration> =
        CompileConfiguration {
            malloc_function_name: "malloc".into(),
            realloc_function_name: "realloc".into()
        }
        .into();
}
