use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::instruction_compiler::InstructionCompiler;
use super::type_compiler::TypeCompiler;
use super::utilities;
use inkwell::types::BasicType;
use std::sync::Arc;

pub struct FunctionApplicationCompiler<'c> {
    context: &'c inkwell::context::Context,
    module: Arc<inkwell::module::Module<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    compile_configuration: Arc<CompileConfiguration>,
}

impl<'c> FunctionApplicationCompiler<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: Arc<inkwell::module::Module<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        compile_configuration: Arc<CompileConfiguration>,
    ) -> Arc<Self> {
        Self {
            context,
            module,
            type_compiler,
            compile_configuration,
        }
        .into()
    }

    // Closures' entry points are always uncurried.
    pub fn compile(
        &self,
        builder: Arc<inkwell::builder::Builder<'c>>,
        closure: inkwell::values::PointerValue<'c>,
        arguments: &[inkwell::values::BasicValueEnum<'c>],
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        let switch_block = builder.get_insert_block().unwrap();
        let phi_block = self.append_basic_block(builder.clone(), "phi");

        let cases = (1..=arguments.len())
            .map(|arity| {
                let block =
                    self.append_basic_block(builder.clone(), &format!("pa_arity_{}", arity));
                builder.position_at_end(block);

                let mut value =
                    self.compile_direct_closure_call(&builder, closure, &arguments[..arity]);

                if arity != arguments.len() {
                    value = self.compile(
                        builder.clone(),
                        value.into_pointer_value(),
                        &arguments[arity..],
                    )?;
                }

                builder.build_unconditional_branch(phi_block);

                Ok((arity, block, value, builder.get_insert_block().unwrap()))
            })
            .collect::<Result<Vec<_>, CompileError>>()?;

        let default_block = self.append_basic_block(builder.clone(), "pa_default");
        builder.position_at_end(default_block);
        let default_value = self.compile_create_closure(builder.clone(), closure, &arguments)?;
        if default_value.is_some() {
            builder.build_unconditional_branch(phi_block);
        }

        builder.position_at_end(switch_block);
        builder.build_switch(
            self.compile_load_arity(&builder, closure),
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
        let entry_function = self.module.add_function(
            "pa_entry",
            self.type_compiler
                .compile_curried_entry_function(function_type, 1),
            None,
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

        let then_block = self.append_basic_block(builder.clone(), "then");
        let else_block = self.append_basic_block(builder.clone(), "else");

        builder.build_conditional_branch(
            builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                self.compile_load_arity(&builder, closure),
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
        if let Some(value) = self.compile_create_closure(builder.clone(), closure, &arguments)? {
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
        let entry_pointer = self.compile_load_entry_pointer(&builder, closure);

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
                &vec![self.compile_load_environment(&builder, closure)]
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
        builder: Arc<inkwell::builder::Builder<'c>>,
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
                &vec![entry_function_type.get_param_types()[0]]
                    .into_iter()
                    .chain(
                        entry_function_type.get_param_types()[arguments.len() + 1..]
                            .iter()
                            .copied(),
                    )
                    .collect::<Vec<_>>(),
                false,
            );

            let function =
                self.compile_partially_applied_function(target_function_type, environment_type)?;

            let closure = self.compile_struct_malloc(
                builder.clone(),
                self.type_compiler
                    .compile_raw_closure(function.get_type(), environment_type),
            );

            self.compile_store_closure_content(
                builder.clone(),
                closure,
                function,
                &environment_values,
            )?;

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

    fn compile_load_entry_pointer(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        // Entry functions of thunks need to be loaded atomically
        // to make thunk update thread-safe.
        InstructionCompiler::compile_atomic_load(&builder, unsafe {
            builder.build_gep(
                closure,
                &[
                    self.context.i32_type().const_int(0, false),
                    self.context.i32_type().const_int(0, false),
                ],
                "",
            )
        })
        .into_pointer_value()
    }

    fn compile_load_arity(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
    ) -> inkwell::values::IntValue<'c> {
        builder
            .build_load(
                builder
                    .build_bitcast(
                        unsafe {
                            builder.build_gep(
                                closure,
                                &[
                                    self.context.i32_type().const_int(0, false),
                                    self.context.i32_type().const_int(1, false),
                                ],
                                "",
                            )
                        },
                        self.type_compiler
                            .compile_arity()
                            .ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
                "",
            )
            .into_int_value()
    }

    fn compile_load_environment(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
    ) -> inkwell::values::BasicValueEnum<'c> {
        builder.build_bitcast(
            unsafe {
                builder.build_gep(
                    closure,
                    &[
                        self.context.i32_type().const_int(0, false),
                        self.context.i32_type().const_int(2, false),
                    ],
                    "",
                )
            },
            self.type_compiler
                .compile_unsized_environment()
                .ptr_type(inkwell::AddressSpace::Generic),
            "",
        )
    }

    // TODO Share this with ExpressionCompiler.
    fn compile_store_closure_content(
        &self,
        builder: Arc<inkwell::builder::Builder<'c>>,
        closure_pointer: inkwell::values::PointerValue<'c>,
        entry_function: inkwell::values::FunctionValue<'c>,
        environment_values: &[inkwell::values::BasicValueEnum<'c>],
    ) -> Result<(), CompileError> {
        let environment_type = self
            .type_compiler
            .compile_raw_environment(environment_values.iter().map(|value| value.get_type()));

        let closure = builder
            .build_insert_value(
                self.type_compiler
                    .compile_raw_closure(entry_function.get_type(), environment_type)
                    .get_undef(),
                entry_function.as_global_value().as_pointer_value(),
                0,
                "",
            )
            .unwrap();

        let closure = builder
            .build_insert_value(
                closure,
                self.type_compiler.compile_arity().const_int(
                    utilities::get_arity(entry_function.get_type()) as u64,
                    false,
                ),
                1,
                "",
            )
            .unwrap();

        let closure = builder
            .build_insert_value(
                closure,
                {
                    let mut environment = environment_type.get_undef();

                    for (index, value) in environment_values.iter().copied().enumerate() {
                        environment = builder
                            .build_insert_value(environment, value, index as u32, "")
                            .unwrap()
                            .into_struct_value();
                    }

                    environment
                },
                2,
                "",
            )
            .unwrap();

        builder.build_store(
            builder
                .build_bitcast(
                    closure_pointer,
                    closure
                        .into_struct_value()
                        .get_type()
                        .ptr_type(inkwell::AddressSpace::Generic),
                    "",
                )
                .into_pointer_value(),
            closure,
        );

        Ok(())
    }

    // TODO Share this with ExpressionCompiler.
    fn compile_struct_malloc(
        &self,
        builder: Arc<inkwell::builder::Builder<'c>>,
        type_: inkwell::types::StructType<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        builder
            .build_bitcast(
                builder
                    .build_call(
                        self.module
                            .get_function(self.compile_configuration.malloc_function_name())
                            .unwrap(),
                        &[type_.size_of().unwrap().into()],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap(),
                type_.ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value()
    }

    // TODO Share this with ExpressionCompiler.
    fn append_basic_block(
        &self,
        builder: Arc<inkwell::builder::Builder<'c>>,
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
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref COMPILE_CONFIGURATION: Arc<CompileConfiguration> =
            CompileConfiguration::new(None, None);
    }

    fn create_function_application_compiler(
        context: &inkwell::context::Context,
    ) -> (
        Arc<FunctionApplicationCompiler>,
        Arc<TypeCompiler>,
        Arc<inkwell::builder::Builder>,
        inkwell::values::FunctionValue,
    ) {
        let module = context.create_module("");

        module.add_function(
            COMPILE_CONFIGURATION.malloc_function_name(),
            context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::Generic)
                .fn_type(&[context.i64_type().into()], false),
            None,
        );

        let function = module.add_function("", context.void_type().fn_type(&[], false), None);
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function, "entry"));

        let type_compiler = TypeCompiler::new(&context);

        (
            FunctionApplicationCompiler::new(
                &context,
                module.into(),
                type_compiler.clone(),
                COMPILE_CONFIGURATION.clone(),
            ),
            type_compiler,
            builder.into(),
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
                builder.clone(),
                type_compiler
                    .compile_closure_struct(
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
                builder.clone(),
                type_compiler
                    .compile_closure_struct(
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
                builder.clone(),
                type_compiler
                    .compile_closure_struct(
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
                builder.clone(),
                type_compiler
                    .compile_closure_struct(
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
                builder.clone(),
                type_compiler
                    .compile_closure_struct(
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
                builder.clone(),
                type_compiler
                    .compile_closure_struct(
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
