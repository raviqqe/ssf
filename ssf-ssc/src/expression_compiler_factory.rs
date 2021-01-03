use super::closure_operation_compiler::ClosureOperationCompiler;
use super::expression_compiler::ExpressionCompiler;
use super::function_application_compiler::FunctionApplicationCompiler;
use super::function_compiler::FunctionCompiler;
use super::malloc_compiler::MallocCompiler;
use super::type_compiler::TypeCompiler;
use std::sync::Arc;

pub struct ExpressionCompilerFactory<'c> {
    context: &'c inkwell::context::Context,
    function_application_compiler: Arc<FunctionApplicationCompiler<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    closure_operation_compiler: Arc<ClosureOperationCompiler<'c>>,
    malloc_compiler: Arc<MallocCompiler<'c>>,
}

impl<'c> ExpressionCompilerFactory<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        function_application_compiler: Arc<FunctionApplicationCompiler<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        closure_operation_compiler: Arc<ClosureOperationCompiler<'c>>,
        malloc_compiler: Arc<MallocCompiler<'c>>,
    ) -> Arc<Self> {
        Self {
            context,
            function_application_compiler,
            type_compiler,
            closure_operation_compiler,
            malloc_compiler,
        }
        .into()
    }

    pub fn create(
        &self,
        builder: Arc<inkwell::builder::Builder<'c>>,
        function_compiler: Arc<FunctionCompiler<'c>>,
    ) -> Arc<ExpressionCompiler<'c>> {
        ExpressionCompiler::new(
            self.context,
            builder,
            function_compiler,
            self.function_application_compiler.clone(),
            types::clone(),
            self.closure_operation_compiler.clone(),
            self.malloc_compiler.clone(),
        )
    }
}
