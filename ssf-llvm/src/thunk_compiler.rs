use super::type_compiler::TypeCompiler;
use inkwell::types::BasicType;

pub struct ThunkCompiler<'c, 'm, 't> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    type_compiler: &'t TypeCompiler<'c>,
}

impl<'c, 'm, 't> ThunkCompiler<'c, 'm, 't> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        type_compiler: &'t TypeCompiler<'c>,
    ) -> Self {
        Self {
            context,
            module,
            type_compiler,
        }
    }

    pub fn compile_normal_thunk_entry(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> inkwell::values::FunctionValue {
        let entry_function_type = self
            .type_compiler
            .compile_entry_function(function_definition.type_());

        let entry_function = self.module.add_function(
            &Self::generate_normal_thunk_entry_name(function_definition.name()),
            entry_function_type,
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        let environment = builder
            .build_bitcast(
                entry_function.get_params()[0],
                entry_function_type
                    .get_return_type()
                    .unwrap()
                    .ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value();

        builder.build_return(Some(&builder.build_load(environment, "")));

        entry_function.verify(true);

        entry_function
    }

    fn generate_normal_thunk_entry_name(name: &str) -> String {
        [name, ".$entry.normal"].concat()
    }
}
