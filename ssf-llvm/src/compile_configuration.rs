#[cfg(test)]
use lazy_static::lazy_static;

pub struct CompileConfiguration {
    pub malloc_function_name: String,
}

#[cfg(test)]
lazy_static! {
    pub static ref COMPILE_CONFIGURATION: std::sync::Arc<CompileConfiguration> =
        CompileConfiguration {
            malloc_function_name: "malloc".into()
        }
        .into();
}
