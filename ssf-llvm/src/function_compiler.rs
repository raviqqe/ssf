use super::error::CompileError;
use super::expression_compiler::ExpressionCompiler;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;

pub struct FunctionCompiler<'c, 'm, 't, 'v> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    type_compiler: &'t TypeCompiler<'c, 'm>,
    global_variables: &'v HashMap<String, inkwell::values::GlobalValue<'c>>,
}

impl<'c, 'm, 't, 'v> FunctionCompiler<'c, 'm, 't, 'v> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        type_compiler: &'t TypeCompiler<'c, 'm>,
        global_variables: &'v HashMap<String, inkwell::values::GlobalValue<'c>>,
    ) -> Self {
        Self {
            context,
            module,
            type_compiler,
            global_variables,
        }
    }

    pub fn compile(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> Result<inkwell::values::FunctionValue, CompileError> {
        let closure_type = self.type_compiler.compile_closure(function_definition);

        let entry_function = self.module.add_function(
            &Self::generate_closure_entry_name(function_definition.name()),
            closure_type.get_field_types()[0]
                .into_pointer_type()
                .get_element_type()
                .into_function_type(),
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(&self.context.append_basic_block(entry_function, "entry"));

        let environment = builder
            .build_bitcast(
                entry_function.get_params()[0],
                closure_type.get_field_types()[1]
                    .into_struct_type()
                    .ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value();

        let mut variables = self
            .global_variables
            .iter()
            .map(|(name, value)| (name.into(), value.as_pointer_value().into()))
            .collect::<HashMap<String, inkwell::values::BasicValueEnum>>();

        for (index, free_variable) in function_definition.environment().iter().enumerate() {
            variables.insert(
                free_variable.name().into(),
                builder.build_load(
                    unsafe {
                        builder.build_gep(
                            environment,
                            &[
                                self.context.i32_type().const_int(0, false),
                                self.context.i32_type().const_int(index as u64, false),
                            ],
                            "",
                        )
                    },
                    "",
                ),
            );
        }

        for (index, argument) in function_definition.arguments().iter().enumerate() {
            variables.insert(
                argument.name().into(),
                entry_function.get_params()[index + 1],
            );
        }

        let expression_compiler = ExpressionCompiler::new(
            self.context,
            self.module,
            &builder,
            self,
            self.type_compiler,
        );
        builder.build_return(Some(
            &expression_compiler.compile(&function_definition.body(), &variables)?,
        ));

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn generate_closure_entry_name(name: &str) -> String {
        [name, ".$entry"].concat()
    }
}
