pub struct CompileConfiguration {
    initializer_name: String,
    dependent_initializer_names: Vec<String>,
}

impl CompileConfiguration {
    pub fn new(
        initializer_name: impl Into<String>,
        dependent_initializer_names: Vec<String>,
    ) -> Self {
        Self {
            initializer_name: initializer_name.into(),
            dependent_initializer_names,
        }
    }

    pub fn initializer_name(&self) -> &str {
        &self.initializer_name
    }

    pub fn dependent_initializer_names(&self) -> &[String] {
        &self.dependent_initializer_names
    }
}
