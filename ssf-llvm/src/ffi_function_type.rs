pub struct FfiFunctionType {
    name: String,
    arguments: Vec<ssf::types::Type>,
    result: ssf::types::Type,
}

impl FfiFunctionType {
    pub fn new(
        name: impl Into<String>,
        arguments: Vec<ssf::types::Type>,
        result: impl Into<ssf::types::Type>,
    ) -> Self {
        Self {
            name: name.into(),
            arguments,
            result: result.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[ssf::types::Type] {
        &self.arguments
    }

    pub fn result(&self) -> &ssf::types::Type {
        &self.result
    }
}
