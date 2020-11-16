use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::expression_compiler_factory::ExpressionCompilerFactory;
use super::function_compiler::FunctionCompiler;
use super::global_variable::GlobalVariable;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ModuleCompiler<'c> {
    context: &'c inkwell::context::Context,
    module: Arc<inkwell::module::Module<'c>>,
    expression_compiler_factory: Arc<ExpressionCompilerFactory<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    compile_configuration: Arc<CompileConfiguration>,
}

impl<'c> ModuleCompiler<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: Arc<inkwell::module::Module<'c>>,
        expression_compiler_factory: Arc<ExpressionCompilerFactory<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        compile_configuration: Arc<CompileConfiguration>,
    ) -> Self {
        Self {
            context,
            module,
            expression_compiler_factory,
            type_compiler,
            compile_configuration,
        }
    }

    pub fn compile(&self, ir_module: &ssf::ir::Module) -> Result<(), CompileError> {
        self.declare_intrinsics();

        let mut global_variables = HashMap::<String, GlobalVariable<'c>>::new();

        for declaration in ir_module.declarations() {
            self.declare_function(&mut global_variables, declaration);
        }

        for definition in ir_module.definitions() {
            self.define_function(&mut global_variables, definition);
        }

        let global_variables = Arc::new(global_variables);

        for definition in ir_module.definitions() {
            self.compile_function(global_variables.clone(), definition)?;
        }

        self.module.verify()?;

        Ok(())
    }

    fn declare_function(
        &self,
        global_variables: &mut HashMap<String, GlobalVariable<'c>>,
        declaration: &ssf::ir::Declaration,
    ) {
        global_variables.insert(
            declaration.name().into(),
            GlobalVariable::new(
                self.module.add_global(
                    self.type_compiler
                        .compile_unsized_closure(declaration.type_()),
                    None,
                    declaration.name(),
                ),
                self.type_compiler
                    .compile_unsized_closure(declaration.type_())
                    .ptr_type(inkwell::AddressSpace::Generic),
            ),
        );
    }

    fn define_function(
        &self,
        global_variables: &mut HashMap<String, GlobalVariable<'c>>,
        definition: &ssf::ir::Definition,
    ) {
        global_variables.insert(
            definition.name().into(),
            GlobalVariable::new(
                self.module.add_global(
                    self.type_compiler.compile_sized_closure(definition),
                    None,
                    definition.name(),
                ),
                self.type_compiler
                    .compile_unsized_closure(definition.type_())
                    .ptr_type(inkwell::AddressSpace::Generic),
            ),
        );
    }

    fn compile_function(
        &self,
        global_variables: Arc<HashMap<String, GlobalVariable<'c>>>,
        definition: &ssf::ir::Definition,
    ) -> Result<(), CompileError> {
        let global_value = global_variables[definition.name()].global_value();
        let closure_type = global_value
            .as_pointer_value()
            .get_type()
            .get_element_type()
            .into_struct_type();

        global_value.set_initializer(
            &closure_type.const_named_struct(&[
                FunctionCompiler::new(
                    self.context,
                    self.module.clone(),
                    self.expression_compiler_factory.clone(),
                    self.type_compiler.clone(),
                    global_variables.clone(),
                )
                .compile(definition)?
                .as_global_value()
                .as_pointer_value()
                .into(),
                self.type_compiler
                    .compile_arity()
                    .const_int(definition.arguments().len() as u64, false)
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
