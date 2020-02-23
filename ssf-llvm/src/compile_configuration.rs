pub struct CompileConfiguration {
    initializer_name: String,
    dependent_initializer_names: Vec<String>,
    malloc_function_name: Option<String>,
    panic_function_name: Option<String>,
}

impl CompileConfiguration {
    pub fn new(
        initializer_name: impl Into<String>,
        dependent_initializer_names: Vec<String>,
        malloc_function_name: Option<String>,
        panic_function_name: Option<String>,
    ) -> Self {
        Self {
            initializer_name: initializer_name.into(),
            dependent_initializer_names,
            malloc_function_name,
            panic_function_name,
        }
    }

    pub fn initializer_name(&self) -> &str {
        &self.initializer_name
    }

    pub fn dependent_initializer_names(&self) -> &[String] {
        &self.dependent_initializer_names
    }

    pub fn malloc_function_name(&self) -> Option<&str> {
        self.malloc_function_name
            .as_ref()
            .map(|string| string.as_ref())
    }

    pub fn panic_function_name(&self) -> Option<&str> {
        self.panic_function_name
            .as_ref()
            .map(|string| string.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        CompileConfiguration::new("", vec![], Some("".into()), None);
    }
}
