use crate::function_applications;
use crate::types;
use crate::variable_builder::VariableBuilder;

pub fn compile_foreign_definition(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::ForeignDefinition,
    function_type: &ssf::types::Function,
    global_variable_builder: &VariableBuilder,
) {
    let foreign_function_type = types::compile_foreign_function(function_type);
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
            instruction_builder.return_(function_applications::compile(
                module_builder,
                &instruction_builder,
                global_variable_builder.build(&instruction_builder),
                &arguments
                    .iter()
                    .map(|argument| fmm::build::variable(argument.name(), argument.type_().clone()))
                    .collect::<Vec<_>>(),
            ))
        },
        foreign_function_type.result().clone(),
        fmm::types::CallingConvention::Target,
        true,
    );
}
