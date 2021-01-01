use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::foreign_declaration_compiler::ForeignDeclarationCompiler;
use super::function_compiler_factory::FunctionCompilerFactory;
use super::global_variable::GlobalVariable;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ModuleCompiler<'c> {
    context: &'c inkwell::context::Context,
    module: Arc<inkwell::module::Module<'c>>,
    function_compiler_factory: Arc<FunctionCompilerFactory<'c>>,
    foreign_declaration_compiler: Arc<ForeignDeclarationCompiler<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    compile_configuration: Arc<CompileConfiguration>,
}

impl<'c> ModuleCompiler<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: Arc<inkwell::module::Module<'c>>,
        function_compiler_factory: Arc<FunctionCompilerFactory<'c>>,
        foreign_declaration_compiler: Arc<ForeignDeclarationCompiler<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        compile_configuration: Arc<CompileConfiguration>,
    ) -> Self {
        Self {
            context,
            module,
            function_compiler_factory,
            foreign_declaration_compiler,
            type_compiler,
            compile_configuration,
        }
    }

    pub fn compile(&self, ir_module: &ssf::ir::Module) -> Result<(), CompileError> {
        self.declare_intrinsics();

        let mut global_variables = HashMap::<String, GlobalVariable<'c>>::new();

        for declaration in ir_module.foreign_declarations() {
            self.foreign_declaration_compiler
                .compile(&mut global_variables, declaration)?;
        }

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

        global_value.set_constant(!definition.is_thunk());
        global_value.set_initializer(
            &closure_type.const_named_struct(&[
                self.function_compiler_factory
                    .create(global_variables.clone())
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
        let pointer_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::Generic);

        self.module.add_function(
            &self.compile_configuration.malloc_function_name,
            pointer_type.fn_type(
                &[self.type_compiler.compile_pointer_sized_integer().into()],
                false,
            ),
            None,
        );

        self.module.add_function(
            &self.compile_configuration.realloc_function_name,
            pointer_type.fn_type(
                &[
                    pointer_type.into(),
                    self.type_compiler.compile_pointer_sized_integer().into(),
                ],
                false,
            ),
            None,
        );
    }
}
