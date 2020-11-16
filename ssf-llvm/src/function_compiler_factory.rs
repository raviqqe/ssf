use super::expression_compiler_factory::ExpressionCompilerFactory;
use super::function_compiler::FunctionCompiler;
use super::global_variable::GlobalVariable;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;
use std::sync::Arc;

pub struct FunctionCompilerFactory<'c> {
    context: &'c inkwell::context::Context,
    module: Arc<inkwell::module::Module<'c>>,
    expression_compiler_factory: Arc<ExpressionCompilerFactory<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
}

impl<'c> FunctionCompilerFactory<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: Arc<inkwell::module::Module<'c>>,
        expression_compiler_factory: Arc<ExpressionCompilerFactory<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
    ) -> Arc<Self> {
        Self {
            context,
            module,
            expression_compiler_factory,
            type_compiler,
        }
        .into()
    }

    pub fn create(
        &self,
        global_variables: Arc<HashMap<String, GlobalVariable<'c>>>,
    ) -> Arc<FunctionCompiler<'c>> {
        FunctionCompiler::new(
            self.context,
            self.module.clone(),
            self.expression_compiler_factory.clone(),
            self.type_compiler.clone(),
            global_variables,
        )
    }
}
