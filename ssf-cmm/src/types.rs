pub const FUNCTION_ARGUMENT_OFFSET: usize = 1;

pub fn compile_generic_pointer() -> cmm::types::Pointer {
    cmm::types::Pointer::new(cmm::types::Primitive::Integer8)
}

pub fn get_arity(type_: &cmm::types::Function) -> usize {
    type_.arguments().len() - FUNCTION_ARGUMENT_OFFSET
}

pub fn compile(type_: &ssf::types::Type) -> cmm::types::Type {
    match type_ {
        ssf::types::Type::Algebraic(algebraic) => compile_algebraic(algebraic, None).into(),
        ssf::types::Type::Function(function) => {
            cmm::types::Pointer::new(compile_unsized_closure(function)).into()
        }
        ssf::types::Type::Index(_) => unreachable!(),
        ssf::types::Type::Primitive(primitive) => compile_primitive(primitive),
    }
}

pub fn compile_primitive(primitive: &ssf::types::Primitive) -> cmm::types::Type {
    match primitive {
        ssf::types::Primitive::Float32 => cmm::types::Primitive::Float32.into(),
        ssf::types::Primitive::Float64 => cmm::types::Primitive::Float64.into(),
        ssf::types::Primitive::Integer8 => cmm::types::Primitive::Integer8.into(),
        ssf::types::Primitive::Integer32 => cmm::types::Primitive::Integer32.into(),
        ssf::types::Primitive::Integer64 => cmm::types::Primitive::Integer64.into(),
        ssf::types::Primitive::Pointer => {
            cmm::types::Pointer::new(cmm::types::Primitive::Integer8).into()
        }
    }
}

pub fn compile_algebraic(
    algebraic: &ssf::types::Algebraic,
    index: Option<u64>,
) -> cmm::types::Record {
    let mut elements = vec![];

    if !algebraic.is_singleton() {
        elements.push(compile_tag().into());
    }

    if !algebraic.is_enum() {
        elements.push(if let Some(index) = index {
            compile_constructor(&algebraic.unfold().constructors()[&index], false)
        } else {
            compile_untyped_constructor(algebraic).into()
        });
    }

    cmm::types::Record::new(elements)
}

pub fn compile_untyped_constructor(algebraic_type: &ssf::types::Algebraic) -> cmm::types::Union {
    cmm::types::Union::new(
        algebraic_type
            .constructors()
            .iter()
            .map(|(_, constructor)| compile_constructor(constructor, true))
            .collect(),
    )
}

pub fn compile_sized_closure(definition: &ssf::ir::Definition) -> cmm::types::Record {
    compile_raw_closure(
        compile_entry_function_from_definition(definition),
        cmm::types::Union::new(vec![
            compile_environment(definition).into(),
            compile(definition.result_type()),
        ]),
    )
}

pub fn compile_unsized_closure(type_: &ssf::types::Function) -> cmm::types::Record {
    compile_raw_closure(
        compile_uncurried_entry_function(type_),
        compile_unsized_environment(),
    )
}

pub fn compile_raw_closure(
    entry_function: cmm::types::Function,
    environment: impl Into<cmm::types::Type>,
) -> cmm::types::Record {
    cmm::types::Record::new(vec![
        entry_function.into(),
        compile_arity().into(),
        environment.into(),
    ])
}

pub fn compile_environment(definition: &ssf::ir::Definition) -> cmm::types::Record {
    compile_raw_environment(
        definition
            .environment()
            .iter()
            .map(|argument| compile(argument.type_())),
    )
}

pub fn compile_raw_environment(
    types: impl IntoIterator<Item = cmm::types::Type>,
) -> cmm::types::Record {
    cmm::types::Record::new(types.into_iter().collect())
}

pub fn compile_unsized_environment() -> cmm::types::Record {
    cmm::types::Record::new(vec![])
}

pub fn compile_curried_entry_function(
    function: &cmm::types::Function,
    arity: usize,
) -> cmm::types::Function {
    if arity == get_arity(function) {
        function.clone()
    } else {
        cmm::types::Function::new(
            function.arguments()[..arity + FUNCTION_ARGUMENT_OFFSET].to_vec(),
            cmm::types::Pointer::new(compile_raw_closure(
                cmm::types::Function::new(
                    function.arguments()[..FUNCTION_ARGUMENT_OFFSET]
                        .iter()
                        .chain(function.arguments()[arity + FUNCTION_ARGUMENT_OFFSET..].iter())
                        .cloned()
                        .collect::<Vec<_>>(),
                    function.result().clone(),
                ),
                compile_unsized_environment(),
            )),
        )
    }
}

fn compile_uncurried_entry_function(function: &ssf::types::Function) -> cmm::types::Function {
    cmm::types::Function::new(
        vec![cmm::types::Pointer::new(compile_unsized_environment()).into()]
            .into_iter()
            .chain(function.arguments().into_iter().map(compile))
            .collect(),
        compile(function.last_result()),
    )
}

pub fn compile_entry_function_from_definition(
    definition: &ssf::ir::Definition,
) -> cmm::types::Function {
    compile_entry_function(
        definition
            .arguments()
            .iter()
            .map(|argument| argument.type_()),
        definition.result_type(),
    )
}

pub fn compile_entry_function<'a>(
    arguments: impl IntoIterator<Item = &'a ssf::types::Type>,
    result: &ssf::types::Type,
) -> cmm::types::Function {
    cmm::types::Function::new(
        vec![cmm::types::Pointer::new(compile_unsized_environment()).into()]
            .into_iter()
            .chain(arguments.into_iter().map(compile))
            .collect(),
        compile(result),
    )
}

pub fn compile_foreign_function(function: &ssf::types::Function) -> cmm::types::Function {
    cmm::types::Function::new(
        function.arguments().into_iter().map(compile).collect(),
        compile(function.last_result()),
    )
}

fn compile_constructor(constructor: &ssf::types::Constructor, shallow: bool) -> cmm::types::Type {
    let type_ = compile_unboxed_constructor(&if shallow && constructor.is_boxed() {
        ssf::types::Constructor::boxed(vec![])
    } else {
        constructor.clone()
    });

    if constructor.is_boxed() {
        cmm::types::Pointer::new(type_).into()
    } else {
        type_.into()
    }
}

pub fn compile_unboxed_constructor(constructor: &ssf::types::Constructor) -> cmm::types::Record {
    cmm::types::Record::new(constructor.elements().iter().map(compile).collect())
}

pub fn compile_tag() -> cmm::types::Primitive {
    cmm::types::Primitive::PointerInteger
}

pub fn compile_arity() -> cmm::types::Primitive {
    cmm::types::Primitive::PointerInteger
}
