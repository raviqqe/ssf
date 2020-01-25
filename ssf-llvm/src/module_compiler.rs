use super::error::CompileError;
use super::expression_compiler::ExpressionCompiler;
use super::function_compiler::FunctionCompiler;
use super::initializer_configuration::InitializerConfiguration;
use super::type_compiler::TypeCompiler;
use super::utilities;
use std::collections::HashMap;

pub struct ModuleCompiler<'c, 'm, 't, 'i> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    type_compiler: &'t TypeCompiler<'c, 'm>,
    global_variables: HashMap<String, inkwell::values::GlobalValue<'c>>,
    initializers: HashMap<String, inkwell::values::FunctionValue<'c>>,
    initializer_configuration: &'i InitializerConfiguration,
}

impl<'c, 'm, 't, 'i> ModuleCompiler<'c, 'm, 't, 'i> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        type_compiler: &'t TypeCompiler<'c, 'm>,
        initializer_configuration: &'i InitializerConfiguration,
    ) -> ModuleCompiler<'c, 'm, 't, 'i> {
        ModuleCompiler {
            context,
            module,
            type_compiler,
            global_variables: HashMap::new(),
            initializers: HashMap::new(),
            initializer_configuration,
        }
    }

    pub fn compile<'s>(&mut self, ast_module: &'s ssf::ast::Module) -> Result<(), CompileError> {
        self.declare_intrinsics();

        for declaration in ast_module.declarations() {
            match declaration.type_() {
                ssf::types::Type::Function(function_type) => {
                    self.declare_function(declaration.name(), function_type)
                }
                ssf::types::Type::Value(value_type) => {
                    self.declare_global_variable(declaration.name(), value_type)
                }
            }
        }

        for definition in ast_module.definitions() {
            match definition {
                ssf::ast::Definition::FunctionDefinition(function_definition) => {
                    self.declare_function(function_definition.name(), function_definition.type_())
                }
                ssf::ast::Definition::ValueDefinition(value_definition) => {
                    self.declare_global_variable(value_definition.name(), value_definition.type_())
                }
            }
        }

        for definition in ast_module.definitions() {
            match definition {
                ssf::ast::Definition::FunctionDefinition(function_definition) => {
                    self.compile_function(function_definition)?
                }
                ssf::ast::Definition::ValueDefinition(value_definition) => {
                    self.compile_global_variable(value_definition)?
                }
            }
        }

        self.compile_module_initializer(ast_module)?;

        self.module.verify()?;

        Ok(())
    }

    fn declare_function(&mut self, name: &str, type_: &ssf::types::Function) {
        self.global_variables.insert(
            name.into(),
            self.module.add_global(
                self.type_compiler.compile_unsized_closure(type_),
                None,
                name,
            ),
        );
    }

    fn compile_function(
        &mut self,
        function_definition: &ssf::ast::FunctionDefinition,
    ) -> Result<(), CompileError> {
        let global_variable = self.global_variables[function_definition.name()];

        global_variable.set_initializer(
            &global_variable
                .as_pointer_value()
                .get_type()
                .get_element_type()
                .into_struct_type()
                .const_named_struct(&[
                    FunctionCompiler::new(
                        self.context,
                        self.module,
                        self.type_compiler,
                        &self.global_variables,
                    )
                    .compile(function_definition)?
                    .as_global_value()
                    .as_pointer_value()
                    .into(),
                    self.context.const_struct(&[], false).into(),
                ]),
        );

        Ok(())
    }

    fn declare_global_variable(&mut self, name: &str, value_type: &ssf::types::Value) {
        self.global_variables.insert(
            name.into(),
            self.module
                .add_global(self.type_compiler.compile_value(value_type), None, name),
        );
    }

    fn compile_global_variable(
        &mut self,
        value_definition: &ssf::ast::ValueDefinition,
    ) -> Result<(), CompileError> {
        let global_variable = self.global_variables[value_definition.name()];
        global_variable.set_initializer(
            utilities::get_any_type_enum_undef(
                &global_variable
                    .as_pointer_value()
                    .get_type()
                    .get_element_type(),
            )
            .as_ref(),
        );

        let initializer = self.module.add_function(
            &Self::get_initializer_name(value_definition.name()),
            self.context.void_type().fn_type(&[], false),
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(&self.context.append_basic_block(initializer, "entry"));
        builder.build_store(
            global_variable.as_pointer_value(),
            ExpressionCompiler::new(
                self.context,
                self.module,
                &builder,
                &FunctionCompiler::new(
                    self.context,
                    self.module,
                    self.type_compiler,
                    &self.global_variables,
                ),
                &self.type_compiler,
            )
            .compile(
                &value_definition.body(),
                &self
                    .global_variables
                    .iter()
                    .map(|(name, value)| (name.into(), value.as_pointer_value().into()))
                    .collect(),
            )?,
        );
        builder.build_return(None);

        initializer.verify(true);
        self.initializers
            .insert(value_definition.name().into(), initializer);

        Ok(())
    }

    fn compile_module_initializer(
        &mut self,
        ast_module: &ssf::ast::Module,
    ) -> Result<(), CompileError> {
        let flag = self.module.add_global(
            self.context.bool_type(),
            None,
            &[self.initializer_configuration.name(), "$initialized"].concat(),
        );
        flag.set_initializer(&self.context.bool_type().const_int(0, false));

        let initializer = self.module.add_function(
            self.initializer_configuration.name(),
            self.context.void_type().fn_type(&[], false),
            None,
        );

        let builder = self.context.create_builder();

        builder.position_at_end(&self.context.append_basic_block(initializer, "entry"));
        let initialize_block = self.context.append_basic_block(initializer, "initialize");
        let end_block = self.context.append_basic_block(initializer, "end");

        builder.build_conditional_branch(
            builder
                .build_load(flag.as_pointer_value(), "")
                .into_int_value(),
            &end_block,
            &initialize_block,
        );
        builder.position_at_end(&initialize_block);

        for dependent_initializer_name in
            self.initializer_configuration.dependent_initializer_names()
        {
            self.module.add_function(
                dependent_initializer_name,
                self.context.void_type().fn_type(&[], false),
                None,
            );
            builder.build_call(
                self.module
                    .get_function(dependent_initializer_name)
                    .unwrap(),
                &[],
                "",
            );
        }

        for name in ast_module.global_variable_initialization_order() {
            builder.build_call(self.initializers[name], &[], "");
        }

        builder.build_store(
            flag.as_pointer_value(),
            self.context.bool_type().const_int(1, false),
        );

        builder.build_unconditional_branch(&end_block);
        builder.position_at_end(&end_block);

        builder.build_return(None);

        Ok(())
    }

    fn get_initializer_name(name: &str) -> String {
        [name, ".$init"].concat()
    }

    fn declare_intrinsics(&self) {
        self.module.add_function(
            "malloc",
            self.context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::Generic)
                .fn_type(&[self.context.i64_type().into()], false),
            None,
        );
    }
}
