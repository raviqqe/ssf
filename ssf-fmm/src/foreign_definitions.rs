use crate::function_applications;
use crate::types;

pub fn compile_foreign_definition(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::ForeignDefinition,
    function_type: &ssf::types::Function,
    global_variable: &fmm::build::TypedExpression,
) -> Result<(), fmm::build::BuildError> {
    // TODO Support a target calling convention.
    let foreign_function_type =
        types::compile_foreign_function(function_type, ssf::ir::CallingConvention::Source);
    let arguments = foreign_function_type
        .arguments()
        .iter()
        .enumerate()
        .map(|(index, type_)| fmm::ir::Argument::new(format!("arg_{}", index), type_.clone()))
        .collect::<Vec<_>>();

    module_builder.define_function(
        definition.foreign_name(),
        arguments.clone(),
        |instruction_builder| {
            Ok(instruction_builder.return_(function_applications::compile(
                module_builder,
                &instruction_builder,
                global_variable.clone(),
                &arguments
                    .iter()
                    .map(|argument| fmm::build::variable(argument.name(), argument.type_().clone()))
                    .collect::<Vec<_>>(),
            )?))
        },
        foreign_function_type.result().clone(),
        foreign_function_type.calling_convention(),
        fmm::ir::Linkage::External,
    )?;

    Ok(())
}
