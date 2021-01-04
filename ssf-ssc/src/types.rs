pub const FUNCTION_ARGUMENT_OFFSET: usize = 1;

pub fn compile_generic_pointer() -> ssc::types::Pointer {
    ssc::types::Pointer::new(ssc::types::Primitive::Integer8)
}

pub fn get_arity(type_: &ssc::types::Function) -> usize {
    type_.arguments().len() - FUNCTION_ARGUMENT_OFFSET
}

pub fn compile(type_: &ssf::types::Type) -> ssc::types::Type {
    match type_ {
        ssf::types::Type::Algebraic(algebraic) => compile_algebraic(algebraic, None).into(),
        ssf::types::Type::Function(function) => {
            ssc::types::Pointer::new(compile_unsized_closure(function)).into()
        }
        ssf::types::Type::Index(_) => unreachable!(),
        ssf::types::Type::Primitive(primitive) => compile_primitive(primitive),
    }
}

pub fn compile_primitive(primitive: &ssf::types::Primitive) -> ssc::types::Type {
    match primitive {
        ssf::types::Primitive::Float32 => ssc::types::Primitive::Float32.into(),
        ssf::types::Primitive::Float64 => ssc::types::Primitive::Float64.into(),
        ssf::types::Primitive::Integer8 => ssc::types::Primitive::Integer8.into(),
        ssf::types::Primitive::Integer32 => ssc::types::Primitive::Integer32.into(),
        ssf::types::Primitive::Integer64 => ssc::types::Primitive::Integer64.into(),
        ssf::types::Primitive::Pointer => {
            ssc::types::Pointer::new(ssc::types::Primitive::Integer8).into()
        }
    }
}

pub fn compile_algebraic(
    algebraic: &ssf::types::Algebraic,
    index: Option<u64>,
) -> ssc::types::Record {
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

    ssc::types::Record::new(elements)
}

pub fn compile_untyped_constructor(algebraic_type: &ssf::types::Algebraic) -> ssc::types::Union {
    ssc::types::Union::new(
        algebraic_type
            .constructors()
            .iter()
            .map(|(_, constructor)| compile_constructor(constructor, true))
            .collect(),
    )
}

pub fn compile_sized_closure(definition: &ssf::ir::Definition) -> ssc::types::Record {
    compile_raw_closure(
        compile_entry_function_from_definition(definition),
        ssc::types::Union::new(vec![
            compile_environment(definition).into(),
            compile(definition.result_type()),
        ]),
    )
}

pub fn compile_unsized_closure(type_: &ssf::types::Function) -> ssc::types::Record {
    compile_raw_closure(
        compile_uncurried_entry_function(type_),
        compile_unsized_environment(),
    )
}

pub fn compile_raw_closure(
    entry_function: ssc::types::Function,
    environment: impl Into<ssc::types::Type>,
) -> ssc::types::Record {
    ssc::types::Record::new(vec![
        entry_function.into(),
        compile_arity().into(),
        environment.into(),
    ])
}

pub fn compile_environment(definition: &ssf::ir::Definition) -> ssc::types::Record {
    compile_raw_environment(
        definition
            .environment()
            .iter()
            .map(|argument| compile(argument.type_())),
    )
}

pub fn compile_raw_environment(
    types: impl IntoIterator<Item = ssc::types::Type>,
) -> ssc::types::Record {
    ssc::types::Record::new(types.into_iter().collect())
}

pub fn compile_unsized_environment() -> ssc::types::Record {
    ssc::types::Record::new(vec![])
}

pub fn compile_curried_entry_function(
    function: &ssc::types::Function,
    arity: usize,
) -> ssc::types::Function {
    if arity == get_arity(function) {
        function.clone()
    } else {
        ssc::types::Function::new(
            function.arguments()[..arity + FUNCTION_ARGUMENT_OFFSET].to_vec(),
            ssc::types::Pointer::new(compile_raw_closure(
                ssc::types::Function::new(
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

fn compile_uncurried_entry_function(function: &ssf::types::Function) -> ssc::types::Function {
    ssc::types::Function::new(
        vec![ssc::types::Pointer::new(compile_unsized_environment()).into()]
            .into_iter()
            .chain(function.arguments().into_iter().map(compile))
            .collect(),
        compile(function.last_result()),
    )
}

pub fn compile_entry_function_from_definition(
    definition: &ssf::ir::Definition,
) -> ssc::types::Function {
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
) -> ssc::types::Function {
    ssc::types::Function::new(
        vec![ssc::types::Pointer::new(compile_unsized_environment()).into()]
            .into_iter()
            .chain(arguments.into_iter().map(compile))
            .collect(),
        compile(result),
    )
}

pub fn compile_foreign_function(function: &ssf::types::Function) -> ssc::types::Function {
    ssc::types::Function::new(
        function.arguments().into_iter().map(compile).collect(),
        compile(function.last_result()),
    )
}

fn compile_constructor(constructor: &ssf::types::Constructor, shallow: bool) -> ssc::types::Type {
    let type_ = compile_unboxed_constructor(&if shallow && constructor.is_boxed() {
        ssf::types::Constructor::boxed(vec![])
    } else {
        constructor.clone()
    });

    if constructor.is_boxed() {
        ssc::types::Pointer::new(type_).into()
    } else {
        type_.into()
    }
}

pub fn compile_unboxed_constructor(constructor: &ssf::types::Constructor) -> ssc::types::Record {
    ssc::types::Record::new(constructor.elements().iter().map(compile).collect())
}

pub fn compile_tag() -> ssc::types::Primitive {
    ssc::types::Primitive::PointerInteger
}

pub fn compile_arity() -> ssc::types::Primitive {
    ssc::types::Primitive::PointerInteger
}
