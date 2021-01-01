use super::closure_operation_compiler::ClosureOperationCompiler;
use super::error::CompileError;
use super::malloc_compiler::MallocCompiler;
use super::type_compiler::TypeCompiler;
use super::utilities::{self, FUNCTION_ARGUMENT_OFFSET};
use inkwell::types::BasicType;
use std::sync::Arc;

pub struct FunctionApplicationCompiler<'c> {
    context: &'c inkwell::context::Context,
    module: Arc<inkwell::module::Module<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    closure_operation_compiler: Arc<ClosureOperationCompiler<'c>>,
    malloc_compiler: Arc<MallocCompiler<'c>>,
}

impl<'c> FunctionApplicationCompiler<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: Arc<inkwell::module::Module<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        closure_operation_compiler: Arc<ClosureOperationCompiler<'c>>,
        malloc_compiler: Arc<MallocCompiler<'c>>,
    ) -> Arc<Self> {
        Self {
            context,
            module,
            type_compiler,
            closure_operation_compiler,
            malloc_compiler,
        }
        .into()
    }

    // Closures' entry points are always uncurried.
    pub fn compile(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
        arguments: &[inkwell::values::BasicValueEnum<'c>],
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        let switch_block = builder.get_insert_block().unwrap();
        let phi_block = self.append_basic_block(builder, "phi");

        let cases = (1..=arguments.len())
            .map(|arity| {
                let block = self.append_basic_block(builder, &format!("pa_arity_{}", arity));
                builder.position_at_end(block);

                let mut value =
                    self.compile_direct_closure_call(&builder, closure, &arguments[..arity]);

                if arity != arguments.len() {
                    value =
                        self.compile(builder, value.into_pointer_value(), &arguments[arity..])?;
                }

                builder.build_unconditional_branch(phi_block);

                Ok((arity, block, value, builder.get_insert_block().unwrap()))
            })
            .collect::<Result<Vec<_>, CompileError>>()?;

        let default_block = self.append_basic_block(builder, "pa_default");
        builder.position_at_end(default_block);
        let default_value = self.compile_create_closure(builder, closure, &arguments)?;
        if default_value.is_some() {
            builder.build_unconditional_branch(phi_block);
        }

        builder.position_at_end(switch_block);
        builder.build_switch(
            self.closure_operation_compiler
                .compile_load_arity(&builder, closure),
            default_block,
            &cases
                .iter()
                .map(|(arity, block, _, _)| {
                    (
                        self.type_compiler
                            .compile_arity()
                            .const_int(*arity as u64, false),
                        *block,
                    )
                })
                .collect::<Vec<_>>(),
        );

        builder.position_at_end(phi_block);
        let phi = builder.build_phi(cases.get(0).unwrap().2.get_type(), "");
        phi.add_incoming(
            &cases
                .iter()
                .map(|(_, _, value, block)| (value as &dyn inkwell::values::BasicValue<'c>, *block))
                .collect::<Vec<_>>(),
        );

        if let Some(default_value) = default_value {
            phi.add_incoming(&[(&default_value, default_block)]);
        }

        Ok(phi.as_basic_value())
    }

    fn compile_partially_applied_function(
        &self,
        function_type: inkwell::types::FunctionType<'c>,
        environment_type: inkwell::types::StructType<'c>,
    ) -> Result<inkwell::values::FunctionValue<'c>, CompileError> {
        let entry_function = utilities::add_function_to_module(
            self.module.clone(),
            "pa_entry",
            self.type_compiler
                .compile_curried_entry_function(function_type, 1),
        );

        let builder = Arc::new(self.context.create_builder());
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        let environment = builder
            .build_load(
                builder
                    .build_bitcast(
                        entry_function.get_params()[0],
                        environment_type.ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
                "",
            )
            .into_struct_value();

        let closure = builder
            .build_extract_value(environment, 0, "")
            .unwrap()
            .into_pointer_value();
        let arguments = (1..environment.get_type().count_fields())
            .map(|index| builder.build_extract_value(environment, index, "").unwrap())
            .chain(vec![entry_function.get_params()[1]])
            .collect::<Vec<_>>();

        let then_block = self.append_basic_block(&builder, "then");
        let else_block = self.append_basic_block(&builder, "else");

        builder.build_conditional_branch(
            builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                self.closure_operation_compiler
                    .compile_load_arity(&builder, closure),
                self.type_compiler
                    .compile_arity()
                    .const_int(arguments.len() as u64, false),
                "",
            ),
            then_block,
            else_block,
        );

        builder.position_at_end(then_block);
        builder.build_return(Some(
            &self.compile_direct_closure_call(&builder, closure, &arguments),
        ));

        builder.position_at_end(else_block);
        if let Some(value) = self.compile_create_closure(&builder, closure, &arguments)? {
            builder.build_return(Some(&value));
        }

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn compile_direct_closure_call(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
        arguments: &[inkwell::values::BasicValueEnum<'c>],
    ) -> inkwell::values::BasicValueEnum<'c> {
        let entry_pointer = self
            .closure_operation_compiler
            .compile_load_entry_pointer(&builder, closure);

        builder
            .build_call(
                builder
                    .build_bitcast(
                        entry_pointer,
                        self.type_compiler
                            .compile_curried_entry_function(
                                entry_pointer
                                    .get_type()
                                    .get_element_type()
                                    .into_function_type(),
                                arguments.len(),
                            )
                            .ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
                &vec![
                    todo!(),
                    todo!(),
                    self.closure_operation_compiler
                        .compile_load_environment(&builder, closure),
                ]
                .into_iter()
                .chain(arguments.iter().copied())
                .collect::<Vec<_>>(),
                "",
            )
            .try_as_basic_value()
            .left()
            .unwrap()
    }

    fn compile_create_closure(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
        arguments: &[inkwell::values::BasicValueEnum<'c>],
    ) -> Result<Option<inkwell::values::PointerValue<'c>>, CompileError> {
        let entry_function_type = closure
            .get_type()
            .get_element_type()
            .into_struct_type()
            .get_field_type_at_index(0)
            .unwrap()
            .into_pointer_type()
            .get_element_type()
            .into_function_type();

        if arguments.len() == utilities::get_arity(entry_function_type) {
            // A number of arguments is equal to the max arity.
            builder.build_unreachable();

            Ok(None)
        } else {
            let environment_values = vec![closure.into()]
                .into_iter()
                .chain(arguments.iter().copied())
                .collect::<Vec<_>>();

            let environment_type = self
                .type_compiler
                .compile_raw_environment(environment_values.iter().map(|value| value.get_type()));
            let target_function_type = entry_function_type.get_return_type().unwrap().fn_type(
                &entry_function_type.get_param_types()[..FUNCTION_ARGUMENT_OFFSET]
                    .iter()
                    .copied()
                    .chain(
                        entry_function_type.get_param_types()
                            [FUNCTION_ARGUMENT_OFFSET + arguments.len()..]
                            .iter()
                            .copied(),
                    )
                    .collect::<Vec<_>>(),
                false,
            );

            let function =
                self.compile_partially_applied_function(target_function_type, environment_type)?;

            let closure = self.malloc_compiler.compile_struct_malloc(
                &builder,
                self.type_compiler
                    .compile_raw_closure(function.get_type(), environment_type),
            );

            self.closure_operation_compiler
                .compile_store_closure_content(&builder, closure, function, &environment_values)?;

            Ok(Some(
                builder
                    .build_bitcast(
                        closure,
                        self.type_compiler
                            .compile_raw_closure(
                                target_function_type,
                                self.type_compiler.compile_unsized_environment(),
                            )
                            .ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
            ))
        }
    }

    fn append_basic_block(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        name: &str,
    ) -> inkwell::basic_block::BasicBlock<'c> {
        self.context.append_basic_block(
            builder.get_insert_block().unwrap().get_parent().unwrap(),
            name,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::compile_configuration::COMPILE_CONFIGURATION;
    use super::*;

    fn create_function_application_compiler(
        context: &inkwell::context::Context,
    ) -> (
        Arc<FunctionApplicationCompiler>,
        Arc<TypeCompiler>,
        inkwell::builder::Builder,
        inkwell::values::FunctionValue,
    ) {
        let module = Arc::new(context.create_module(""));

        module.add_function(
            &COMPILE_CONFIGURATION.malloc_function_name,
            context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::Generic)
                .fn_type(&[context.i64_type().into()], false),
            None,
        );

        let function = module.add_function("", context.void_type().fn_type(&[], false), None);
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function, "entry"));

        let type_compiler = TypeCompiler::new(context);
        let closure_operation_compiler =
            ClosureOperationCompiler::new(context, type_compiler.clone());
        let malloc_compiler = MallocCompiler::new(module.clone(), COMPILE_CONFIGURATION.clone());

        (
            FunctionApplicationCompiler::new(
                &context,
                module,
                type_compiler.clone(),
                closure_operation_compiler.clone(),
                malloc_compiler,
            ),
            type_compiler,
            builder,
            function,
        )
    }

    #[test]
    fn apply_1_argument_to_1_arity_function() {
        let context = inkwell::context::Context::create();
        let (function_application_compiler, type_compiler, builder, function) =
            create_function_application_compiler(&context);

        function_application_compiler
            .compile(
                &builder,
                type_compiler
                    .compile_raw_closure(
                        context.f64_type().fn_type(
                            &[
                                type_compiler
                                    .compile_unsized_environment()
                                    .ptr_type(inkwell::AddressSpace::Generic)
                                    .into(),
                                context.f64_type().into(),
                            ],
                            false,
                        ),
                        type_compiler.compile_unsized_environment(),
                    )
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .get_undef(),
                &[context.f64_type().const_float(42.0).into()],
            )
            .unwrap();

        builder.build_return(None);

        assert!(function.verify(true));
    }

    #[test]
    fn apply_1_argument_to_2_arity_function() {
        let context = inkwell::context::Context::create();
        let (function_application_compiler, type_compiler, builder, function) =
            create_function_application_compiler(&context);

        function_application_compiler
            .compile(
                &builder,
                type_compiler
                    .compile_raw_closure(
                        context.f64_type().fn_type(
                            &[
                                type_compiler
                                    .compile_unsized_environment()
                                    .ptr_type(inkwell::AddressSpace::Generic)
                                    .into(),
                                context.f64_type().into(),
                                context.f64_type().into(),
                            ],
                            false,
                        ),
                        type_compiler.compile_unsized_environment(),
                    )
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .get_undef(),
                &[context.f64_type().const_zero().into()],
            )
            .unwrap();

        builder.build_return(None);

        assert!(function.verify(true));
    }

    #[test]
    fn apply_2_argument_to_2_arity_function() {
        let context = inkwell::context::Context::create();
        let (function_application_compiler, type_compiler, builder, function) =
            create_function_application_compiler(&context);

        function_application_compiler
            .compile(
                &builder,
                type_compiler
                    .compile_raw_closure(
                        context.f64_type().fn_type(
                            &[
                                type_compiler
                                    .compile_unsized_environment()
                                    .ptr_type(inkwell::AddressSpace::Generic)
                                    .into(),
                                context.f64_type().into(),
                                context.f64_type().into(),
                            ],
                            false,
                        ),
                        type_compiler.compile_unsized_environment(),
                    )
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .get_undef(),
                &[
                    context.f64_type().const_zero().into(),
                    context.f64_type().const_zero().into(),
                ],
            )
            .unwrap();

        builder.build_return(None);

        assert!(function.verify(true));
    }

    #[test]
    fn apply_1_argument_to_3_arity_function() {
        let context = inkwell::context::Context::create();
        let (function_application_compiler, type_compiler, builder, function) =
            create_function_application_compiler(&context);

        function_application_compiler
            .compile(
                &builder,
                type_compiler
                    .compile_raw_closure(
                        context.f64_type().fn_type(
                            &[
                                type_compiler
                                    .compile_unsized_environment()
                                    .ptr_type(inkwell::AddressSpace::Generic)
                                    .into(),
                                context.f64_type().into(),
                                context.f64_type().into(),
                                context.f64_type().into(),
                            ],
                            false,
                        ),
                        type_compiler.compile_unsized_environment(),
                    )
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .get_undef(),
                &[context.f64_type().const_zero().into()],
            )
            .unwrap();

        builder.build_return(None);

        assert!(function.verify(true));
    }

    #[test]
    fn apply_2_argument_to_3_arity_function() {
        let context = inkwell::context::Context::create();
        let (function_application_compiler, type_compiler, builder, function) =
            create_function_application_compiler(&context);

        function_application_compiler
            .compile(
                &builder,
                type_compiler
                    .compile_raw_closure(
                        context.f64_type().fn_type(
                            &[
                                type_compiler
                                    .compile_unsized_environment()
                                    .ptr_type(inkwell::AddressSpace::Generic)
                                    .into(),
                                context.f64_type().into(),
                                context.f64_type().into(),
                                context.f64_type().into(),
                            ],
                            false,
                        ),
                        type_compiler.compile_unsized_environment(),
                    )
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .get_undef(),
                &[
                    context.f64_type().const_zero().into(),
                    context.f64_type().const_zero().into(),
                ],
            )
            .unwrap();

        builder.build_return(None);

        assert!(function.verify(true));
    }

    #[test]
    fn apply_3_argument_to_3_arity_function() {
        let context = inkwell::context::Context::create();
        let (function_application_compiler, type_compiler, builder, function) =
            create_function_application_compiler(&context);

        function_application_compiler
            .compile(
                &builder,
                type_compiler
                    .compile_raw_closure(
                        context.f64_type().fn_type(
                            &[
                                type_compiler
                                    .compile_unsized_environment()
                                    .ptr_type(inkwell::AddressSpace::Generic)
                                    .into(),
                                context.f64_type().into(),
                                context.f64_type().into(),
                                context.f64_type().into(),
                            ],
                            false,
                        ),
                        type_compiler.compile_unsized_environment(),
                    )
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .get_undef(),
                &[
                    context.f64_type().const_zero().into(),
                    context.f64_type().const_zero().into(),
                    context.f64_type().const_zero().into(),
                ],
            )
            .unwrap();

        builder.build_return(None);

        assert!(function.verify(true));
    }
}
