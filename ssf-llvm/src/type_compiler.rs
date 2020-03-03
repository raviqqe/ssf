use inkwell::types::BasicType;

pub struct TypeCompiler<'c> {
    context: &'c inkwell::context::Context,
}

impl<'c> TypeCompiler<'c> {
    pub fn new(context: &'c inkwell::context::Context) -> Self {
        Self { context }
    }

    fn compile(&self, type_: &ssf::types::Type) -> inkwell::types::BasicTypeEnum<'c> {
        match type_ {
            ssf::types::Type::Function(function) => self
                .compile_unsized_closure(function)
                .ptr_type(inkwell::AddressSpace::Generic)
                .into(),
            ssf::types::Type::Value(value) => self.compile_value(value),
        }
    }

    pub fn compile_value(&self, value: &ssf::types::Value) -> inkwell::types::BasicTypeEnum<'c> {
        match value {
            ssf::types::Value::Algebraic(algebraic) => self.compile_algebraic(algebraic).into(),
            ssf::types::Value::Index(_) => self.compile_unsized_constructor().into(),
            ssf::types::Value::Primitive(primitive) => self.compile_primitive(primitive),
        }
    }

    pub fn compile_primitive(
        &self,
        primitive: &ssf::types::Primitive,
    ) -> inkwell::types::BasicTypeEnum<'c> {
        match primitive {
            ssf::types::Primitive::Float64 => self.context.f64_type().into(),
            ssf::types::Primitive::Integer8 => self.context.i8_type().into(),
            ssf::types::Primitive::Integer64 => self.context.i64_type().into(),
        }
    }

    pub fn compile_algebraic(
        &self,
        algebraic: &ssf::types::Algebraic,
    ) -> inkwell::types::StructType<'c> {
        if algebraic.is_singleton() && algebraic.is_enum() {
            self.context.struct_type(&[], false)
        } else if algebraic.is_singleton() {
            self.context
                .struct_type(&[self.compile_unsized_constructor().into()], false)
        } else if algebraic.is_enum() {
            self.context
                .struct_type(&[self.context.i64_type().into()], false)
        } else {
            self.context.struct_type(
                &[
                    self.context.i64_type().into(),
                    self.compile_unsized_constructor().into(),
                ],
                false,
            )
        }
    }

    pub fn compile_closure(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> inkwell::types::StructType<'c> {
        self.context.struct_type(
            &[
                self.compile_entry_function(function_definition.type_())
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .into(),
                self.compile_environment(function_definition.environment())
                    .into(),
            ],
            false,
        )
    }

    pub fn compile_unsized_closure(
        &self,
        function: &ssf::types::Function,
    ) -> inkwell::types::StructType<'c> {
        self.context.struct_type(
            &[
                self.compile_entry_function(function)
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .into(),
                self.compile_unsized_environment().into(),
            ],
            false,
        )
    }

    fn compile_environment(
        &self,
        free_variables: &[ssf::ir::Argument],
    ) -> inkwell::types::StructType {
        self.context.struct_type(
            &free_variables
                .iter()
                .map(|argument| self.compile(argument.type_()))
                .collect::<Vec<_>>(),
            false,
        )
    }

    fn compile_unsized_environment(&self) -> inkwell::types::StructType<'c> {
        self.context.struct_type(&[], false)
    }

    fn compile_entry_function(
        &self,
        function: &ssf::types::Function,
    ) -> inkwell::types::FunctionType {
        let mut arguments = vec![self
            .compile_unsized_environment()
            .ptr_type(inkwell::AddressSpace::Generic)
            .into()];

        arguments.extend_from_slice(
            &function
                .arguments()
                .iter()
                .map(|type_| self.compile(type_))
                .collect::<Vec<_>>(),
        );

        self.compile_value(function.result())
            .fn_type(&arguments, false)
    }

    fn compile_boxed_constructor(
        &self,
        constructor: &ssf::types::BoxedConstructor,
    ) -> inkwell::types::BasicTypeEnum<'c> {
        match constructor {
            ssf::types::BoxedConstructor(constructor) => self.compile_constructor(constructor),
            ssf::types::UnboxedConstructor(constructor) => self.compile_constructor(constructor),
        }
        .ptr_type(inkwell::AddressSpace::Generic)
    }

    pub fn compile_constructor(
        &self,
        constructor: &ssf::types::Constructor,
    ) -> inkwell::types::PointerType<'c> {
        self.context.struct_type(
            &constructor
                .elements()
                .iter()
                .map(|element| self.compile(element))
                .collect::<Vec<_>>(),
            false,
        )
    }

    pub fn compile_unsized_constructor(&self) -> inkwell::types::PointerType<'c> {
        self.context
            .struct_type(&[], false)
            .ptr_type(inkwell::AddressSpace::Generic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_number() {
        let context = inkwell::context::Context::create();
        TypeCompiler::new(&context).compile(&ssf::types::Primitive::Float64.into());
    }

    #[test]
    fn compile_function() {
        let context = inkwell::context::Context::create();
        TypeCompiler::new(&context).compile(
            &ssf::types::Function::new(
                vec![ssf::types::Primitive::Float64.into()],
                ssf::types::Primitive::Float64,
            )
            .into(),
        );
    }

    #[test]
    fn compile_function_twice() {
        let context = inkwell::context::Context::create();
        let compiler = TypeCompiler::new(&context);
        let type_ = ssf::types::Function::new(
            vec![ssf::types::Primitive::Float64.into()],
            ssf::types::Primitive::Float64,
        )
        .into();

        assert_eq!(compiler.compile(&type_), compiler.compile(&type_));
    }

    #[test]
    fn compile_algebraic_with_one_constructor() {
        let context = inkwell::context::Context::create();
        TypeCompiler::new(&context).compile(
            &ssf::types::Algebraic::new(vec![ssf::types::Constructor::new(vec![
                ssf::types::Primitive::Float64.into(),
            ])])
            .into(),
        );
    }

    #[test]
    fn compile_algebraic_with_two_constructors() {
        let context = inkwell::context::Context::create();
        TypeCompiler::new(&context).compile(
            &ssf::types::Algebraic::new(vec![
                ssf::types::Constructor::new(vec![ssf::types::Primitive::Float64.into()]),
                ssf::types::Constructor::new(vec![ssf::types::Primitive::Float64.into()]),
            ])
            .into(),
        );
    }

    #[test]
    fn compile_recursive_algebraic() {
        let context = inkwell::context::Context::create();
        TypeCompiler::new(&context).compile(
            &ssf::types::Algebraic::new(vec![ssf::types::Constructor::new(vec![
                ssf::types::Value::Index(0).into(),
            ])])
            .into(),
        );
    }

    #[test]
    fn keep_equality_of_recursive_types() {
        let context = inkwell::context::Context::create();
        let compiler = TypeCompiler::new(&context);

        let compile_type = || {
            compiler.compile(
                &ssf::types::Algebraic::new(vec![ssf::types::Constructor::new(vec![
                    ssf::types::Value::Index(0).into(),
                ])])
                .into(),
            )
        };

        assert_eq!(compile_type(), compile_type());
    }
}
