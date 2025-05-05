use std::{ffi::CString, ptr};

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction},
    core::{
        LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildAdd, LLVMBuildFDiv, LLVMBuildFRem,
        LLVMBuildMul, LLVMBuildNeg, LLVMBuildNot, LLVMBuildRet, LLVMBuildSDiv, LLVMBuildSRem,
        LLVMBuildSub, LLVMBuildUDiv, LLVMBuildURem, LLVMConstInt, LLVMContextCreate,
        LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMDisposeModule,
        LLVMDumpModule, LLVMFunctionType, LLVMInt32TypeInContext,
        LLVMModuleCreateWithNameInContext, LLVMPositionBuilderAtEnd, LLVMSetSourceFileName,
    },
    prelude::{
        LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef,
    },
};

use crate::ast::{
    BinaryOperator, Expression, ExpressionKind, FunctionDefinition, Statement, StatementKind,
    TranslationUnit, UnaryOperator,
};

#[derive(Debug)]
pub struct Codegen {
    builder: LLVMBuilder,
    module: LLVMModule,
    context: LLVMContext,
}

#[expect(clippy::unwrap_used, clippy::undocumented_unsafe_blocks)]
impl Codegen {
    pub fn new(file_path: &str) -> Self {
        let module_name = CString::new(file_path).unwrap();

        let context = LLVMContext::new();
        let module = LLVMModule::new_in_context(module_name.clone(), &context);
        let builder = LLVMBuilder::new_in_context(&context);

        module.set_source_file_name(&module_name);

        Self {
            builder,
            module,
            context,
        }
    }

    pub fn dump(&self) {
        unsafe { LLVMDumpModule(self.module.0) };
    }

    #[must_use]
    fn int32_type(&self) -> LLVMTypeRef {
        self.context.int32_type()
    }

    #[must_use]
    fn function_type(&self, return_type: LLVMTypeRef) -> LLVMTypeRef {
        unsafe { LLVMFunctionType(return_type, ptr::null_mut(), 0, 0) }
    }

    fn function(&self, name: &str, function_type: LLVMTypeRef) -> LLVMValueRef {
        let Ok(function_name) = CString::new(name) else {
            return ptr::null_mut();
        };

        self.module.add_function(&function_name, function_type)
    }

    fn function_basic_block(&self, name: &str, function: LLVMValueRef) -> LLVMBasicBlockRef {
        let Ok(block_name) = CString::new(name) else {
            return ptr::null_mut();
        };

        let basic_block = self
            .context
            .create_basic_block_for_function(function, &block_name);

        // Move the builder to the end of the basic block
        self.builder.position_at_end(basic_block);

        basic_block
    }

    #[must_use]
    fn const_int(&self, value: u32) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.int32_type(), u64::from(value), 0) }
    }

    #[must_use]
    fn negate(&self, value: LLVMValueRef) -> LLVMValueRef {
        self.builder.negate(value)
    }

    #[must_use]
    fn not(&self, value: LLVMValueRef) -> LLVMValueRef {
        self.builder.not(value)
    }

    pub fn codegen(&self, translation_unit: &TranslationUnit) {
        // Code gen all functions
        for function in &translation_unit.function {
            self.codegen_function(function);
        }
    }

    fn codegen_function(&self, function: &FunctionDefinition) -> Option<()> {
        // Create the function type
        let function_type = self.function_type(self.int32_type());

        // Create the function
        let llvm_function = self.function(&function.name, function_type);
        if llvm_function.is_null() {
            return None;
        }

        // Create a basic block in the function and set our builder to generate
        // code in it.
        self.function_basic_block("entry", llvm_function);

        // Codegen the function body
        self.codegen_statement(&function.body);

        // Verify generated function
        unsafe {
            LLVMVerifyFunction(
                llvm_function,
                LLVMVerifierFailureAction::LLVMPrintMessageAction,
            )
        };

        Some(())
    }

    fn codegen_statement(&self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Return(expression) => {
                let value = self.codegen_expression(expression);

                self.builder.ret(value);
            }
        }
    }

    fn codegen_expression(&self, expression: &Expression) -> LLVMValueRef {
        match &expression.kind {
            ExpressionKind::IntegerLiteral(value) => self.const_int(*value),
            ExpressionKind::UnaryOperation {
                operator,
                expression,
            } => self.codegen_unary_operation(operator, expression.as_ref()),
            ExpressionKind::Parenthesis(expression) => self.codegen_expression(expression),
            ExpressionKind::BinaryOperation {
                operator,
                left,
                right,
            } => self.codegen_binary_operation(operator, left, right),
        }
    }

    fn codegen_binary_operation(
        &self,
        operator: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> LLVMValueRef {
        let left_value = self.codegen_expression(left);
        let right_value = self.codegen_expression(right);

        match operator {
            BinaryOperator::Add => self.builder.add(left_value, right_value),
            BinaryOperator::Subtract => self.builder.subtract(left_value, right_value),
            BinaryOperator::Multiply => self.builder.multiply(left_value, right_value),
            BinaryOperator::Divide => self.builder.signed_divide(left_value, right_value),
            BinaryOperator::Remainder => self.builder.signed_remainder(left_value, right_value),
        }
    }

    fn codegen_unary_operation(
        &self,
        operator: &UnaryOperator,
        expression: &Expression,
    ) -> LLVMValueRef {
        let value = self.codegen_expression(expression);

        match operator {
            UnaryOperator::Negate => self.negate(value),
            UnaryOperator::Complement => self.not(value),
        }
    }
}

