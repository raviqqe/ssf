pub const FUNCTION_ARGUMENT_OFFSET: usize = 1;

pub fn compile_generic_pointer() -> fmm::types::Pointer {
    fmm::types::Pointer::new(fmm::types::Primitive::Integer8)
}

pub fn get_arity(type_: &fmm::types::Function) -> usize {
    type_.arguments().len() - FUNCTION_ARGUMENT_OFFSET
}

pub fn compile(type_: &ssf::types::Type) -> fmm::types::Type {
    match type_ {
        ssf::types::Type::Algebraic(algebraic) => compile_algebraic(algebraic, None).into(),
        ssf::types::Type::Function(function) => {
            fmm::types::Pointer::new(compile_unsized_closure(function)).into()
        }
        ssf::types::Type::Index(_) => unreachable!(),
        ssf::types::Type::Primitive(primitive) => compile_primitive(primitive),
    }
}

pub fn compile_primitive(primitive: &ssf::types::Primitive) -> fmm::types::Type {
    match primitive {
        ssf::types::Primitive::Float32 => fmm::types::Primitive::Float32.into(),
        ssf::types::Primitive::Float64 => fmm::types::Primitive::Float64.into(),
        ssf::types::Primitive::Integer8 => fmm::types::Primitive::Integer8.into(),
        ssf::types::Primitive::Integer32 => fmm::types::Primitive::Integer32.into(),
        ssf::types::Primitive::Integer64 => fmm::types::Primitive::Integer64.into(),
        ssf::types::Primitive::Pointer => {
            fmm::types::Pointer::new(fmm::types::Primitive::Integer8).into()
        }
    }
}

pub fn compile_algebraic(
    algebraic: &ssf::types::Algebraic,
    index: Option<u64>,
) -> fmm::types::Record {
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

    fmm::types::Record::new(elements)
}

pub fn compile_untyped_constructor(algebraic_type: &ssf::types::Algebraic) -> fmm::types::Union {
    fmm::types::Union::new(
        algebraic_type
            .constructors()
            .iter()
            .map(|(_, constructor)| compile_constructor(constructor, true))
            .collect(),
    )
}

pub fn get_constructor_union_index(algebraic_type: &ssf::types::Algebraic, tag: u64) -> usize {
    algebraic_type
        .constructors()
        .iter()
        .enumerate()
        .find(|(index, (constructor_tag, _))| **constructor_tag == tag)
        .unwrap()
        .0
}

pub fn compile_sized_closure(definition: &ssf::ir::Definition) -> fmm::types::Record {
    compile_raw_closure(
        compile_entry_function_from_definition(definition),
        fmm::types::Union::new(vec![
            compile_environment(definition).into(),
            compile(definition.result_type()),
        ]),
    )
}

pub fn compile_unsized_closure(type_: &ssf::types::Function) -> fmm::types::Record {
    compile_raw_closure(
        compile_uncurried_entry_function(type_),
        compile_unsized_environment(),
    )
}

pub fn compile_raw_closure(
    entry_function: fmm::types::Function,
    environment: impl Into<fmm::types::Type>,
) -> fmm::types::Record {
    fmm::types::Record::new(vec![
        entry_function.into(),
        compile_arity().into(),
        environment.into(),
    ])
}

pub fn compile_environment(definition: &ssf::ir::Definition) -> fmm::types::Record {
    compile_raw_environment(
        definition
            .environment()
            .iter()
            .map(|argument| compile(argument.type_())),
    )
}

pub fn compile_raw_environment(
    types: impl IntoIterator<Item = fmm::types::Type>,
) -> fmm::types::Record {
    fmm::types::Record::new(types.into_iter().collect())
}

pub fn compile_unsized_environment() -> fmm::types::Record {
    fmm::types::Record::new(vec![])
}

pub fn compile_curried_entry_function(
    function: &fmm::types::Function,
    arity: usize,
) -> fmm::types::Function {
    if arity == get_arity(function) {
        function.clone()
    } else {
        fmm::types::Function::new(
            function.arguments()[..arity + FUNCTION_ARGUMENT_OFFSET].to_vec(),
            fmm::types::Pointer::new(compile_raw_closure(
                fmm::types::Function::new(
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

fn compile_uncurried_entry_function(function: &ssf::types::Function) -> fmm::types::Function {
    fmm::types::Function::new(
        vec![fmm::types::Pointer::new(compile_unsized_environment()).into()]
            .into_iter()
            .chain(function.arguments().into_iter().map(compile))
            .collect(),
        compile(function.last_result()),
    )
}

pub fn compile_entry_function_from_definition(
    definition: &ssf::ir::Definition,
) -> fmm::types::Function {
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
) -> fmm::types::Function {
    fmm::types::Function::new(
        vec![fmm::types::Pointer::new(compile_unsized_environment()).into()]
            .into_iter()
            .chain(arguments.into_iter().map(compile))
            .collect(),
        compile(result),
    )
}

pub fn compile_foreign_function(function: &ssf::types::Function) -> fmm::types::Function {
    fmm::types::Function::new(
        function.arguments().into_iter().map(compile).collect(),
        compile(function.last_result()),
    )
}

fn compile_constructor(constructor: &ssf::types::Constructor, shallow: bool) -> fmm::types::Type {
    let type_ = compile_unboxed_constructor(&if shallow && constructor.is_boxed() {
        ssf::types::Constructor::boxed(vec![])
    } else {
        constructor.clone()
    });

    if constructor.is_boxed() {
        fmm::types::Pointer::new(type_).into()
    } else {
        type_.into()
    }
}

pub fn compile_unboxed_constructor(constructor: &ssf::types::Constructor) -> fmm::types::Record {
    fmm::types::Record::new(constructor.elements().iter().map(compile).collect())
}

pub fn compile_tag() -> fmm::types::Primitive {
    fmm::types::Primitive::PointerInteger
}

pub fn compile_arity() -> fmm::types::Primitive {
    fmm::types::Primitive::PointerInteger
}