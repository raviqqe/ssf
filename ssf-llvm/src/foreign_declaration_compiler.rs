use super::error::CompileError;
use super::global_variable::GlobalVariable;
use super::type_compiler::TypeCompiler;
use super::utilities::FUNCTION_ARGUMENT_OFFSET;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ForeignDeclarationCompiler<'c> {
    context: &'c inkwell::context::Context,
    module: Arc<inkwell::module::Module<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
}

impl<'c> ForeignDeclarationCompiler<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: Arc<inkwell::module::Module<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
    ) -> Arc<Self> {
        Self {
            context,
            module,
            type_compiler,
        }
        .into()
    }

    pub fn compile(
        &self,
        global_variables: &mut HashMap<String, GlobalVariable<'c>>,
        declaration: &ssf::ir::ForeignDeclaration,
    ) -> Result<(), CompileError> {
        let closure_type = self
            .type_compiler
            .compile_unsized_closure(declaration.type_());
        let global_value = self
            .module
            .add_global(closure_type, None, declaration.name());

        global_value.set_constant(true);
        global_value.set_initializer(
            &closure_type.const_named_struct(&[
                self.compile_entry_function(declaration)
                    .as_global_value()
                    .as_pointer_value()
                    .into(),
                self.type_compiler
                    .compile_arity()
                    .const_int(
                        declaration.type_().arguments().into_iter().count() as u64,
                        false,
                    )
                    .into(),
                closure_type.get_field_types()[2]
                    .into_struct_type()
                    .get_undef()
                    .into(),
            ]),
        );

        global_variables.insert(
            declaration.name().into(),
            GlobalVariable::new(
                global_value,
                closure_type.ptr_type(inkwell::AddressSpace::Generic),
            ),
        );

        Ok(())
    }

    fn compile_entry_function(
        &self,
        declaration: &ssf::ir::ForeignDeclaration,
    ) -> inkwell::values::FunctionValue<'c> {
        let entry_function = self.module.add_function(
            &format!("{}.entry", declaration.name()),
            self.type_compiler.compile_entry_function(
                declaration.type_().arguments(),
                declaration.type_().last_result(),
            ),
            Some(inkwell::module::Linkage::Private),
        );

        let builder = Arc::new(self.context.create_builder());
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        builder.build_return(Some(
            &builder
                .build_call(
                    self.module.add_function(
                        declaration.foreign_name(),
                        self.type_compiler
                            .compile_foreign_function(declaration.type_()),
                        None,
                    ),
                    &entry_function.get_params()[FUNCTION_ARGUMENT_OFFSET..],
                    "",
                )
                .try_as_basic_value()
                .left()
                .unwrap(),
        ));

        entry_function
    }
}
