pub struct CompileConfiguration {
    malloc_function_name: Option<String>,
    panic_function_name: Option<String>,
}

impl CompileConfiguration {
    pub fn new(malloc_function_name: Option<String>, panic_function_name: Option<String>) -> Self {
        Self {
            malloc_function_name,
            panic_function_name,
        }
    }

    pub fn malloc_function_name(&self) -> &str {
        self.malloc_function_name
            .as_ref()
            .map(|string| string.as_ref())
            .unwrap_or("malloc")
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
        CompileConfiguration::new(Some("".into()), None);
    }
}
