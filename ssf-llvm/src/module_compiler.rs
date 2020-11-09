use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::function_compiler::FunctionCompiler;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;

pub struct ModuleCompiler<'c, 'm, 't> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    type_compiler: &'t TypeCompiler<'c>,
    global_variables: HashMap<String, inkwell::values::GlobalValue<'c>>,
    compile_configuration: &'c CompileConfiguration,
}

impl<'c, 'm, 't> ModuleCompiler<'c, 'm, 't> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        type_compiler: &'t TypeCompiler<'c>,
        compile_configuration: &'c CompileConfiguration,
    ) -> Self {
        Self {
            context,
            module,
            type_compiler,
            global_variables: HashMap::new(),
            compile_configuration,
        }
    }

    pub fn compile(&mut self, ir_module: &ssf::ir::Module) -> Result<(), CompileError> {
        self.declare_intrinsics();

        for declaration in ir_module.declarations() {
            self.declare_function(declaration.name());
        }

        for definition in ir_module.definitions() {
            self.define_function(definition);
        }

        for definition in ir_module.definitions() {
            self.compile_function(definition)?
        }

        self.module.verify()?;

        Ok(())
    }

    fn declare_function(&mut self, name: &str) {
        self.global_variables.insert(
            name.into(),
            self.module
                .add_global(self.type_compiler.compile_unsized_closure(), None, name),
        );
    }

    fn define_function(&mut self, definition: &ssf::ir::Definition) {
        self.global_variables.insert(
            definition.name().into(),
            self.module.add_global(
                self.type_compiler.compile_sized_closure(definition),
                None,
                definition.name(),
            ),
        );
    }

    fn compile_function(&mut self, definition: &ssf::ir::Definition) -> Result<(), CompileError> {
        let global_variable = self.global_variables[definition.name()];
        let closure_type = global_variable
            .as_pointer_value()
            .get_type()
            .get_element_type()
            .into_struct_type();

        global_variable.set_initializer(
            &closure_type.const_named_struct(&[
                FunctionCompiler::new(
                    self.context,
                    self.module,
                    self.type_compiler,
                    &self.global_variables,
                    self.compile_configuration,
                )
                .compile(definition)?
                .as_global_value()
                .as_pointer_value()
                .into(),
                closure_type.get_field_types()[2]
                    .into_struct_type()
                    .get_undef()
                    .into(),
            ]),
        );

        Ok(())
    }

    fn declare_intrinsics(&self) {
        self.module.add_function(
            self.compile_configuration.malloc_function_name(),
            self.context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::Generic)
                .fn_type(&[self.context.i64_type().into()], false),
            None,
        );

        if let Some(panic_function_name) = self.compile_configuration.panic_function_name() {
            self.module.add_function(
                panic_function_name,
                self.context.void_type().fn_type(&[], false),
                None,
            );
        }
    }
}
