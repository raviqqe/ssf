use inkwell::types::BasicType;
use std::cmp::max;

static EMPTY_BOXED_CONSTRUCTOR: ssf::types::Constructor =
    ssf::types::Constructor::boxed(Vec::new());

pub struct TypeCompiler<'c> {
    context: &'c inkwell::context::Context,
    target_machine: inkwell::targets::TargetMachine,
}

impl<'c> TypeCompiler<'c> {
    pub fn new(context: &'c inkwell::context::Context) -> Self {
        inkwell::targets::Target::initialize_all(&inkwell::targets::InitializationConfig::default());
        let target_triple = inkwell::targets::TargetMachine::get_default_triple();

        Self {
            context,
            target_machine: inkwell::targets::Target::from_triple(&target_triple)
                .unwrap()
                .create_target_machine(
                    &target_triple,
                    "",
                    "",
                    Default::default(),
                    inkwell::targets::RelocMode::Default,
                    inkwell::targets::CodeModel::Default,
                )
                .unwrap(),
        }
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
            ssf::types::Value::Algebraic(algebraic) => {
                self.compile_algebraic(algebraic, None).into()
            }
            ssf::types::Value::Index(_) => unreachable!(),
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
        index: Option<u64>,
    ) -> inkwell::types::StructType<'c> {
        let mut elements = vec![];

        if !algebraic.is_singleton() {
            elements.push(self.context.i64_type().into());
        }

        if !algebraic.is_enum() {
            if let Some(index) = index {
                elements.push(
                    self.compile_constructor(&algebraic.unfold().constructors()[&index], false),
                );
            } else {
                elements.push(self.compile_unsized_constructor(algebraic));
            }
        }

        self.context.struct_type(&elements, false)
    }

    pub fn compile_sized_closure(
        &self,
        definition: &ssf::ir::Definition,
    ) -> inkwell::types::StructType<'c> {
        self.context.struct_type(
            &[
                self.compile_entry_function(definition.type_())
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .into(),
                self.compile_payload(definition).into(),
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

    fn compile_payload(&self, definition: &ssf::ir::Definition) -> inkwell::types::StructType {
        let size = max(
            self.target_machine
                .get_target_data()
                .get_store_size(&self.compile_environment(definition)),
            self.target_machine
                .get_target_data()
                .get_store_size(&self.compile_value(definition.result_type())),
        );

        self.context.struct_type(
            &(0..((size as isize - 1) / 8 + 1))
                .map(|_| self.context.i64_type().into())
                .collect::<Vec<_>>(),
            false,
        )
    }

    pub fn compile_environment(
        &self,
        definition: &ssf::ir::Definition,
    ) -> inkwell::types::StructType<'c> {
        self.context.struct_type(
            &definition
                .environment()
                .iter()
                .map(|argument| self.compile(argument.type_()))
                .collect::<Vec<_>>(),
            false,
        )
    }

    pub fn compile_unsized_environment(&self) -> inkwell::types::StructType<'c> {
        self.context.struct_type(&[], false)
    }

    pub fn compile_entry_function(
        &self,
        function: &ssf::types::Function,
    ) -> inkwell::types::FunctionType<'c> {
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

    fn compile_constructor(
        &self,
        constructor: &ssf::types::Constructor,
        shallow: bool,
    ) -> inkwell::types::BasicTypeEnum<'c> {
        let type_ = self.compile_unboxed_constructor(if shallow && constructor.is_boxed() {
            &EMPTY_BOXED_CONSTRUCTOR
        } else {
            constructor
        });

        if constructor.is_boxed() {
            type_.ptr_type(inkwell::AddressSpace::Generic).into()
        } else {
            type_.into()
        }
    }

    pub fn compile_unboxed_constructor(
        &self,
        constructor: &ssf::types::Constructor,
    ) -> inkwell::types::StructType<'c> {
        self.context.struct_type(
            &constructor
                .elements()
                .iter()
                .map(|element| self.compile(element))
                .collect::<Vec<_>>(),
            false,
        )
    }

    fn compile_unsized_constructor(
        &self,
        algebraic_type: &ssf::types::Algebraic,
    ) -> inkwell::types::BasicTypeEnum<'c> {
        self.context
            .i8_type()
            .array_type(
                algebraic_type
                    .constructors()
                    .iter()
                    .map(|(_, constructor)| {
                        self.target_machine
                            .get_target_data()
                            .get_store_size(&self.compile_constructor(constructor, true))
                    })
                    .max()
                    .unwrap() as u32,
            )
            .into()
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
    fn compile_function_type_with_no_argument() {
        let context = inkwell::context::Context::create();

        TypeCompiler::new(&context)
            .compile(&ssf::types::Function::new(vec![], ssf::types::Primitive::Float64).into());
    }

    #[test]
    fn compile_algebraic_with_one_constructor() {
        let context = inkwell::context::Context::create();
        TypeCompiler::new(&context).compile(
            &ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
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
                ssf::types::Constructor::boxed(vec![ssf::types::Primitive::Float64.into()]),
                ssf::types::Constructor::boxed(vec![ssf::types::Primitive::Float64.into()]),
            ])
            .into(),
        );
    }

    #[test]
    fn compile_recursive_algebraic() {
        let context = inkwell::context::Context::create();
        TypeCompiler::new(&context).compile(
            &ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                ssf::types::Value::Index(0).into(),
            ])])
            .into(),
        );
    }

    #[test]
    fn compile_recursive_algebraic_with_constructor_content() {
        let context = inkwell::context::Context::create();
        TypeCompiler::new(&context).compile_algebraic(
            &ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                ssf::types::Value::Index(0).into(),
            ])]),
            Some(0),
        );
    }

    #[test]
    fn keep_equality_of_recursive_types() {
        let context = inkwell::context::Context::create();
        let compiler = TypeCompiler::new(&context);

        let compile_type = || {
            compiler.compile(
                &ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                    ssf::types::Value::Index(0).into(),
                ])])
                .into(),
            )
        };

        assert_eq!(compile_type(), compile_type());
    }

    #[test]
    fn compile_updatable_closure() {
        let module = ssf::ir::Module::new(
            vec![],
            vec![
                ssf::ir::Definition::new("f", vec![], 42, ssf::types::Primitive::Integer64).into(),
            ],
        )
        .unwrap();

        let context = inkwell::context::Context::create();

        TypeCompiler::new(&context).compile_sized_closure(&module.definitions()[0]);
    }
}