// -- LLVM Wrappers --

#[derive(Debug)]
struct LLVMContext(LLVMContextRef);

#[expect(clippy::undocumented_unsafe_blocks)]
impl LLVMContext {
    pub fn new() -> Self {
        let context = unsafe { LLVMContextCreate() };
        Self(context)
    }

    pub fn int32_type(&self) -> LLVMTypeRef {
        unsafe { LLVMInt32TypeInContext(self.0) }
    }

    pub fn create_basic_block_for_function(
        &self,
        function: LLVMValueRef,
        name: &CString,
    ) -> LLVMBasicBlockRef {
        unsafe { LLVMAppendBasicBlockInContext(self.0, function, name.as_ptr()) }
    }
}

#[expect(clippy::undocumented_unsafe_blocks)]
impl Drop for LLVMContext {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.0) };
    }
}

#[derive(Debug)]
struct LLVMModule(LLVMModuleRef);

#[expect(clippy::undocumented_unsafe_blocks)]
impl LLVMModule {
    pub fn new_in_context<S: Into<CString>>(name: S, context: &LLVMContext) -> Self {
        let module = unsafe { LLVMModuleCreateWithNameInContext(name.into().as_ptr(), context.0) };
        Self(module)
    }

    pub fn set_source_file_name(&self, name: &CString) {
        unsafe { LLVMSetSourceFileName(self.0, name.as_ptr(), name.as_bytes().len()) };
    }

    pub fn add_function(&self, name: &CString, function_type: LLVMTypeRef) -> LLVMValueRef {
        unsafe { LLVMAddFunction(self.0, name.as_ptr(), function_type) }
    }
}

#[expect(clippy::undocumented_unsafe_blocks)]
impl Drop for LLVMModule {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.0) };
    }
}

#[derive(Debug)]
struct LLVMBuilder(LLVMBuilderRef);

#[expect(clippy::undocumented_unsafe_blocks, clippy::unwrap_used)]
impl LLVMBuilder {
    pub fn new_in_context(context: &LLVMContext) -> Self {
        let builder = unsafe { LLVMCreateBuilderInContext(context.0) };
        Self(builder)
    }

    fn position_at_end(&self, basic_block: LLVMBasicBlockRef) {
        unsafe { LLVMPositionBuilderAtEnd(self.0, basic_block) };
    }

    fn ret(&self, value: LLVMValueRef) {
        unsafe { LLVMBuildRet(self.0, value) };
    }

    fn not(&self, value: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("not").unwrap();
        unsafe { LLVMBuildNot(self.0, value, name.as_ptr()) }
    }

    fn negate(&self, value: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("neg").unwrap();
        unsafe { LLVMBuildNeg(self.0, value, name.as_ptr()) }
    }

    fn add(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("add").unwrap();
        unsafe { LLVMBuildAdd(self.0, left, right, name.as_ptr()) }
    }

    fn subtract(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("sub").unwrap();
        unsafe { LLVMBuildSub(self.0, left, right, name.as_ptr()) }
    }

    fn multiply(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("mul").unwrap();
        unsafe { LLVMBuildMul(self.0, left, right, name.as_ptr()) }
    }

    fn signed_divide(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("sdiv").unwrap();
        unsafe { LLVMBuildSDiv(self.0, left, right, name.as_ptr()) }
    }

    fn unsigned_divide(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("udiv").unwrap();
        unsafe { LLVMBuildUDiv(self.0, left, right, name.as_ptr()) }
    }

    fn float_divide(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("fdiv").unwrap();
        unsafe { LLVMBuildFDiv(self.0, left, right, name.as_ptr()) }
    }

    fn signed_remainder(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("srem").unwrap();
        unsafe { LLVMBuildSRem(self.0, left, right, name.as_ptr()) }
    }

    fn unsigned_remainder(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("urem").unwrap();
        unsafe { LLVMBuildURem(self.0, left, right, name.as_ptr()) }
    }

    fn float_remainder(&self, left: LLVMValueRef, right: LLVMValueRef) -> LLVMValueRef {
        let name = CString::new("frem").unwrap();
        unsafe { LLVMBuildFRem(self.0, left, right, name.as_ptr()) }
    }
}

#[expect(clippy::undocumented_unsafe_blocks)]
impl Drop for LLVMBuilder {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.0) };
    }
}
